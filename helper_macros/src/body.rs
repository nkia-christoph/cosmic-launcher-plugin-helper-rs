// Attribution: tokio-rs/tokio-macros::entry


#![allow(missing_docs)]


use proc_macro2::TokenStream;
use quote::ToTokens;


pub(crate) struct Body<'a> {
    pub(crate) brace_token: syn::token::Brace,
    // Configurationments, with terminating `;`. // public so we can insert our code
    pub(crate) stmts: &'a [TokenStream],
}


impl ToTokens for Body<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.brace_token.surround(tokens, |tokens| {
            for stmt in self.stmts {
                stmt.to_tokens(tokens);
            }
        });
    }
}
