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
    /**
    Enum variants with an anonymous field
    ```
    enum MyEnum {
         MyField { x: u8, y: i8 },
    }
    ```
    # Formatting
    Formatting is required as display is not implemented for anonymous structs.
    The arguments for the [format] macro become the names of the fields.
    ```
    enum MyEnum {
         #[format = "x: {x}, y: {y}"]
         MyField { x: u8, y: u8 },
    }
    let my_enum = MyEnum::MyField { x: 10, y: 10 };
    assert_eq!(String::from("x: 10, y: 10"), my_enum.to_string());
    ```*/
    AnonymousStruct {
        ident: &'a Ident,
        fields: Vec<(&'a Ident, &'a Type)>,
        format: &'a LitStr,
    },
    /**
    Enum variants with custom set discriminants.
    ```
    enum MyEnum {
         MyField = 10,
    }
    ```
    # Formattting
    Formatting is required because the discriminant has no meaningful message. */
    Discriminant {
        discriminant: &'a Expr,
        format: &'a LitStr,
        ident: &'a Ident,
    },
    /**
    Tuple variant with a single type
    ```
    enum MyEnum {
        MyField(i8),
    }
    ```
    # Formatting
    If the variant does not have a format, you can only have a singular instance of its type.
    This is so that the type can can implement [From] for the type.
    If it does have a format, it will not have a [From] implementation.

    The reason for this is that error types can only have a singular message, so having duplicates would not make sense, and implementing [From] would allow for the try operator.
     */
    SingleType {
        ident: &'a Ident,
        format: Option<&'a LitStr>,
        ty: &'a Type,
    },
    /**
    Tuple variant with multiple types
    It is recommended to use this over something like ```enum MyEnum { Foo((i8, i8)) }``` as tuples do not have formatting.
    ```
    enum MyEnum {
         MyField(i8, i8),
    }
    ```
    # Formatting
    Formatting is required.
    Numbers correspond to the type index and arg_ eg.
    ```
    enum MyEnum {
         #[format = "{arg_1}{arg_0}"]
         MyField(i8, i8)
    }
    assert_eq!(String::from("010"), format!("{}", MyEnum::MyField(10, 0)));
    ```
    */
    Tuple {
        ident: &'a Ident,
        format: &'a LitStr,
        types: Vec<&'a Type>,
    },
    /**
    Enum variants with no values associated with it.
    ```
    enum MyEnum {
         MyField,
    }
    ```
    # Formatting
    Formatting is required as a unit enum has no text associated with it.
    */
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
                quote! {
                    Self::#ident => format!(#format, #discriminant),
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
            return Self::Discriminant {
                discriminant: &discriminant.1,
                format: get_required_format(&variant.attrs, &variant.ident),
                ident: &variant.ident,
            };
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
