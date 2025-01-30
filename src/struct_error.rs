use crate::{
    common::attributes_get_lit_str,
    prelude::*,
};

pub enum StructError<'a> {
    Named {
        ident: &'a Ident,
        format: &'a LitStr,
        fields: Vec<&'a Ident>,
    },
    SingleUnnamed {
        ident: &'a Ident,
        format: &'a LitStr,
    },
    Unit {
        ident: &'a Ident,
        format: &'a LitStr,
    },
    Unnamed {
        ident: &'a Ident,
        format: &'a LitStr,
        len: usize,
    },
}
impl<'a> From<&'a DeriveInput> for StructError<'a> {
    fn from(input: &'a DeriveInput) -> Self {
        let ident = &input.ident;
        let format = attributes_get_lit_str(&input.attrs, "format").unwrap_or_else(|_| Diagnostic::new(Level::Error, String::from("the `format` attribute is required for the `StructError` derive macro")).help(String::from("add `#[format = \"...\"]`")).abort());

        let Data::Struct(data) = &input.data else {
            Diagnostic::new(Level::Error, String::from("the `StructError` macro may only be used on structs")).help(String::from("remove")).abort()
        };

        match &data.fields {
            Fields::Named(fields) => {
                Self::Named {
                    ident,
                    format,
                    fields: fields.named.iter().map(|field| field.ident.as_ref().unwrap()).collect(),
                }
            },
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    Self::SingleUnnamed {
                        ident,
                        format,
                    }
                } else {
                    Self::Unnamed {
                        ident,
                        format,
                        len: fields.unnamed.len(),
                    }
                }
            },
            Fields::Unit => Self::Unit {
                ident,
                format,
            },
        }
    }
}
impl ToTokens for StructError<'_> {
   fn to_tokens(&self, output: &mut TokenStream2) {
        output.extend(match self {
            Self::Named { ident, fields, format } => quote! {
                #[automatically_derived]
                impl std::fmt::Display for #ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                        #(let #fields = self.#fields;)*
                        write!(f, #format)
                    }
                }
            },
            Self::SingleUnnamed { ident, format, .. } => quote! {
                #[automatically_derived]
                impl std::fmt::Display for #ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                        write!(f, #format, self.0)
                    }
                }
            },
            Self::Unit { ident, format } => quote! {
                #[automatically_derived]
                impl std::fmt::Display for #ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                        write!(f, #format)
                    }
                }
            },
            Self::Unnamed { ident, format, len } => {
                let (variable_names, fields): (Vec<TokenStream2>, Vec<TokenStream2>) = (0..*len).map(|i| (format!("arg_{}", i).parse().unwrap(), format!("self.{}", i).parse().unwrap())).unzip();

                quote! {
                    #[automatically_derived]
                    impl std::fmt::Display for #ident {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                            #(let #variable_names = #fields;)*
                            write!(f, #format)
                        }
                    }
                }
            }
        });
    }
}
