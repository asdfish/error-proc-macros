pub use {
    proc_macro::TokenStream,
    proc_macro_error::{Diagnostic, Level, proc_macro_error},
    quote::{
        quote,
        ToTokens,
    },
    std::collections::HashSet,
    syn::{Data, DeriveInput, Expr, Ident, Lit, LitStr, Meta, Type, parse_macro_input},
};

pub type TokenStream2 = proc_macro2::TokenStream;
