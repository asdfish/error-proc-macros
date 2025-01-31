//! Shared functions

use crate::prelude::*;

/// Get an [Expr] from [attributes][Attribute]
pub fn attrs_get_value<'a>(
    attrs: &'a [Attribute],
    search: &'a str,
) -> Result<&'a Expr, AttrsGetValueError<'a>> {
    let attr = attrs
        .iter()
        .find(|attr| attr.path().is_ident(search))
        .ok_or(AttrsGetValueError::NotFound(search))?;
    let Meta::NameValue(name_value) = &attr.meta else {
        Err(AttrsGetValueError::NotNameValue(search))?
    };

    Ok(&name_value.value)
}

/// Errors from [attrs_get_value]
#[derive(Debug)]
pub enum AttrsGetValueError<'a> {
    /// A attribute is not a [name value][MetaNameValue]
    NotNameValue(&'a str),
    /// The specified attribute is not found
    NotFound(&'a str),
}

impl std::fmt::Display for AttrsGetValueError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::NotNameValue(attr) => {
                write!(f, "attribute `{}` only accepts name value arguments", attr)
            }
            Self::NotFound(attr) => write!(f, "attribute `{}` was not found", attr),
        }
    }
}
impl std::error::Error for AttrsGetValueError<'_> {}

/// Extract a [string literal] from an attribute
pub fn attrs_get_lit_str<'a>(
    attrs: &'a [Attribute],
    search: &'a str,
) -> Result<&'a LitStr, AttrsGetLitStrError<'a>> {
    let expr = attrs_get_value(attrs, search)?;

    let Expr::Lit(lit) = &expr else {
        return Err(AttrsGetLitStrError::NotStringLiteral(search));
    };
    let Lit::Str(lit_str) = &lit.lit else {
        return Err(AttrsGetLitStrError::NotStringLiteral(search));
    };

    Ok(lit_str)
}

/// Errors from [attrs_get_lit_str]
#[derive(Debug)]
pub enum AttrsGetLitStrError<'a> {
    GetError(AttrsGetValueError<'a>),
    NotStringLiteral(&'a str),
}
impl std::fmt::Display for AttrsGetLitStrError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::GetError(error) => error.to_string(),
                Self::NotStringLiteral(attribute) => {
                    format!("attribute `{}` only accepts string literals", attribute)
                }
            }
        )
    }
}
impl<'a> From<AttrsGetValueError<'a>> for AttrsGetLitStrError<'a> {
    fn from(error: AttrsGetValueError<'a>) -> Self {
        Self::GetError(error)
    }
}
impl std::error::Error for AttrsGetLitStrError<'_> {}

/// Converts `variable` to something that implements display with the `display` closure if it exists, or return `variable`.
pub fn display_field<T: ToTokens + ?Sized>(
    display: &Option<&LitStr>,
    variable: &T,
) -> TokenStream2 {
    match display {
        Some(display) => {
            let conversion = display
                .parse()
                .unwrap_or_else(|err| err.into_compile_error());

            quote! {
                (#conversion)(#variable)
            }
        }
        None => variable.to_token_stream(),
    }
}
