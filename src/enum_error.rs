use crate::{
    common::{attrs_get_lit_str, display_field},
    prelude::*,
};

/// Gets the `format` attribute but panics if not found.
fn get_required_format<'a>(attrs: &'a [Attribute], ident: &Ident) -> &'a LitStr {
    attrs_get_lit_str(attrs, "format").unwrap_or_else(|err| {
        Diagnostic::new(
            Level::Error,
            format!(
                "failed to get required attribute `format` for variant `{}`: {}",
                ident, err
            ),
        )
        .abort()
    })
}

/// The possible enum variants.
pub enum EnumVariant<'a> {
    AnonymousStruct {
        ident: &'a Ident,
        fields: Vec<(Option<&'a LitStr>, &'a Ident)>,
        format: &'a LitStr,
    },
    Discriminant {
        discriminant: &'a Expr,
        format: Option<&'a LitStr>,
        display: Option<&'a LitStr>,
        ident: &'a Ident,
    },
    SingleType {
        ident: &'a Ident,
        display: Option<&'a LitStr>,
        format: Option<&'a LitStr>,
        ty: &'a Type,
    },
    Tuple {
        ident: &'a Ident,
        format: &'a LitStr,
        displays: Vec<Option<&'a LitStr>>,
    },
    Unit {
        ident: &'a Ident,
        format: &'a LitStr,
    },
}
impl EnumVariant<'_> {
    pub fn to_display_match_arm(&self) -> TokenStream2 {
        match self {
            Self::AnonymousStruct {
                ident,
                fields,
                format,
            } => {
                let field_displays = fields
                    .iter()
                    .filter(|(display, _)| display.is_some())
                    .map(|(display, ident)| {
                        let display = display_field(display, ident);
                        quote! {
                            let #ident = #display;
                        }
                    })
                    .collect::<TokenStream2>();
                let fields = fields.iter().map(|(_, ident)| ident).collect::<Vec<_>>();

                quote! {
                    Self::#ident { #(#fields,)* } => {
                        #field_displays

                        format!(#format)
                    },
                }
            }
            Self::Discriminant {
                discriminant,
                display,
                ident,
                format,
            } => {
                let display = display_field(display, discriminant);

                match format {
                    Some(format) => quote! {
                        Self::#ident => format!(#format, #display),
                    },
                    None => quote! {
                        Self::#ident => format!("{}", #display),
                    },
                }
            }
            Self::SingleType {
                display,
                ident,
                format,
                ..
            } => {
                let error_var = quote! { error };
                let display = display_field(display, &error_var);

                match format {
                    Some(format) => {
                        quote! {
                            Self::#ident(#error_var) => format!(#format, #display),
                        }
                    }
                    None => {
                        quote! {
                            Self::#ident(#error_var) => format!("{}", #display),
                        }
                    }
                }
            }
            Self::Tuple {
                displays,
                ident,
                format,
            } => {
                let args = (0..displays.len())
                    .map(|i| {
                        format!("arg_{}", i)
                            .parse()
                            .unwrap_or_else(|error| syn::Error::from(error).into_compile_error())
                    })
                    .collect::<Vec<TokenStream2>>();
                let arg_displays = args
                    .iter()
                    .zip(displays)
                    .flat_map(|(arg, display)| {
                        display.map(|display| {
                            quote! {
                                let #arg = #display;
                            }
                        })
                    })
                    .collect::<TokenStream2>();

                quote! {
                    Self::#ident(#(#args),*) => {
                        #arg_displays
                        format!(#format)
                    },
                }
            }
            Self::Unit { ident, format } => quote! {
                Self::#ident => format!(#format),
            },
        }
    }
    pub fn to_from_impl(&self, onto: &Ident, generics: &Generics) -> Option<TokenStream2> {
        let Self::SingleType {
            ident, ty, format, ..
        } = self
        else {
            return None;
        };
        if format.is_some() {
            return None;
        }

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        Some(quote! {
            #[automatically_derived]
            impl #impl_generics From<#ty> for #onto #ty_generics #where_clause{
                fn from(error: #ty) -> Self {
                    Self::#ident(error)
                }
            }
        })
    }
}
impl<'a> From<&'a Variant> for EnumVariant<'a> {
    fn from(variant: &'a Variant) -> Self {
        if let Some(discriminant) = &variant.discriminant {
            if variant.fields == Fields::Unit {
                return Self::Discriminant {
                    discriminant: &discriminant.1,
                    display: attrs_get_lit_str(&variant.attrs, "display").ok(),
                    format: attrs_get_lit_str(&variant.attrs, "format").ok(),
                    ident: &variant.ident,
                };
            }
        }

        match &variant.fields {
            Fields::Named(fields) => Self::AnonymousStruct {
                ident: &variant.ident,
                fields: fields
                    .named
                    .iter()
                    .map(|field| {
                        (
                            attrs_get_lit_str(&field.attrs, "display").ok(),
                            field.ident.as_ref().unwrap(),
                        )
                    })
                    .collect(),
                format: get_required_format(&variant.attrs, &variant.ident),
            },
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    Self::SingleType {
                        display: attrs_get_lit_str(&variant.attrs, "display").ok(),
                        ident: &variant.ident,
                        format: attrs_get_lit_str(&variant.attrs, "format").ok(),
                        ty: fields.unnamed.iter().map(|field| &field.ty).next().unwrap(),
                    }
                } else {
                    Self::Tuple {
                        displays: fields
                            .unnamed
                            .iter()
                            .map(|field| attrs_get_lit_str(&field.attrs, "display").ok())
                            .collect(),
                        ident: &variant.ident,
                        format: get_required_format(&variant.attrs, &variant.ident),
                    }
                }
            }
            Fields::Unit => Self::Unit {
                ident: &variant.ident,
                format: get_required_format(&variant.attrs, &variant.ident),
            },
        }
    }
}

