#![cfg(feature = "panic")]


pub use crate::panic;


/// impl logging to standard launcher logs
/// and optionally send panic message to
/// launcher on panics and aborts.
#[macro_export]
macro_rules! set_hook {
    ( $( $send:expr )? ) => {
        std::panic::set_hook(
            Box::new(|why: &std::panic::PanicInfo<'_>| {
                use pop_launcher_toolkit::plugin_trait::tracing;

                let msg = format!("PLUGIN PANIC!\n{why:#?}");
                tracing::error!(msg);
                $(
                    // bs assignment to conditionally send message
                    // to launcher on panics and aborts.
                    let _ = $send;
                    match tokio::runtime::Builder::new_current_thread()
                        .thread_name("panicking Pop!_OS Launcher plugin")
                        .enable_all()
                        .build()
                    {
                        Ok(rt) => rt.block_on( async {
                            $crate::send!("Panic!", msg);
                            $crate::send!(PluginResponse::Finished);
                        }),
                        Err(why) => {
                            tracing::error!("failed to create tokio runtime: {:#?}", why);
                        },
                    };
                )?
            }
        ));
    };
}


#[macro_export]
macro_rules! log {
    (
        $( #[$attrs:meta] )*
        $pub:vis $( $async:ident )?
        fn main $( < $($gen:tt),* > )? ( $($arg:tt)* ) $( -> $ret:ty )? $body:block
    ) => (
        #[allow(dead_code)]
        $( #[$attrs] )*
        $pub $( $async )?
        fn main $( < $($gen),* > )? ( $($arg)* ) $( -> $ret )? {
            set_hook!();
            $body
        }
    );
    (
        $( #[$attrs:meta] )*
        $pub:vis $( $async:ident )?
        fn $NAME:ident $( < $($gen:tt),* > )? ( $($arg:tt)* ) $( -> $ret:ty )? $body:block
    ) => (compile_error!("function needs to be main!"));
}


#[macro_export]
macro_rules! log_and_display {
    (
        $( #[$attrs:meta] )*
        $pub:vis $( $async:ident )?
        fn main $( < $($gen:tt),* > )? ( $($arg:tt)* ) $( -> $ret:ty )? $body:block
    ) => (
        $( #[$attrs] )*
        #[allow(dead_code)]
        $pub $( $async )?
        fn main $( < $($gen),* > )? ( $($arg)* ) $( -> $ret )? {
            set_hook!(0);
            $body
        }
    );
    (
        $( #[$attrs:meta] )*
        $pub:vis $( $async:ident )?
        fn $NAME:ident $( < $($gen:tt),* > )? ( $($arg:tt)* ) $( -> $ret:ty )? $body:block
    ) => (compile_error!("function needs to be main!"));
}


#[allow(unused_macros)]
macro_rules! plugin {
    (
        $( #[$attrs:meta] )*
        $pub:vis async
        fn main $( < $($gen:tt),* > )? ( $($arg:tt)* ) $( -> $ret:ty )? $body:block
    ) => (
        #[allow(dead_code)]
        $( #[$attrs] )*
        #[tokio::main(flavor = "current_thread")]
        $pub async
        fn main $( < $($gen),* > )? ( $($arg)* ) $( -> $ret )? {
            set_hook!();
            $body
        }
    );
    (
        $( #[$attrs:meta] )*
        #[ $( tokio:: )? main $( $($tokio:tt)+ )? ]
        $( #[$bttrs:meta] )*
        $pub:vis async
        fn $NAME:ident $( < $($gen:tt),* > )? ( $($arg:tt)* ) $( -> $ret:ty )? $body:block
    ) => (compile_error!(
        "remove attribute: `#[tokio::main(flavor = \"current_thread\")]` from main.\n\
        `#[apply(plugin)]` applies `#[tokio::main(flavor = \"current_thread\")]` for you!"
    ));
    (
        $( #[$attrs:meta] )*
        #[ $( tokio:: )? main $( $($tokio:tt)+ )? ]
        $( #[$bttrs:meta] )*
        $pub:vis
        fn $NAME:ident $( < $($gen:tt),* > )? ( $($arg:tt)* ) $( -> $ret:ty )? $body:block
    ) => (compile_error!("main needs to be async!"));
}


#[allow(unused_macros)]
macro_rules! plugin_display_errors {
    (
        $( #[$attrs:meta] )*
        $pub:vis async
        fn main $( < $($gen:tt),* > )? ( $($arg:tt)* ) $( -> $ret:ty )? $body:block
    ) => (
        #[allow(dead_code)]
        $( #[$attrs] )*
        #[tokio::main(flavor = "current_thread")]
        $pub async
        fn main $( < $($gen),* > )? ( $($arg)* ) $( -> $ret )? {
            set_hook!(0);
            $body
        }
    );
    (
        $( #[$attrs:meta] )*
        #[ $( tokio:: )? main $( $($tokio:tt)+ )? ]
        $( #[$bttrs:meta] )*
        $pub:vis async
        fn $NAME:ident $( < $($gen:tt),* > )? ( $($arg:tt)* ) $( -> $ret:ty )? $body:block
    ) => (compile_error!(
        "remove attribute: `#[tokio::main(flavor = \"current_thread\")]` from main.\n\
        `#[apply(plugin)]` applies `#[tokio::main(flavor = \"current_thread\")]` for you!"
    ));
    (
        $( #[$attrs:meta] )*
        #[ $( tokio:: )? main $( $($tokio:tt)+ )? ]
        $( #[$bttrs:meta] )*
        $pub:vis
        fn $NAME:ident $( < $($gen:tt),* > )? ( $($arg:tt)* ) $( -> $ret:ty )? $body:block
    ) => (compile_error!("main needs to be async!"));
}


/// test
#[test]
fn log_panic() {
    #[apply(log)]
    async fn main() {
        println!("success!");
        panic!("test panic!");
    }
}


/// test
#[test]
fn log_panic_and_display() {
    #[apply(log_and_display)]
    async fn main() {
        println!("success!");
        panic!("test panic!");
    }
}


/// test
#[test]
fn tokio_log_panic() {
    #[apply(log)]
    #[tokio::main(flavor = "current_thread")]
    async fn main() {
        println!("success!");
        panic!("test panic!");
    }
}


/// test
#[test]
fn tokio_log_panic_and_display() {
    #[apply(log_and_display)]
    #[tokio::main(flavor = "current_thread")]
    async fn main() {
        println!("success!");
        panic!("test panic!");
    }
}


/// test
#[test]
fn tokio_log_panic_plugin() {
    #[apply(plugin)]
    async fn main() {
        println!("success!");
        panic!("test panic!");
    }
}


/// test
#[test]
fn tokio_log_panic_plugin_display() {
    #[apply(plugin_display_errors)]
    async fn main() {
        println!("success!");
        panic!("test panic!");
    }
}
