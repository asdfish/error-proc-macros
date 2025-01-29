use {
    proc_macro::TokenStream,
    proc_macro_error::{Diagnostic, Level, proc_macro_error},
    proc_macro2::Span,
    quote::quote,
    std::{collections::HashMap, str::FromStr},
    syn::{Data, DeriveInput, Expr, Ident, Lit, Meta, Path, parse_macro_input},
};

/// Saves you from typing ```impl std::error::Error for FooError {}```.
/// # Examples
/// ```
/// #[derive(Debug, Error, StructError)]
/// #[format = "scary error"]
/// pub struct MyError {}
/// let my_error: Box<dyn std::error::Error> = Box::new(MyError {});
/// ```
#[proc_macro_derive(Error)]
pub fn error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    quote! {
        impl std::error::Error for #ident {}
    }.into()
}

/// Creates a error type from an enum.
/// # Example
/// ```
/// use std::ffi::{
///   c_char,
///   CStr,
///   CString,
///   IntoStringError,
/// };
///
/// #[derive(StructError)]
/// #[format = "unexpected null pointer"]
/// pub struct NullError;
///
/// #[derive(EnumError)]
/// pub enum MyError {
///     IntoString(IntoStringError),
///     Null(NullError),
/// }
/// pub fn ptr_to_string<c_char>(ptr: *const c_char) -> Result<String, MyError> {
///    if ptr.is_null() { return Err(MyError::from(NullError)) };
///    CStr::from_ptr(ptr).into_c_string().into_string()?
/// }
/// ```
/// # Errors
/// ```compile_fail
/// #[derive(EnumError)]
/// struct MyStruct { foo: u8, bar: i16 } // not an enum
/// #[derive(EnumError)]
/// union MyUnion { foo: u8, bar: i16 } // not an enum
/// #[derive(EnumError)]
/// enum MyEnum { Foo = 0, Bar, Baz(u8) } // contains variants with no types
/// ```
#[proc_macro_derive(EnumError)]
#[proc_macro_error]
pub fn enum_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    let Data::Enum(data) = input.data else {
        Diagnostic::new(
            Level::Error,
            format!("EnumError cannot be called on non enum \"{}\"", ident),
        )
        .abort()
    };

    let mut variants = HashMap::new();

    data.variants
        .iter()
        .map(|variant| {
            (
                variant.fields.iter().map(|field| &field.ty).next(),
                &variant.ident,
            )
        })
        .map(|(option, ident)| {
            (
                option.unwrap_or_else(|| {
                    Diagnostic::new(
                        Level::Error,
                        String::from("all variants of an EnumError must be a named field"),
                    )
                    .help(format!("change field {} to have a type", ident))
                    .abort()
                }),
                ident,
            )
        })
        .for_each(|(ty, ident)| {
            variants
                .entry(ty)
                .and_modify(|_| {
                    Diagnostic::new(
                        Level::Error,
                        String::from("error variants should contain different types"),
                    )
                    .help(format!("remove field \"{}\"", ident))
                    .abort()
                })
                .or_insert(ident);
        });

    let mut token_stream = variants
        .iter()
        .map(|(field_ty, field_ident)| {
            quote! {
                #[automatically_derived]
                impl From<#field_ty> for #ident {
                    fn from(error: #field_ty) -> Self {
                        Self::#field_ident(error)
                    }
                }
            }
        })
        .collect::<proc_macro2::TokenStream>();

    token_stream.extend(if variants.len() > 1 {
        let match_arms = variants
            .iter()
            .map(|(_, ident)| {
                quote! {
                    Self::#ident(error) => write!(f, "{}", error),
                }
            })
            .collect::<proc_macro2::TokenStream>();

        quote! {
            #[automatically_derived]
            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                    match self {
                        #match_arms
                    }
                }
            }
        }
    } else {
        quote! {
            #[automatically_derived]
            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                    write!(f, "{}", self)
                }
            }
        }
    });

    token_stream.into()
}

#[proc_macro_derive(StructError, attributes(format))]
#[proc_macro_error]
pub fn struct_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let Data::Struct(data) = input.data else {
        Diagnostic::new(
            Level::Error,
            format!("StructError cannot be called on non struct `{}`", ident),
        )
        .abort()
    };

    let format = input
        .attrs
        .iter()
        .map(|attr| &attr.meta)
        .filter(|meta| meta.path().is_ident("format"))
        .map(|meta| {
            let Meta::NameValue(meta) = meta else {
                Diagnostic::new(
                    Level::Error,
                    String::from("the `format` attribute can only be used as a name value pair"),
                )
                .help(String::from("change to #[format = \"...\"]"))
                .abort()
            };
            meta
        })
        .next()
        .map(|meta| {
            let diagnostic = Diagnostic::new(
                Level::Error,
                String::from("the `format` attribute only accept string literals"),
            )
            .help(String::from("change to `#[format = \"...\"]`"));

            let Expr::Lit(value) = &meta.value else {
                diagnostic.abort()
            };
            let Lit::Str(value) = &value.lit else {
                diagnostic.abort()
            };

            value
        })
        .unwrap_or_else(|| {
            Diagnostic::new(
                Level::Error,
                String::from("a derived StructError must have a `format` attribute"),
            )
            .abort()
        });

    let field_declarations = data.fields.iter().map(|field| {
        field.ident.as_ref().unwrap_or_else(|| {
            Diagnostic::new(
                Level::Error,
                String::from("`StructError` can only be used on structs with named fields"),
            )
            .abort()
        })
    })
        .map(|ident| {
            quote! {
                let #ident = &self.#ident;
            }
        }).collect::<proc_macro2::TokenStream>();

    quote! {
        #[automatically_derived]
        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                #field_declarations

                write!(f, #format)
            }
        }
    }
    .into()
}
