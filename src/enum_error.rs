use crate::{
    common::attributes_get_lit_str,
    prelude::*,
};

struct TypedVariant<'a> {
    ident: &'a Ident,
    ty: &'a Type,
}
impl<'a> TypedVariant<'a> {
    pub const fn new(ident: &'a Ident, ty: &'a Type) -> Self {
        Self {
            ident,
            ty,
        }
    }
}
struct UntypedVariant<'a> {
    ident: &'a Ident,
    message: Option<&'a LitStr>
}
impl<'a> UntypedVariant<'a> {
    pub const fn new(ident: &'a Ident, message: Option<&'a LitStr>) -> Self {
        Self {
            ident,
            message,
        }
    }
}

enum EnumVariant<'a> {
    Typed(TypedVariant<'a>),
    Untyped(UntypedVariant<'a>)
}
impl<'a> EnumVariant<'a> {
    fn ident(&self) -> &Ident {
        match self {
            Self::Untyped(variant) => {
                &variant.ident
            },
            Self::Typed(variant) => {
                &variant.ident
            }
        }
    }
}

pub struct EnumError<'a> {
    message: Option<&'a LitStr>,
    ident: &'a Ident,
    variants: Vec<EnumVariant<'a>>,
}
impl EnumError<'_> {
    /// Runs both assert functions
    fn assert(&self) {
        [Self::assert_messages, Self::assert_unique_types].into_iter().for_each(|function| (function)(self));
    }
    /// Ensure that either [Self::message] is some or all [untyped variants][UntypedVariant] contain messages
    fn assert_messages(&self) {
        if self.message.is_none() {
            self.variants.iter().filter_map(|variant| {
                if let EnumVariant::Untyped(variant) = variant {
                    Some(variant)
                } else {
                    None
                }
            })
                .next()
                .inspect(|variant| if variant.message.is_none() {
                    Diagnostic::new(Level::Error, String::from("untyped variants of `EnumError` must all have messages")).help(String::from("insert `#[message = \"...\"]`")).abort()
                });
        }
    }
    /// Asserts that all [typed variants][TypedVariants] have unique types
    fn assert_unique_types(&self) {
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
    fn to_display_match_arms(&self, formatter: &Ident) -> TokenStream2 {
        // all valid instances of Self should be asserted before construction
        // self.assert_messages();

        self.variants.iter().map(|variant| {
            let variant_ident = variant.ident();

            match variant {
                EnumVariant::Typed(_) => {
                    quote! {
                        Self::#variant_ident(error) => write!(#formatter, "{}", error),
                    }
                },
                EnumVariant::Untyped(variant) => {
                    let message = variant.message.or(self.message).unwrap();

                    quote! {
                        Self::#variant_ident => write!(#formatter, "{}", #message),
                    }
                },
            }
        }).collect()
    }
    /// Creates a [Display][std::fmt::Display] implementation
    fn to_display_impl(&self) -> TokenStream2 {
        let formatter_var: Ident = Ident::new("f", Span::call_site());
        let self_ident = &self.ident;

        if self.variants.is_empty() {
            if self.message.is_none() {
                Diagnostic::new(Level::Error, String::from("cannot have empty `ErrorEnum` without message")).help(String::from("add `#[message = \"...\"]`")).abort();
            }
            let message = self.message.unwrap();

            quote! {
                impl std::fmt::Display for #self_ident {
                    fn fmt(&self, #formatter_var: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                        write!(#formatter_var, #message)
                    }
                }
            }
        } else {
            let match_arms = self.to_display_match_arms(&formatter_var);

            quote! {
                #[automatically_derived]
                impl std::fmt::Display for #self_ident {
                    fn fmt(&self, #formatter_var: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                        match self {
                            #match_arms
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
            message: attributes_get_lit_str(&ast.attrs, "message").ok(),
            ident: &ast.ident,
            variants: Vec::new(),
        };

        let Data::Enum(data) = &ast.data else { Diagnostic::new(Level::Error, String::from("EnumError can only be used on enums")).abort() };
        output.variants = data.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let ty = variant.fields.iter().map(|field| &field.ty).next();

            match ty {
                Some(ty) => EnumVariant::Typed(TypedVariant::new(ident, ty)),
                None => EnumVariant::Untyped(UntypedVariant::new(ident, attributes_get_lit_str(&variant.attrs, "message").ok()))
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
