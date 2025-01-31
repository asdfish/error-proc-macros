mod common;
mod enum_error;
mod prelude;
mod struct_error;

use {enum_error::EnumError, prelude::*, struct_error::StructError};

/// Saves you from typing ```impl std::error::Error for FooError {}```.
/// # Examples
/// ```
/// use error_proc_macros::{Error, StructError};
/// #[derive(Debug, Error, StructError)]
/// #[format = "scary error"]
/// pub struct MyError;
/// let my_error: Box<dyn std::error::Error> = Box::new(MyError {});
/// ```
#[proc_macro_derive(Error)]
pub fn error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics std::error::Error for #ident #ty_generics #where_clause {}
    }
    .into()
}

/// Creates a error type from an enum.
/// # Attributes
/// | Attribute | Description |
/// | --------- | ----------- |
/// | display | A way for fields that do not implement `Display` to still be formatted. The string gets appended to the end of the field that is to be formatted. |
/// | format | Formats the specified field. If format is used on the enum itself, it formats all fields. |
/// # Example
/// ```
/// use {
///     error_proc_macros::EnumError,
///     std::{
///         ffi::{
///           c_char,
///           CStr,
///         },
///         str::Utf8Error,
///     },
/// };
///
/// #[derive(EnumError)]
/// pub enum PtrToStrError {
///     #[format = "unexpected null pointer"]
///     NullError,
///     Utf8Error(Utf8Error),
/// }
/// pub fn ptr_to_string(ptr: &*const c_char) -> Result<&str, PtrToStrError> {
///     if ptr.is_null() { return Err(PtrToStrError::NullError) };
///     Ok(unsafe { CStr::from_ptr(*ptr).to_str()? })
/// }
/// ```
#[proc_macro_derive(EnumError, attributes(display, format))]
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
