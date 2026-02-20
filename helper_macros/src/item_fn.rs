// Attribution: tokio-rs/tokio-macros::entry


#![allow(missing_docs)]


use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::ToTokens;
use quote::quote;

use super::body::Body;


pub(crate) struct ItemFn {
    pub(crate) outer_attrs: Vec<syn::Attribute>,
    pub(crate) vis: syn::Visibility,
    pub(crate) sig: syn::Signature,
    pub(crate) brace_token: syn::token::Brace,
    pub(crate) inner_attrs: Vec<syn::Attribute>,
    pub(crate) stmts: Vec<TokenStream>,
}


impl Clone for ItemFn {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            outer_attrs: self.outer_attrs.clone(),
            vis: self.vis.clone(),
            sig: self.sig.clone(),
            brace_token: self.brace_token,
            inner_attrs: self.inner_attrs.clone(),
            stmts: self.stmts.clone(),
        }
    }
}


// minimal ToString impl. since we can't derive it...
impl ToString for ItemFn {
    #[inline]
    fn to_string(&self) -> String {
        "TODO".to_string()
    }
}


impl ItemFn {
    /// Get the body of the function item in a manner so that it can be
    /// conveniently used with the `quote!` macro.
    pub(crate) fn body(&self) -> Body<'_> {
        Body {
            brace_token: self.brace_token,
            stmts: &self.stmts,
        }
    }

    pub(crate) fn to_string(self) -> String {
        let header: TokenStream = quote! {};

        let body: Body<'_> = self.body();
        let body: TokenStream = quote! {
            let body = async #body;
        };

        let last_block: TokenStream = TokenStream::new();

        self.into_tokens(header, body, last_block).to_string()
    }

    /// Convert our local function item into a token stream.
    pub(crate) fn into_tokens(
        self,
        header: TokenStream,
        body: TokenStream,
        last_block: TokenStream,
    ) -> TokenStream {
        let mut tokens = TokenStream::new();
        header.to_tokens(&mut tokens);

        // Outer attributes are simply streamed as-is.
        for attr in self.outer_attrs {
            // strip the `plugin` attribute itself so the expanded output
            // does not contain the attribute again
            // Skip any form of `plugin` attribute, including qualified paths
            if attr.path().segments.iter().any(|seg| seg.ident == "plugin") {
                continue;
            }
            attr.to_tokens(&mut tokens);
        }

        // Inner attributes require extra care, since they're not supported on
        // blocks (which is what we're expanded into) we instead lift them
        // outside of the function. This matches the behavior of `syn`.
        for mut attr in self.inner_attrs {
            attr.style = syn::AttrStyle::Outer;
            attr.to_tokens(&mut tokens);
        }

        self.vis.to_tokens(&mut tokens);
        self.sig.to_tokens(&mut tokens);

        self.brace_token.surround(&mut tokens, |tokens| {
            body.to_tokens(tokens);
            last_block.to_tokens(tokens);
        });

        tokens
    }
}


impl syn::parse::Parse for ItemFn {

    // TODO: read up on
    // https://docs.rs/syn/latest/syn/struct.Attribute.html
    // and check if tokio attribute is present
    #[inline]
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        // This parse implementation has been largely lifted from `syn`, with
        // the exception of:
        // * We don't have access to the plumbing necessary to parse inner
        //   attributes in-place.
        // * We do our own Configurationments parsing to avoid recursively parsing
        //   entire Configurationments and only look for the parts we're interested in.

        let outer_attrs = input.call(syn::Attribute::parse_outer)?;
        let vis: syn::Visibility = input.parse()?;
        let sig: syn::Signature = input.parse()?;

        let content;
        let brace_token = syn::braced!(content in input);
        let inner_attrs = syn::Attribute::parse_inner(&content)?;

        let mut buf = TokenStream::new();
        let mut stmts = Vec::new();

        while !content.is_empty() {
            if let Some(semi) = content.parse::<Option<syn::Token![;]>>()? {
                semi.to_tokens(&mut buf);
                stmts.push(buf);
                buf = TokenStream::new();
                continue;
            }

            // Parse a single token tree and extend our current buffer with it.
            // This avoids parsing the entire content of the sub-tree.
            buf.extend([content.parse::<TokenTree>()?]);
        }

        if !buf.is_empty() {
            stmts.push(buf);
        }

        Ok(Self {
            outer_attrs,
            vis,
            sig,
            brace_token,
            inner_attrs,
            stmts,
        })
    }
}

