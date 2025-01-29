use {
    proc_macro::TokenStream,
    proc_macro_error::{Diagnostic, Level, proc_macro_error},
    quote::quote,
    std::{collections::HashMap, str::FromStr},
    syn::{Data, DeriveInput, Fields, parse_macro_input},
};

/// Creates a error type from an enum.
///
/// # Examples
/// ```compile_fail
/// #[derive(EnumError)]
/// struct MyStruct { foo: u8, bar: i16 } // not an enum
/// #[derive(EnumError)]
/// union MyUnion { foo: u8, bar: i16 } // not an enum
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
                        Level::Warning,
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
            let token_stream = quote! {
                #[automatically_derived]
                impl From<#field_ty> for #ident {
                    fn from(error: #field_ty) -> Self {
                        Self::#field_ident(error)
                    }
                }
            };

            token_stream
        })
        .collect::<proc_macro2::TokenStream>();

    if variants.len() > 1 {
        let display_impl = format!(
            r#"
#[automatically_derived]
impl std::fmt::Display for {} {{
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {{
       match self {{
               {}
       }}
   }}
}}
"#,
            ident,
            variants
                .iter()
                .map(|(_, ident)| {
                    format!(r#"Self::{}(error) => write!(f, "{{}}", error),"#, ident)
                })
                .collect::<String>()
        );
        token_stream.extend(
            proc_macro2::TokenStream::from_str(&display_impl).unwrap_or_else(|error| {
                Diagnostic::new(Level::Warning, format!("{}", error)).abort()
            }),
        );
    }

    token_stream.into()
}
