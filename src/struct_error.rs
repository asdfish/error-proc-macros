use crate::{
    common::{attrs_get_lit_str, display_field},
    prelude::*,
};

pub struct StructError<'a> {
    ident: &'a Ident,
    format: &'a LitStr,
    generics: &'a Generics,
    variant: StructErrorVariant<'a>,
}
impl<'a> From<&'a DeriveInput> for StructError<'a> {
    fn from(input: &'a DeriveInput) -> Self {
        let Data::Struct(data) = &input.data else {
            Diagnostic::new(
                Level::Error,
                String::from("the `StructError` only works on structs"),
            )
            .help(String::from("remove"))
            .abort()
        };

        Self {
            ident: &input.ident,
            format: attrs_get_lit_str(&input.attrs, "format").unwrap_or_else(|_| {
                Diagnostic::new(
                    Level::Error,
                    String::from(
                        "failed to get required attribute `format` for macro `StructError`",
                    ),
                )
                .help(String::from("add `#[format = \"...\"]`"))
                .abort()
            }),
            generics: &input.generics,
            variant: StructErrorVariant::from(&data.fields),
        }
    }
}
impl ToTokens for StructError<'_> {
    fn to_tokens(&self, output: &mut TokenStream2) {
        output.extend(
            self.variant
                .to_display_impl(self.ident, self.generics, self.format),
        );
    }
}

pub enum StructErrorVariant<'a> {
    Named(Vec<(Option<&'a LitStr>, &'a Ident)>),
    SingleUnnamed,
    Unit,
    Unnamed(Vec<Option<&'a LitStr>>),
}
impl StructErrorVariant<'_> {
    /// Creates a display implementation
    pub fn to_display_impl(
        &self,
        self_ident: &Ident,
        self_generics: &Generics,
        self_format: &LitStr,
    ) -> TokenStream2 {
        let (impl_generics, ty_generics, where_clause) = self_generics.split_for_impl();

        match self {
            Self::Named(fields) => {
                let declarations = fields
                    .iter()
                    .map(|(display, field)| {
                        (
                            display,
                            field,
                            quote! {
                                self.#field
                            },
                        )
                    })
                    .map(|(display, field, self_field)| {
                        let display = display_field(display, &self_field);

                        quote! {
                            let #field = #display;
                        }
                    })
                    .collect::<TokenStream2>();

                quote! {
                    #[automatically_derived]
                    impl #impl_generics std::fmt::Display for #self_ident #ty_generics #where_clause {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                            #declarations

                            write!(f, #self_format)
                        }
                    }
                }
            }
            Self::SingleUnnamed => {
                quote! {
                    #[automatically_derived]
                    impl #impl_generics std::fmt::Display for #self_ident #ty_generics #where_clause {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                            write!(f, #self_format, self.0)
                        }
                    }
                }
            }
            Self::Unit => {
                quote! {
                    #[automatically_derived]
                    impl #impl_generics std::fmt::Display for #self_ident #ty_generics #where_clause {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                            write!(f, #self_format)
                        }
                    }
                }
            }
            Self::Unnamed(displays) => {
                let definitions = displays
                    .iter()
                    .enumerate()
                    .map(|(i, display)| {
                        (
                            display,
                            format!("arg_{i}").parse().unwrap_or_else(|error| {
                                syn::Error::from(error).into_compile_error()
                            }),
                            format!("self.{i}").parse().unwrap_or_else(|error| {
                                syn::Error::from(error).into_compile_error()
                            }),
                        )
                    })
                    .map(|(display, arg_var, self_var)| {
                        let display = display_field(display, &self_var);
                        quote! {
                            let #arg_var = #display;
                        }
                    })
                    .collect::<TokenStream2>();

                quote! {
                    #[automatically_derived]
                    impl #impl_generics std::fmt::Display for #self_ident #ty_generics #where_clause {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                            #definitions

                            write!(f, #self_format)
                        }
                    }
                }
            }
        }
    }
}
impl<'a> From<&'a Fields> for StructErrorVariant<'a> {
    fn from(fields: &'a Fields) -> Self {
        match fields {
            Fields::Named(fields) => Self::Named(
                fields
                    .named
                    .iter()
                    .map(|field| {
                        (
                            attrs_get_lit_str(&field.attrs, "display").ok(),
                            field.ident.as_ref().unwrap(),
                        )
                    })
                    .collect(),
            ),
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    Self::SingleUnnamed
                } else {
                    Self::Unnamed(
                        fields
                            .unnamed
                            .iter()
                            .map(|field| attrs_get_lit_str(&field.attrs, "display").ok())
                            .collect(),
                    )
                }
            }
            Fields::Unit => Self::Unit,
        }
    }
}