pub struct EnumError<'a> {
    ident: &'a Ident,
    format: Option<&'a LitStr>,
    generics: &'a Generics,
    variants: Vec<EnumVariant<'a>>,
}
impl EnumError<'_> {
    fn to_display_impl(&self) -> TokenStream2 {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let match_arms = self
            .variants
            .iter()
            .map(|variant| variant.to_display_match_arm())
            .collect::<TokenStream2>();

        match self.format {
            Some(format) => quote! {
                #[automatically_derived]
                impl #impl_generics std::fmt::Display for #ident #ty_generics #where_clause {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                        write!(f, #format, match self {
                            #match_arms
                        })
                    }
                }
            },
            None => quote! {
                #[automatically_derived]
                impl #impl_generics std::fmt::Display for #ident #ty_generics #where_clause {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                        write!(f, "{}", match self {
                            #match_arms
                        })
                    }
                }
            },
        }
    }
    fn to_from_impls(&self) -> TokenStream2 {
        self.variants
            .iter()
            .flat_map(|variant| variant.to_from_impl(self.ident, self.generics))
            .collect()
    }
}
impl<'a> From<&'a DeriveInput> for EnumError<'a> {
    fn from(input: &'a DeriveInput) -> Self {
        let Data::Enum(data) = &input.data else {
            Diagnostic::new(Level::Error, String::from("`EnumError` only works on enum"))
                .help(String::from("remove"))
                .abort()
        };
        let variants = data.variants.iter().map(EnumVariant::from).collect();

        Self {
            ident: &input.ident,
            format: attrs_get_lit_str(&input.attrs, "format").ok(),
            generics: &input.generics,
            variants,
        }
    }
}
impl ToTokens for EnumError<'_> {
    fn to_tokens(&self, output: &mut TokenStream2) {
        output.extend(
            [Self::to_display_impl, Self::to_from_impls]
                .into_iter()
                .map(|convertor| (convertor)(self))
                .collect::<TokenStream2>(),
        );
    }
}
