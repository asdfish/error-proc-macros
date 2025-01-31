use crate::{common::attrs_get_lit_str, prelude::*};

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
    Named(Vec<&'a Ident>),
    SingleUnnamed,
    Unit,
    Unnamed { len: usize },
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
                quote! {
                    #[automatically_derived]
                    impl #impl_generics std::fmt::Display for #self_ident #ty_generics #where_clause {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                            #(let #fields = self.#fields;)*

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
            Self::Unnamed { len } => {
                let (arg_idents, self_idents): (Vec<TokenStream2>, Vec<TokenStream2>) = (0..*len)
                    .map(|i| {
                        (
                            format!("arg_{}", i).parse().unwrap(),
                            format!("self.{}", i).parse().unwrap(),
                        )
                    })
                    .unzip();

                quote! {
                    #[automatically_derived]
                    impl #impl_generics std::fmt::Display for #self_ident #ty_generics #where_clause {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                            #(let #arg_idents = &#self_idents;)*
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
            // flat map to avoid panic but may end up with unnamed field
            Fields::Named(fields) => {
                Self::Named(fields.named.iter().flat_map(|field| &field.ident).collect())
            }
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    Self::SingleUnnamed
                } else {
                    Self::Unnamed {
                        len: fields.unnamed.len(),
                    }
                }
            }
            Fields::Unit => Self::Unit,
        }
    }
}
