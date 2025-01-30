use {crate::{common::attributes_get_lit_str, prelude::*}, syn::DataEnum};

fn get_required_format<'a>(attributes: &'a [Attribute], ident: &Ident) -> &'a LitStr {
    attributes_get_lit_str(attributes, "format").unwrap_or_else(|err| {
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

pub enum EnumVariant<'a> {
    AnonymousStruct {
        ident: &'a Ident,
        fields: Vec<(&'a Ident, &'a Type)>,
        format: &'a LitStr,
    },
    Discriminant {
        discriminant: &'a Expr,
        format: Option<&'a LitStr>,
        ident: &'a Ident,
    },
    SingleType {
        ident: &'a Ident,
        format: Option<&'a LitStr>,
        ty: &'a Type,
    },
    Tuple {
        ident: &'a Ident,
        format: &'a LitStr,
        types: Vec<&'a Type>,
    },
    Unit {
        ident: &'a Ident,
        format: &'a LitStr,
    },
}
impl EnumVariant<'_> {
    pub fn to_display_match_arm(&self) -> TokenStream2 {
        match self {
            Self::AnonymousStruct { ident, fields, format } => {
                let field_idents = fields.iter().map(|(ident, _)| ident).collect::<Vec<_>>();

                quote! {
                    Self::#ident { #(#field_idents,)* } => format!(#format),
                }
            },
            Self::Discriminant { discriminant, ident, format } => {
                match format {
                    Some(format) => quote! {
                        Self::#ident => format!(#format, #discriminant),
                    },
                    None => quote! {
                        Self::#ident => format!("{}", #discriminant),
                    },
                }
            },
            Self::SingleType { ident, format, .. } => {
                match format {
                    Some(format) => {
                        quote! {
                            Self::#ident(error) => format!(#format, error),
                        }
                    },
                    None => {
                        quote! {
                            Self::#ident(error) => format!("{}", error),
                        }
                    }
                }
            },
            Self::Tuple {
                ident,
                format,
                types,
            } => {
                let args = types.iter().enumerate().map(|(i, _)| format!("arg_{i}").parse().unwrap()).collect::<Vec<TokenStream2>>();

                quote! {
                    Self::#ident(#(#args),*) => format!(#format),
                }
            },
            Self::Unit { ident, format } => quote! {
                Self::#ident => format!(#format),
            },
        }
    }
    pub fn to_from_impl(&self, onto: &Ident) -> Option<TokenStream2> {
        let Self::SingleType { ident, ty, format } = self else { return None };
        if format.is_some() { return None; }

        Some(quote! {
            #[automatically_derived]
            impl From<#ty> for #onto {
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
                    format: attributes_get_lit_str(&variant.attrs, "format").ok(),
                    ident: &variant.ident,
                };
            }
        }

        match &variant.fields {
            Fields::Named(fields) => {
                Self::AnonymousStruct {
                    ident: &variant.ident,
                    fields: fields.named.iter().map(|field| (field.ident.as_ref().unwrap(), &field.ty)).collect(),
                    format: get_required_format(&variant.attrs, &variant.ident),
                }
            },
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    Self::SingleType {
                        ident: &variant.ident,
                        format: attributes_get_lit_str(&variant.attrs, "format").ok(),
                        ty: fields.unnamed.iter().map(|field| &field.ty).next().unwrap(),
                    }
                } else {
                    Self::Tuple {
                        ident: &variant.ident,
                        format: get_required_format(&variant.attrs, &variant.ident),
                        types: fields.unnamed.iter().map(|field| &field.ty).collect(),
                    }
                }
            },
            Fields::Unit => {
                Self::Unit {
                    ident: &variant.ident,
                    format: get_required_format(&variant.attrs, &variant.ident),
                }
            },
        }
    }
}

pub struct EnumError<'a> {
    ident: &'a Ident,
    format: Option<&'a LitStr>,
    variants: Vec<EnumVariant<'a>>,
}
impl EnumError<'_> {
    fn to_display_impl(&self) -> TokenStream2 {
        let ident = &self.ident;
        let match_arms = self.variants.iter().map(|variant| variant.to_display_match_arm()).collect::<TokenStream2>();

        match self.format {
            Some(format) => quote! {
                #[automatically_derived]
                impl std::fmt::Display for #ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                        write!(f, #format, match self {
                            #match_arms
                        })
                    }
                }
            },
            None => quote! {
                #[automatically_derived]
                impl std::fmt::Display for #ident {
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
        self.variants.iter().flat_map(|variant| variant.to_from_impl(&self.ident)).collect()
    }
}
impl<'a> From<&'a DeriveInput> for EnumError<'a> {
    fn from(input: &'a DeriveInput) -> Self {
        let Data::Enum(data) = &input.data else { Diagnostic::new(Level::Error, String::from("`EnumError` only works on enum")).help(String::from("remove")).abort() };
        let variants = data.variants.iter().map(|variant| EnumVariant::from(variant)).collect();

        Self {
            ident: &input.ident,
            format: attributes_get_lit_str(&input.attrs, "format").ok(),
            variants,
        }
    }
}
impl ToTokens for EnumError<'_> {
    fn to_tokens(&self, output: &mut TokenStream2) {
        output.extend(
            [Self::to_display_impl, Self::to_from_impls].into_iter().map(|convertor| (convertor)(self)).collect::<TokenStream2>()
        );
    }
}
