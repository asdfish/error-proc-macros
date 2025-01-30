//! Shared functions

use crate::prelude::*;

pub fn attributes_get_lit_str<'a>(
    attributes: &'a [Attribute],
    search_attribute: &'a str,
) -> Result<&'a LitStr, AttributesGetLitStrError<'a>> {
    let attribute = attributes
        .iter()
        .find(|attribute| attribute.path().is_ident(search_attribute))
        .ok_or(AttributesGetLitStrError::NotFound(search_attribute))?;
    let Meta::NameValue(name_value) = &attribute.meta else {
        return Err(AttributesGetLitStrError::NotStringLiteral(search_attribute));
    };
    let Expr::Lit(lit) = &name_value.value else {
        return Err(AttributesGetLitStrError::NotStringLiteral(search_attribute));
    };
    let Lit::Str(lit_str) = &lit.lit else {
        return Err(AttributesGetLitStrError::NotStringLiteral(search_attribute));
    };

    Ok(lit_str)
}

#[derive(Debug)]
pub enum AttributesGetLitStrError<'a> {
    NotFound(&'a str),
    NotStringLiteral(&'a str),
}
impl AttributesGetLitStrError<'_> {
    pub fn to_description(&self) -> String {
        match self {
            Self::NotFound(attribute) => format!("attribute `{}` was not found", attribute),
            Self::NotStringLiteral(attribute) => {
                format!("attribute `{}` only accepts string literals", attribute)
            }
        }
    }
}
impl std::fmt::Display for AttributesGetLitStrError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.to_description())
    }
}
impl std::error::Error for AttributesGetLitStrError<'_> {}
