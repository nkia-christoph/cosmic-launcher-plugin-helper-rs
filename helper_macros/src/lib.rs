// Attribution: tokio-rs/tokio-macros::entry


#![allow(missing_docs)]
#![allow(clippy::needless_return)]


mod body;
mod config;
mod item_fn;


use ::proc_macro;


/// convert proc_macro::TokenStream to proc_macro"::TokenStream and vice versa
#[proc_macro_attribute]
pub fn plugin(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    self::plugin_private(args.into(), item.into()).into()
}


use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use syn::parse::Parser;

use crate::body::Body;
use crate::config::Configuration;
use crate::item_fn::ItemFn;


// syn::AttributeArgs does not implement syn::Parse


fn plugin_private(
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream {

    let mut config = match crate::Configuration::from_attrs(&attrs) {
        Ok(config) => config,
        Err(err) => {
            return token_stream_with_error(None, attrs, err)
        },
    };

    // https://github.com/tokio-rs/tokio/blob/12ce924fb9c1ffe0340b979fefa00d13ebf631c3/tokio-macros/src/entry.rs#L426C1-L429C7
    // need empty assignments because compiler doesn't know that
    // token_stream_with_error halts execution
    match config.parse(&input) {
        Ok(_) => eprintln!("parsed input function"),
        Err(why) => {
            return token_stream_with_error(Some(config), input, why);
        },
    };

    match config.ensure_main() {
        Ok(_) => eprintln!("ensured input function is main"),
        Err(why) => {
            return token_stream_with_error(Some(config), input, why);
        },
    };

    match config.create_insertion() {
        Ok(_) => eprintln!("created code insertion"),
        Err(why) => {
            return token_stream_with_error(Some(config), input, why);
        },
    };

    match config.prepend_fn_body() {
        Ok(_) => eprintln!("prepended generated code to input function body"),
        Err(why) => {
            return token_stream_with_error(Some(config), input, why);
        },
    };

    match config.convert()  {
        Ok(_) => eprintln!("converted code to tokio runtime"),
        Err(why) => {
            return token_stream_with_error(Some(config), input, why);
        },
    };

    config.converted.unwrap()
}


/// tokio_macros::entry::parse_knobs
/// https://github.com/tokio-rs/tokio/blob/master/tokio-macros/src/entry.rs#L327
impl Configuration {

    #[inline]
    fn convert(&mut self) -> Result<(), syn::Error> {
        eprintln!("converting to tokio runtime");
        let mut input = self.parsed.clone()?;
        input.sig.asyncness = None;

        // If type mismatch occurs, the current rustc points to the last Configurationment.
        let (last_stmt_start_span, last_stmt_end_span) = {
            let mut last_stmt = input.stmts.last().cloned().unwrap_or_default().into_iter();

            // `Span` on stable Rust has a limitation that only points to the first
            // token, not the whole tokens. We can work around this limitation by
            // using the first/last span of the tokens like
            // `syn::Error::new_spanned` does.
            let start = last_stmt.next().map_or_else(Span::call_site, |t| t.span());
            let end = last_stmt.last().map_or(start, |t| t.span());
            (start, end)
        };

        let mut rt = quote_spanned! {last_stmt_start_span=>
            tokio::runtime::Builder::new_current_thread()
        };

        // add name to thread
        let mut thread_name: String = "Pop!_OS Launcher plugin".to_string();
        if let Some(name) = &self.name {
            thread_name = format!("{}: {}", thread_name, name);
            rt = quote_spanned! {
                last_stmt_start_span=> #rt.thread_name(#thread_name)
            };
        }

        let body_ident = quote! { body };
        let last_block = quote_spanned! {last_stmt_end_span=>
            #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
            {
                return #rt
                    .enable_all()
                    .build()
                    .expect("Failed building the Runtime")
                    .block_on(#body_ident);
            }
        };

        let body = input.body();
        let body = quote! {
            let body = async #body;
        };

        // ensure dependencies are available
        let header = quote! {
            #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
            use ::pop_launcher_toolkit::plugin_trait::tracing;
        };
        let output = input.into_tokens(header, body, last_block);

        eprintln!("converted code to tokio runtime: \n{:#?}", output);
        self.converted = Ok(output);
        Ok(())
    }

    #[inline]
    fn create_insertion(&mut self) -> Result<(), syn::Error> {
        eprintln!("creating code insertion");

        #[allow(unused_variables)]
        let display = if self.display {
            self.insert_display()
        } else {
            TokenStream::new()
        };

        let log = if self.log {
            self.insert_logger(&display)
        } else {
            TokenStream::new()
        };

        let insertion = quote!( #log );

        eprintln!("generated code: {:#?}", insertion);
        self.insertion = Ok(insertion);
        Ok(())
    }

    /// Check if the function is main.
    /// Yeet compile error if not.
    #[inline]
    fn ensure_main(&self) -> Result<(), syn::Error> {
        let parsed = self.parsed.clone()?;
        if parsed.sig.ident != "main" {
            return Err(syn::Error::new_spanned(
                // TODO: return whole function
                parsed.sig.ident,
                "Plugin attribute can only be applied on main function.",
            ));
        }
        Ok(())
    }

    #[inline]
    fn from_attrs(attrs: &TokenStream) -> Result<Self, syn::Error> {
        eprintln!("parsing attrs: {:#?}", attrs);
            let raw = attrs.to_string();
            let mut config = Self::new();
            if raw.contains("log") {
                config.log = true;
            }
            if raw.contains("display") {
                config.display = true;
            }
            // parse name = "..."
            if let Some(idx) = raw.find("name") {
                // simple parse: look for first '"' after '=' and the closing '"'
                if let Some(eq_idx) = raw[idx..].find('=') {
                    let after_eq = &raw[idx + eq_idx + 1..];
                    if let Some(start_quote) = after_eq.find('"') {
                        let after_quote = &after_eq[start_quote + 1..];
                        if let Some(end_quote) = after_quote.find('"') {
                            let name = &after_quote[..end_quote];
                            config.name = Some(name.to_string());
                        }
                    }
                }
            }
            Ok(config)
    }


    #[inline]
    fn prepend_fn_body(&mut self) -> Result<(), syn::Error> {
        eprintln!("prepending generated code to input function body");

        let binding = self.parsed.clone()?;
        let Body { brace_token, stmts } = binding.body();
        let output = &[
            self.insertion.clone()?,
            stmts.iter().cloned().collect::<TokenStream>(),
        ];

        eprintln!("code after insertion: {}", stmts.iter().cloned().fold(
            "".to_string(),
            |accumulator: String, stmt: TokenStream|
            format!("{}\n{:#?}", accumulator, stmt)
        ));

        let item: ItemFn = self.prepend_from_body(Body {
            brace_token,
            stmts: output,
        })?;
        self.prepended = Ok(item);
        Ok(())
    }

    /// Convert a Body into an ItemFn.
    #[inline]
    fn prepend_from_body(&self, body: Body) -> Result<ItemFn, syn::Error> {
        let parsed = self.parsed.clone()?;
        if parsed.stmts.is_empty() {
            return Err(syn::Error::new_spanned(
                parsed.body(),
                "config.prepend is empty?! This should not happen.",
            ));
        }
        let Body { brace_token, stmts } = body;
        Ok(ItemFn {
            outer_attrs: parsed.outer_attrs,
            vis: parsed.vis,
            sig: parsed.sig,
            brace_token,
            inner_attrs: parsed.inner_attrs,
            stmts: stmts.to_vec(),
        })
    }
}

// Rust Forum https://users.rust-lang.org/t/how-to-add-a-function-to-a-module-with-a-macro-solved/94004/4


/// customized from tokio-macros::entry::token_stream_with_error
/// If any of the steps for this macro fail, we still want to expand to an item that is as close
/// to the expected output as possible. This helps out IDEs such that completions and other
/// related features keep working.
fn token_stream_with_error(
    mut config: Option<Configuration>,
    mut tokens: TokenStream,
    mut error: syn::Error
) -> TokenStream {
    if let Some(config) = config.take() {
        error.combine(
            syn::Error::new(
                Span::call_site(),
                config.to_string(),
        ));
    };
    tokens.extend(error.into_compile_error());
    tokens
}


// #[test]
// fn main_plugin() {
//     let tokens = quote! {
//         #[plugin]
//         async fn main() {
//             tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
//         }
//     };
//     eprintln!("{:#?}", tokens);
//     let expected = quote! {
//         #[plugin]
//         async fn main() {
//             tokio::runtime::Builder::new_current_thread()
//                 .enable_all()
//                 .build()
//                 .expect("Failed building the Runtime")
//                 .block_on(async {
//                     tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
//                 })
//         }
//     };
//     eprintln!("{:#?}", expected);

//     assert_eq!(expected.to_string(), tokens.to_string());
// }


// more tests
// should pass
//
// #[plugin(log)]
// #[plugin(display)]
// #[plugin(name="my_plugin")]
// #[plugin(log, display)]
// #[plugin(display, name="my_plugin")]
// #[plugin(log, name="my_plugin")]
// #[plugin(log, display, name="my_plugin")]
//
// should fail
// #[plugin(log, display, name="my_plugin", "my_plugin")]
// #[plugin(log, display, name="my_plugin", my_plugin)]
//
//
// tokio integration tests
//
// #[plugin(log)]
// #[tokio::main(flavor = "current_thread")]
//
// #[plugin(display)]
// #[tokio::main(flavor = "current_thread")]
//
// #[plugin(name="my_plugin")]
// #[tokio::main(flavor = "current_thread")]
//
// #[plugin(log, display)]
// #[tokio::main(flavor = "current_thread")]
//
// #[plugin(display, name="my_plugin")]
// #[tokio::main(flavor = "current_thread")]
//
// #[plugin(log, name="my_plugin")]
// #[tokio::main(flavor = "current_thread")]
//
// #[plugin(log, display, name="my_plugin")]
// #[tokio::main(flavor = "current_thread")]
//
// #[plugin(log, display, name="my_plugin"]
// #[tokio::main(flavor = "current_thread")]
//
