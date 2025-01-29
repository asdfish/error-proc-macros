use {crate::prelude::*, enum_iterator::Sequence, std::ops::Not, syn::Fields};

struct EnumVariant<'a> {
    ident: &'a Ident,
    format: Option<&'a LitStr>,
}

#[derive(Debug)]
enum EnumVariants<'a> {
    Discriminant(&'a Expr),
    Typed(&'a Fields),
    Untyped,
}
impl EnumVariants<'_> {
    pub fn to_display_match_arm(
        &self,
        subject: &Ident,
        metadata: &EnumVariant<'_>,
    ) -> TokenStream2 {
        match self {
            Self::Discriminant(expr) => {
                let self_ident = &metadata.ident;

                match metadata.format {
                    Some(format) => quote! {
                        Self::#self_ident => format!(#format, Self::#self_ident),
                    },
                    None => Diagnostic::new(
                        Level::Error,
                        format!("enum variant `{}` must have a formatter", metadata.ident),
                    )
                        .help(String::from("add `#[format = \"...\"]`"))
                        .abort(),
                }
            }
            _ => todo!(),
        }
    }
}
impl<'a> From<&'a Variant> for EnumVariants<'a> {
    fn from(variant: &'a Variant) -> Self {
        [
            |variant: &'a Variant| {
                Some(EnumVariants::Discriminant(
                    variant.discriminant.as_ref().map(|(_, expr)| expr)?,
                ))
            },
            |variant: &'a Variant| {
                Some(EnumVariants::Typed(
                    variant.fields.is_empty().not().then_some(&variant.fields)?,
                ))
            },
            |variant: &'a Variant| variant.fields.is_empty().then_some(EnumVariants::Untyped),
        ]
        .into_iter()
        .flat_map(|builder| (builder)(variant))
        .next()
        .unwrap()
    }
}
