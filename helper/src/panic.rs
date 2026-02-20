#![cfg(feature = "panic")]
#![warn(missing_docs)]


pub use crate::panic;


/// impl logging to standard launcher logs
/// and optionally send panic message to
/// launcher on panics and aborts.
///
///
#[macro_export]
macro_rules! install {
    ( $( $send:expr )? ) => (
        std::panic::set_hook(
            Box::new(|why: &std::panic::PanicInfo<'_>| {
                use pop_launcher_toolkit::plugin_trait::tracing;

                let msg = format!("PLUGIN PANIC!\n{why:#?}");
                tracing::error!(msg);
                $(
                    // bs assignment to conditionally send message
                    // to launcher on panics and aborts.
                    let _ = $send;
                    match ::tokio::runtime::Builder::new_current_thread()
                        .thread_name("panicking Pop!_OS Launcher plugin")
                        .enable_all()
                        .build()
                    {
                        Ok(rt) => rt.block_on( async {
                            $crate::display_err!("Panic!", msg);
                            $crate::finished!();
                        }),
                        Err(why) => {
                            tracing::error!("failed to create tokio runtime: {:#?}", why);
                        },
                    };
                )?
            }
        ));
    );
}


/// test
#[not_implemented]
#[test]
fn log_panic() {
    #[apply(log)]
    async fn main() {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
}


/// test
#[not_implemented]
#[test]
fn log_panic_and_display() {
    #[apply(log_and_display)]
    async fn main() {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
}


/// test
#[not_implemented]
#[test]
fn tokio_log_panic() {
    #[apply(log)]
    #[tokio::main(flavor = "current_thread")]
    async fn main() {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
}


/// test
#[not_implemented]
#[test]
fn tokio_log_panic_and_display() {
    #[apply(log_and_display)]
    #[tokio::main(flavor = "current_thread")]
    async fn main() {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
    main();
}


/// test
#[not_implemented]
#[test]
fn tokio_log_panic_plugin() {
    #[apply(plugin)]
    async fn main() {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
    main();
}


/// test
#[not_implemented]
#[test]
fn tokio_log_panic_plugin_display() {
    #[apply(plugin_display_errors)]
    async fn main() {
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
    main();
}
