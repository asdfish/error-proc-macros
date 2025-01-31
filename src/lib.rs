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

/**
Creates an error type from an enum.

# Attributes
## `display`
Insert a closure to give a field formatting.
```
use {
    error_proc_macros::EnumError,
    std::path::Path,
};

#[derive(EnumError)]
enum PathError<'a> {
    #[format = "path `{}` does not exist"]
    #[display = "|path: &'a Path| path.display()"]
    NonExistant(&'a Path),
}

assert_eq!(
    PathError::NonExistant(Path::new("foo.txt")).to_string(),
    String::from("path `foo.txt` does not exist")
);
```

## `format`
Applies formatting.

### Argument access
| Variant Type   | Argument access |
| -------------- | --------------- |
| Single tuple   | `{}`            |
| Multiple tuple | `arg_{i}`       |
| Struct-like    | field name      |
| Unit           | inaccessable    |
*/
#[proc_macro_derive(EnumError, attributes(display, format))]
#[proc_macro_error]
pub fn enum_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    EnumError::from(&input).into_token_stream().into()
}

/**
Creates an error type from an struct.

# Attributes

# `format`
Format can only be used on the struct itself.
## Argument access
| Struct Type    | Argument access |
| -------------- | --------------- |
| Single tuple   | `{}`            |
| Multiple tuple | `arg_{i}`       |
| Named fields   | field name      |
| Unit struct    | inaccessable    |

## Examples
```
use error_proc_macros::StructError;

#[derive(StructError)]
#[format = "{foo}"]
struct MyError {
    foo: i8,
}
assert_eq!(MyError { foo: 10 }.to_string(), 10.to_string());
```
 */
#[proc_macro_derive(StructError, attributes(display, format))]
#[proc_macro_error]
pub fn struct_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    StructError::from(&input).into_token_stream().into()
}
