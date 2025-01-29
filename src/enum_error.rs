use crate::{
    common::attributes_get_lit_str,
    prelude::*,
};

struct DiscriminantVariant<'a> {
    ident: &'a Ident,
    format: &'a LitStr,
    value: &'a Expr,
}
impl<'a> DiscriminantVariant<'a> {
    pub fn new(
        ident: &'a Ident,
        format: &'a LitStr,
        value: &'a Expr,
    ) -> Self {
        Self {
            ident,
            format,
            value,
        }
    }
}
struct TypedVariant<'a> {
    ident: &'a Ident,
    format: Option<&'a LitStr>,
    ty: &'a Type,
}
impl<'a> TypedVariant<'a> {
    pub const fn new(ident: &'a Ident, ty: &'a Type) -> Self {
        Self {
            ident,
            format: None,
            ty,
        }
    }
}
struct UntypedVariant<'a> {
    ident: &'a Ident,
    format: Option<&'a LitStr>,
    message: &'a LitStr,
}
impl<'a> UntypedVariant<'a> {
    pub const fn new(ident: &'a Ident, message: &'a LitStr) -> Self {
        Self {
            ident,
            format: None,
            message,
        }
    }
}

enum EnumVariant<'a> {
    Discriminant(DiscriminantVariant<'a>),
    Typed(TypedVariant<'a>),
    Untyped(UntypedVariant<'a>)
}
impl EnumVariant<'_> {
    fn ident(&self) -> &Ident {
        match self {
            Self::Discriminant(variant) => variant.ident,
            Self::Typed(variant) => variant.ident,
            Self::Untyped(variant) => variant.ident,
        }
    }
    fn format(&self) -> Option<&LitStr> {
        match self {
            Self::Discriminant(variant) => Some(variant.format),
            Self::Typed(variant) => variant.format,
            Self::Untyped(variant) => variant.format,
        }
    }
}

pub struct EnumError<'a> {
    ident: &'a Ident,
    format: Option<&'a LitStr>,
    variants: Vec<EnumVariant<'a>>,
}
impl EnumError<'_> {
    /// check that the error is valid
    fn assert(&self) {
        let mut unique_types: HashSet<&Type> = HashSet::new();
        self.variants.iter().filter_map(|variant| {
            if let EnumVariant::Typed(variant) = variant {
                Some(variant)
            } else {
                None
            }
        })
            .for_each(|variant| {
                if unique_types.contains(&variant.ty) {
                    Diagnostic::new(Level::Error, String::from("enum errors should not contain duplicate types as it is redundant")).help(format!("remove variant `{}`", variant.ident)).abort();
                } else {
                    unique_types.insert(variant.ty);
                }
            });
    }

    /// Converts self into the match arms for a [Display] implementation.
    fn to_display_match_arms(&self) -> TokenStream2 {
        // all valid instances of Self should be asserted before construction
        // self.assert_messages();

        self.variants.iter().map(|variant| {
            let variant_format = variant.format();
            let variant_ident = variant.ident();

            if let Some(variant_format) = variant_format {
                match variant {
                    EnumVariant::Discriminant(variant) => {
                        let variant_value = &variant.value;

                        quote! {
                            Self::#variant_ident => format!(#variant_format, #variant_value)
                        }
                    },
                    EnumVariant::Typed(_) => {
                        quote! {
                            Self::#variant_ident(error) => format!(#variant_format, error),
                        }
                    },
                    EnumVariant::Untyped(variant) => {
                        let message = variant.message;

                        quote! {
                            Self::#variant_ident => format!(#variant_format, #message),
                        }
                    },
                }
            } else {
                match variant {
                    EnumVariant::Discriminant(variant) => {
                        let variant_value = &variant.value;

                        quote! {
                            Self::#variant_ident => format!("{}", #variant_value)
                        }
                    },
                    EnumVariant::Typed(_) => {
                        quote! {
                            Self::#variant_ident(error) => format!("{}", error),
                        }
                    },
                    EnumVariant::Untyped(variant) => {
                        let message = variant.message;

                        quote! {
                            Self::#variant_ident => format!("{}", #message),
                        }
                    },
                }
            }
        }).collect()
    }
    /// Creates a [Display][std::fmt::Display] implementation
    fn to_display_impl(&self) -> TokenStream2 {
        let self_ident = &self.ident;

        if self.variants.is_empty() {
            Diagnostic::new(Level::Error, String::from("enums without variants cannot be constructed, making it pointless")).help(String::from("remove enum or add variants")).abort();
        } else {
            let match_arms = self.to_display_match_arms();

            if let Some(self_format) = self.format {
                quote! {
                    #[automatically_derived]
                    impl std::fmt::Display for #self_ident {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                            write!(f, #self_format,
                                match self {
                                    #match_arms
                                }
                            )
                        }
                    }
                }
            } else {
                quote! {
                    #[automatically_derived]
                    impl std::fmt::Display for #self_ident {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                            write!(f, "{}",
                                match self {
                                    #match_arms
                                }
                            )
                        }
                    }
                }
            }
        }
    }
    /// Creates [From] implementations for all typed variants.
    fn to_from_impls(&self) -> TokenStream2 {
        self.variants.iter().filter_map(|variant| {
            let EnumVariant::Typed(variant) = variant else { return None };
            Some(variant)
        })
            .map(|variant| {
                let self_ident = &self.ident;
                let variant_ident = &variant.ident;
                let variant_ty = &variant.ty;

                quote! {
                    #[automatically_derived]
                    impl From<#variant_ty> for #self_ident {
                        fn from(error: #variant_ty) -> Self {
                            Self::#variant_ident(error)
                        }
                    }
                }
            }).collect()
    }
}
impl<'a> From<&'a DeriveInput> for EnumError<'a> {
    fn from(ast: &'a DeriveInput) -> Self {
        let mut output = Self {
            ident: &ast.ident,
            format: attributes_get_lit_str(&ast.attrs, "format").ok(),
            variants: Vec::new(),
        };

        let Data::Enum(data) = &ast.data else { Diagnostic::new(Level::Error, String::from("EnumError can only be used on enums")).abort() };
        output.variants = data.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let ty = variant.fields.iter().map(|field| &field.ty).next();

            if variant.discriminant.is_none() {
                match ty {
                    Some(ty) => EnumVariant::Typed(TypedVariant::new(ident, ty)),
                    None => EnumVariant::Untyped(UntypedVariant::new(ident, attributes_get_lit_str(&variant.attrs, "message").unwrap_or_else(|error| error.abort())))
                }
            } else {
                EnumVariant::Discriminant(DiscriminantVariant::new(ident, attributes_get_lit_str(&variant.attrs, "format").unwrap_or_else(|error| error.abort()), variant.discriminant.as_ref().map(|(_, expr)| expr).unwrap()))
            }
        }).collect();

        output.assert();
        output
    }
}
impl ToTokens for EnumError<'_> {
    fn to_tokens(&self, output: &mut proc_macro2::TokenStream) {
        [self.to_display_impl(), self.to_from_impls()].into_iter().for_each(|tokens| output.extend(tokens));
        // [self.to_display_impl(), self.to_from_impls()].into_iter().for_each(|tokens| { println!("{}", tokens);output.extend(tokens)});
    }
}
