// Attribution: tokio-rs/tokio-macros::entry


#![allow(missing_docs)]


use proc_macro2::TokenStream;
use quote::quote;

use super::item_fn::ItemFn;


/// holds Configuration of our macro
pub(crate) struct Configuration {
    pub(crate) name: Option<String>,
    pub(crate) log: bool,
    pub(crate) display: bool,
    pub(crate) parsed: Result<ItemFn, syn::Error>,
    pub(crate) insertion: Result<TokenStream, syn::Error>,
    pub(crate) prepended: Result<ItemFn, syn::Error>,
    pub(crate) converted: Result<TokenStream, syn::Error>,
}


/// custom iterator for error display
impl IntoIterator for Configuration {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;


    #[inline]
    fn into_iter(self) -> Self::IntoIter {

        let name: String = self.name.unwrap_or_default().to_string();
        let log: String = self.log.to_string();
        let display: String = self.display.to_string();

        let parsed: String = match self.parsed {
            Ok(item_fn) => item_fn.to_string(),
            Err(why) => why.to_string(),
        };

        let insertion: String = match self.insertion {
            Ok(token_stream) => token_stream.to_string(),
            Err(why) => why.to_string(),
        };

        let prepended: String = match self.prepended {
            Ok(item_fn) => item_fn.to_string(),
            Err(why) => why.to_string(),
        };

        let converted: String = match self.converted {
            Ok(token_stream) => token_stream.to_string(),
            Err(why) => why.to_string(),
        };

        vec![
            format!("\n### NAME \
                plugin name: {}",
                name),
            format!("\n### LOG \
                logging enabled: {}",
                log),
            format!("\n### DISPLAY \
                display of errors enabled: {}",
                display),
            format!("\n### PARSED \
                parsed input fn:\n{}",
                parsed),
            format!("\n### INSERTION \
                generated insertion:\n{}",
                insertion),
            format!("\n### PREPENDED \
                result after we prepended the insertion in fn body:\n{}",
                prepended),
            format!("\n### CONVERTED \
                converted fn to tokio runtime of flavour `current-thread`:\n{}",
                converted),
        ].into_iter()
    }
}


impl Configuration {
    #[allow(dead_code, unused_variables)]
    #[inline]
    pub(crate) fn insert_display(&self) -> TokenStream {
        let mut thread_name: String = "Panicking Pop!_OS Launcher plugin".to_string();
        let mut msg: String = "Failed to create tokio runtime to send panic to launcher".to_string();

        if let Some(name) = &self.name {
            thread_name = format!("{}: {}", thread_name, name);
            msg = format!("{} for plugin `{}`", msg, name);
        }

        return quote!(
            match tokio::runtime::Builder::new_current_thread()
                .thread_name(#thread_name)
                .enable_all()
                .build()
            {
                Ok(rt) => rt.block_on( async {
                    helper::display_err!("Panic!", msg);
                    helper::finished!();
                }),
                Err(why) => {
                    let msg: String = #msg;
                    let msg: String = format!("{}: {:#?}", msg, why);
                    tracing::error!(msg)
                },
            };
        )
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn insert_logger(&self, display: &TokenStream)
        -> TokenStream
    {
        return quote!(
            std::panic::set_hook(
                Box::new(|why: &std::panic::PanicInfo<'_>| {
                    use ::pop_launcher_toolkit::plugin_trait::tracing;
                    let msg = format!("PLUGIN PANIC!\n{why:#?}");
                    tracing::error!(msg);
                    #display
                }
            ));
        );
    }

    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            name: None,
            log: false,
            display: false,
            parsed: Err(syn::Error::new_spanned(
                TokenStream::new(),
                "Configuration.parsed has not been set yet.",
            )),
            insertion: Err(syn::Error::new_spanned(
                TokenStream::new(),
                "Configuration.insertion has not been set yet.",
            )),
            prepended: Err(syn::Error::new_spanned(
                TokenStream::new(),
                "Configuration.prepended has not been set yet.",
            )),
            converted: Err(syn::Error::new_spanned(
                TokenStream::new(),
                "Configuration.converted has not been set yet.",
            )),
        }
    }

    #[inline]
    pub(crate) fn parse(&mut self, input: &TokenStream) -> Result<(), syn::Error> {
        let output: ItemFn = syn::parse2(input.to_owned())?;
        self.parsed = Ok(output);
        Ok(())
    }

    #[inline]
    pub(crate) fn to_string(self) -> String {
        self.into_iter().fold(
            "".to_string(),
            |accumulator: String, string: String|
            format!("{}\n{}", accumulator, string)
        )
    }
}
