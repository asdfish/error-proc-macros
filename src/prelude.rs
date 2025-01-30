pub use {
    proc_macro::TokenStream,
    proc_macro_error::{proc_macro_error, Diagnostic, Level},
    quote::{quote, ToTokens},
    std::collections::HashSet,
    syn::{
        parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, Ident, Lit, LitStr, Meta, Type,
        Variant,
    },
};

pub type TokenStream2 = proc_macro2::TokenStream;
