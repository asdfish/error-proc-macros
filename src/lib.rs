mod common;
mod enum_error;
mod prelude;
mod struct_error;

use {
    prelude::*,
    enum_error::EnumError,
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
    }
    .into()
}

/// Creates a error type from an enum.
/// # Attributes
/// | Attribute | Used on enum         | On on variant                                                 |
/// | --------- | -------------------- | ------------------------------------------------------------- |
/// | format    | Formats all variants | Formats this variant                                          |
/// | message   | Does nothing         | Makes the variant use this message if it does not have a type |
/// # Example
/// ```
/// use std::ffi::{
///   c_char,
///   CStr,
///   CString,
///   IntoStringError,
/// };
///
/// #[derive(EnumError)]
/// pub enum MyError {
///     IntoString(IntoStringError),
///     #[message = "unexpected null pointer"]
///     Null(NullError),
/// }
/// pub fn ptr_to_string<c_char>(ptr: *const c_char) -> Result<String, MyError> {
///    if ptr.is_null() { return Err(MyError::from(NullError)) };
///    CStr::from_ptr(ptr).into_c_string().into_string()?
/// }
/// ```
#[proc_macro_derive(EnumError, attributes(format, message))]
#[proc_macro_error]
pub fn enum_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    EnumError::from(&input).into_token_stream().into()
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

    let field_declarations = data
        .fields
        .iter()
        .map(|field| {
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
        })
        .collect::<TokenStream2>();

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
