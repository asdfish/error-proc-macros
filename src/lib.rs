mod common;
mod enum_error;
mod prelude;
mod struct_error;

use {
    enum_error::EnumError,
    struct_error::StructError,
    prelude::*,
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
#[proc_macro_derive(EnumError, attributes(format))]
#[proc_macro_error]
pub fn enum_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    EnumError::from(&input).into_token_stream().into()
}

#[proc_macro_derive(StructError, attributes(format))]
#[proc_macro_error]
pub fn struct_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    StructError::from(&input).into_token_stream().into()
}
