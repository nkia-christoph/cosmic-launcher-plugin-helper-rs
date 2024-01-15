// Attribution: heavily inspired by / copied from tokio-macros::entry


#![deny(missing_docs)]


extern crate proc_macro;


use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use quote::quote_spanned;
use syn;


// syn::AttributeArgs does not implement syn::Parse
type AttributeArgs = syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>;


/// tokio_macros::entry::Body
struct Body<'a> {
    pub brace_token: syn::token::Brace,
    // Statements, with terminating `;`. // public so we can insert our code
    pub stmts: &'a [proc_macro2::TokenStream],
}

/// tokio_macros::entry::Body::ToTokens
impl ToTokens for Body<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.brace_token.surround(tokens, |tokens| {
            for stmt in self.stmts {
                stmt.to_tokens(tokens);
            }
        });
    }
}


/// tokio_macros::entry::ItemFn
/// https://github.com/tokio-rs/tokio/blob/master/tokio-macros/src/entry.rs#L469
struct ItemFn {
    outer_attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    sig: Signature,
    brace_token: syn::token::Brace,
    inner_attrs: Vec<syn::Attribute>,
    // instead of implementing the IsEmpty trait, we just check if the vec is empty
    // a functional plugin main function should at least contain `plugin.run();`
    pub stmts: Vec<proc_macro2::TokenStream>,
}

impl Default for ItemFn {
    #[inline]
    fn default() -> Self {
        Self {
            outer_attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            sig: Signature::default(),
            brace_token: syn::token::Brace::default(),
            inner_attrs: Vec::new(),
            stmts: Vec::new(),
        }
    }
}

// minimal ToString impl. since we can't derive it
impl std::str::ToString for ItemFn {
    #[inline]
    fn to_string(self) {
        format!(
            "{}{}{}{}{}{}",
            outer_attrs.map( |t| t.to_string()).join(" "),
            vis.to_string(),
            sig.to_string(),
            brace_token.to_string(),
            inner_attrs.map( |t| t.to_string()).join(" "),
            stmts.map( |t| t.to_string()).join(" "),
        )
    }
}

/// tokio_macros::entry::ItemFn
impl ItemFn {
    /// Access all attributes of the function item.
    fn attrs(&self) -> impl Iterator<Item = &Attribute> {
        self.outer_attrs.iter().chain(self.inner_attrs.iter())
    }

