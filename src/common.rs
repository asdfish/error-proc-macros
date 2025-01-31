//! Shared functions

use crate::prelude::*;

pub fn attrs_get_value<'a>(
    attrs: &'a [Attribute],
    search: &'a str
) -> Result<&'a Expr, AttrsGetValueError<'a>> {
    let attr = attrs.iter().find(|attr| attr.path().is_ident(search)).ok_or(AttrsGetValueError::NotFound(search))?;
    let Meta::NameValue(name_value) = &attr.meta else {
        Err(AttrsGetValueError::NotNameValue(search))?
    };

    Ok(&name_value.value)
}

#[derive(Debug)]
pub enum AttrsGetValueError<'a> {
    NotNameValue(&'a str),
    NotFound(&'a str),
}

impl std::fmt::Display for AttrsGetValueError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::NotNameValue(attr) => write!(f, "attribute `{}` only accepts name value arguments", attr),
            Self::NotFound(attr) => write!(f, "attribute `{}` was not found", attr),
        }
    }
}
impl std::error::Error for AttrsGetValueError<'_> {}

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

#[derive(Debug)]
pub enum AttrsGetLitStrError<'a> {
    GetError(AttrsGetValueError<'a>),
    NotStringLiteral(&'a str),
}
impl std::fmt::Display for AttrsGetLitStrError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", match self {
            Self::GetError(error) => error.to_string(),
            Self::NotStringLiteral(attribute) => {
                format!("attribute `{}` only accepts string literals", attribute)
            }
        })
    }
}
impl<'a> From<AttrsGetValueError<'a>> for AttrsGetLitStrError<'a> {
    fn from(error: AttrsGetValueError<'a>) -> Self {
        Self::GetError(error)
    }
}
impl std::error::Error for AttrsGetLitStrError<'_> {}