    /// Get the body of the function item in a manner so that it can be
    /// conveniently used with the `quote!` macro.
    fn body(&self) -> Body<'_> {
        Body {
            brace_token: self.brace_token,
            stmts: &self.stmts,
        }
    }

    /// Convert our local function item into a token stream.
    fn into_tokens(
        self,
        header: proc_macro2::TokenStream,
        body: proc_macro2::TokenStream,
        last_block: proc_macro2::TokenStream,
    ) -> TokenStream {
        let mut tokens = proc_macro2::TokenStream::new();
        header.to_tokens(&mut tokens);

        // Outer attributes are simply streamed as-is.
        for attr in self.outer_attrs {
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

/// tokio_macros::entry::ItemFn
/// https://github.com/tokio-rs/tokio/blob/master/tokio-macros/src/entry.rs#L478
impl syn::Parse for ItemFn {

    // TODO: read up on
    // https://docs.rs/syn/latest/syn/struct.Attribute.html
    // and check if tokio attribute is present
    #[inline]
    fn parse(input: syn::ParseStream<'_>) -> syn::Result<Self> {
        // This parse implementation has been largely lifted from `syn`, with
        // the exception of:
        // * We don't have access to the plumbing necessary to parse inner
        //   attributes in-place.
        // * We do our own statements parsing to avoid recursively parsing
        //   entire statements and only look for the parts we're interested in.

        let outer_attrs = input.call(syn::Attribute::parse_outer)?;
        let vis: syn::Visibility = input.parse()?;
        let sig: syn::Signature = input.parse()?;

        let content;
        let brace_token = syn::braced!(content in input);
        let inner_attrs = syn::Attribute::parse_inner(&content)?;

        let mut buf = proc_macro2::TokenStream::new();
        let mut stmts = Vec::new();

        while !content.is_empty() {
            if let Some(semi) = content.parse::<Option<syn::Token![;]>>()? {
                semi.to_tokens(&mut buf);
                stmts.push(buf);
                buf = proc_macro2::TokenStream::new();
                continue;
            }

            // Parse a single token tree and extend our current buffer with it.
            // This avoids parsing the entire content of the sub-tree.
            buf.extend([content.parse::<proc_macro2::TokenTree>()?]);
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


struct Configuration {
    pub name: Option<String>,
    pub log: bool,
    pub display: bool,
    pub parsed: ItemFn,
    pub insertion: proc_macro2::TokenStream,
    pub prepended: ItemFn,
    pub converted: ItemFn,
}

impl New for Configuration {
    #[inline]
    fn new() -> Self {
        Self {
            name: None,
            log: false,
            display: false,
            parsed: ItemFn::default(),
            insertion: proc_macro2::TokenStream::new(),
            prepended: ItemFn::default(),
            converted: ItemFn::default(),
        }
    }
}

/// minimal iterator impl. since we can't derive it
impl IntoConfigIterator for Configuration {
    type Item = String;
    type IntoIter = std::array::IntoIter<Item, 4>;

    #[inline]
    fn into_config_iter(&self) -> Self::IntoIter {
        std::array::IntoIter::new([
            format!("### name\n
                # plugin name: {}",
                self.name.to_string()),
            format!("### log\n
                # logging enabled:",
                self.log.to_string()),
            format!("### display\n
                # display of errors enabled:",
                self.display.to_string()),
            format!("### parsed\n
                # parsed input fn:",
                "aborted" if self.parsed.stmts.is_empty()
                else self.parsed.to_string()),
            format!("### insertion\n
                # generated insertion:",
                "aborted" if self.insertion.is_empty()
                else self.insertion.to_string()),
            format!("### prepended\n
                # result after we prepended the insertion in fn body:\n",
                "aborted" if self.prepended.is_empty()
                else self.prepended.to_string()),
            format!("### converted\n
                # converted fn to tokio runtime of flavour `current-thread`:\n",
                "aborted" if self.converted.is_empty()
                else self.converted.to_string()),
        ])
    }
}

impl std::str::ToString for Configuration {
    #[inline]
    fn to_string(&self) {
        self.into_config_iter().join("\n")
    }
}

impl Parse for Configuration {
    #[inline]
    fn parse(input: &proc_macro::TokenStream) -> syn::Result<(), syn::Error> {
        self.parsed = syn::parse2(&input)?;
        Ok(())
    }
}

/// tokio-macros::entry::main
/// https://github.com/tokio-rs/tokio/blob/master/tokio-macros/src/entry.rs#L422
impl FromAttrs for Configuration {
    #[inline]
    fn from_attrs(args: &AttributeArgs,) -> Result<Self, syn::Error> {
        println!("parsing attrs: {}", attrs.to_string());

        let mut config = Self::new();
        for arg in args {
            match arg {
                syn::Meta::NameValue(namevalue) => {
                    let ident = namevalue
                        .path
                        .get_ident()
                        .ok_or_else(|| {
                            syn::Error::new_spanned(&namevalue, "Must have specified ident")
                        })?
                        .to_string()
                        .to_lowercase();
                    let lit = match &namevalue.value {
                        syn::Expr::Lit(syn::ExprLit { lit, .. }) => lit,
                        expr => return Err(syn::Error::new_spanned(expr, "Must be a literal")),
                    };
                    match ident.as_str() {
                        "name" => {
                            config.name = match lit {
                                syn::Lit::Str(litstr) => config.name = Some(litstr.value()),
                                _ => return Err(syn::Error::new_spanned(lit,
                                    "Must be a string literal")),
                            }
                        }
                        "log" => config.log = true,
                        "display" => config.display = true,
                        arg => return Err(syn::Error::new_spanned(arg,
                            "Unknown attribute specified; expected one of: \
                            `log`, `display`, `name='My Plugin'`")),
                    }
                },
                other => return Err(syn::Error::new_spanned(other,
                    "Unknown attribute specified; expected one of: \
                    `log`, `display`, `name='My Plugin'`")),
            }
        }
        Ok(config)
    }
}

/// tokio_macros::entry::parse_knobs
/// https://github.com/tokio-rs/tokio/blob/master/tokio-macros/src/entry.rs#L327
impl Configuration {
    /// Check if the function is main.
    /// Yeet compile error if not.
    #[inline]
    fn ensure_main(&self) -> Result<(), syn::Error> {
        if self.parsed.sig.ident != "main" {
            let msg = "Plugin attribute can only be applied on main function.";
            let error = syn::Error::new_spanned(&input.sig.ident, msg);
            return Err(error.into_compile_error());
        }
        Ok(())
    }

    ///
    #[inline]
    fn create_insertion(&self) -> Result<(), syn::Error> {
        println!("creating code insertion");

        let name = match self.name.as_ref() {
            Some(name) => quote!(#name),
            None => proc_macro2::TokenStream::new(),
        };

        let display = if self.display {
            quote!(insert_display(vec![name]))
        } else {
            proc_macro2::TokenStream::new()
        };

        let log = if self.log {
            quote!(insert_logger(&display))
        } else {
            proc_macro2::TokenStream::new()
        };

        todo!();
        let insertion = quote!(
            #log
            #display
        );

        println!("generated code: {}", insertion.to_string());
        self.insertion = insertion;
        Ok(())
    }

    #[inline]
    fn prepend_fn_body(&self) -> Result<(), syn::Error> {
        println!("prepending generated code to input function body");

        let Body { brace_token, stmnts } = self.parsed.body();
        let output = stmnts.insert(0, self.insertion.clone())?;

        println!("code after insertion: {}", stmnts.to_string());
        self.prepended = Body {
            brace_token, stmnts };
        Ok()
    }

    #[inline]
    fn convert(&self) -> Result<(), syn::Error> {
        println!("converting to tokio runtime");
        let mut input = self.parsed.clone();
        input.sig.asyncness = None;

        // If type mismatch occurs, the current rustc points to the last statement.
        let (last_stmt_start_span, last_stmt_end_span) = {
            let mut last_stmt = input.stmts.last().cloned().unwrap_or_default().into_iter();

            // `Span` on stable Rust has a limitation that only points to the first
            // token, not the whole tokens. We can work around this limitation by
            // using the first/last span of the tokens like
            // `syn::Error::new_spanned` does.
            let start = last_stmt.next().map_or_else(proc_macro2::Span::call_site, |t| t.span());
            let end = last_stmt.last().map_or(start, |t| t.span());
            (start, end)
        };

        let mut rt = quote_spanned! {last_stmt_start_span=>
            tokio::runtime::Builder::new_current_thread()
        };

        // add name to thread
        if let Some(name) = config.name {
            rt = quote_spanned! {
                last_stmt_start_span=> #rt.thread_name(
                    "Pop!_OS Launcher plugin: #name"
            )};
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
        let output = input.into_tokens(header, body, last_block);

        println!("converted code to tokio runtime: {}", input.to_string());
        self.converted = output;
        Ok(())
    }

    #[inline]
    fn insert_logger(&self, display: &proc_macro2::TokenStream)
        -> proc_macro2::TokenStream
    {
        return quote!(
            std::panic::set_hook(
                Box::new(|why: &std::panic::PanicInfo<'_>| {
                    use pop_launcher_toolkit::plugin_trait::tracing;
                    let msg = format!("PLUGIN PANIC!\n{why:#?}");
                    tracing::error!(msg);
                    #display
                }
            ));
        );
    }

    #[inline]
    fn insert_display(&self, name: Vec<proc_macro2::TokenStream>) -> proc_macro2::TokenStream {
        return quote!(
            match tokio::runtime::Builder::new_current_thread()
                .thread_name("Panicking Pop!_OS Launcher plugin
                    #(
                        #name
                    )*
                ")
                .enable_all()
                .build()
            {
                Ok(rt) => rt.block_on( async {
                    helper::display_err!("Panic!", msg);
                    helper::finished!();
                }),
                Err(why) => tracing::error!(
                    "Failed to create tokio runtime \
                    to send panic to launcher
                    #( for plugin `#name` )*
                    : {:#?}", why
                ),
            };
        )
    }
}


/// #[plugin]
/// #[plugin(log)]
/// #[plugin(display)]
/// #[plugin(name="my_plugin")]
/// #[plugin(log, display)]
/// #[plugin(log, display, name="my_plugin")]
///
/// Rust Forum https://users.rust-lang.org/t/how-to-add-a-function-to-a-module-with-a-macro-solved/94004/4
#[proc_macro_attribute]
pub fn plugin(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {

    let config = crate::Configuration::from_attrs(&attrs)
        .unwrap_or_else(|err|
            return token_stream_with_error(Some(&config), &attrs, err));

    // https://github.com/tokio-rs/tokio/blob/12ce924fb9c1ffe0340b979fefa00d13ebf631c3/tokio-macros/src/entry.rs#L426C1-L429C7
    config.parse(&input)
        .unwrap_or_else(|err|
            return token_stream_with_error(Some(&config), item, err));

    config.ensure_main()
        .unwrap_or_else(|err|
            return token_stream_with_error(Some(&config), &attrs, err));

    config.create_insertion()
        .unwrap_or_else(|err|
            return token_stream_with_error(Some(&config), parsed, err));

    config.prepend_fn_body()
        .unwrap_or_else(|err|
            return token_stream_with_error(Some(&config), input, err));

    config.convert()
        .unwrap_or_else(|err|
            return token_stream_with_error(Some(&config), prepended, err));

    proc_macro::TokenStream::from(config.converted)
}

// customized from tokio-macros::entry::token_stream_with_error
// If any of the steps for this macro fail, we still want to expand to an item that is as close
// to the expected output as possible. This helps out IDEs such that completions and other
// related features keep working.
fn token_stream_with_error(
    config: Option<&Configuration>,
    mut tokens: TokenStream,
    error: syn::Error
) -> TokenStream {
    match config {
        Some(config) => {
            error = error.combine(
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    config.to_string(),
            ));
        },
        None => {},
    }
    tokens.extend(error.combine(custom_err)).into_compile_error();
    tokens
}


#[test]
fn main_plugin() {
    let tokens = quote! {
        #[plugin]
        async fn main() {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    };
    println!("{}", tokens.to_string());
    let expected = quote! {
        #[plugin]
        async fn main() {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed building the Runtime")
                .block_on(async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                })
        }
    };
    println!("{}", expected.to_string());

    assert_eq!(expected.to_string(), tokens.to_string());
}


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
