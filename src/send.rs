#![cfg(feature = "send")]


pub use crate::send;


/// **send a plugin response to launcher**
///
/// *using `async_stdout()` exported by the `pop_launcher_toolkit`*
///
/// ## Examples
/// ### Send a `PluginSearchResult`
/// ```rust
/// let result = PluginSearchResult { ..Default::default() };
/// send!(result);
/// send!(PluginSearchResult { ..Default::default() });
/// ```
/// ### Send any `PluginResponse`
/// ```rust
/// use pop_launcher_toolkit::launcher::PluginResponse;
/// send!(PluginResponse::*);
/// ```
/// ### Send an error
/// It will be displayed in launcher (and logged)
/// ```rust
/// send!(some_error_title);
/// send!(some_error_title, some_error_description);
/// send!("Error Title!");
/// send!("Error Title!", "Error description");
/// ```
/// sends:
/// ```
/// PluginSearchResult {
///     id: 0 as Indice,
///     name: "Error!".to_owned(),
///     description: "Error description".to_owned(),
///     icon: Some(IconSource::Name(Cow::Borrowed(
///             "dialog-error"))),
///     ..Default::default()
/// }
/// ```
#[macro_export]
macro_rules! send {
    ( PluginResponse::$r:ident $(,)? ) => {
        {
            use pop_launcher_toolkit::plugins;
            use pop_launcher_toolkit::launcher;
            plugins::send(
                &mut launcher::async_stdout(),
                $crate::any!(PluginResponse::$r)
            ).await
        }
    };
    ( PluginSearchResult $fields:tt $(,)? ) => {
        {
            use pop_launcher_toolkit::plugins;
            use pop_launcher_toolkit::launcher;
            plugins::send(
                &mut launcher::async_stdout(),
                $crate::any!(PluginSearchResult $fields)
            ).await
        }
    };
    ( $result:ident $(,)? ) => {
        {
            use pop_launcher_toolkit::plugins;
            use pop_launcher_toolkit::launcher;
            plugins::send(
                &mut launcher::async_stdout(),
                $crate::any!($result)
            ).await
        }
    };
    ( $error_name:expr $(, $error_description:expr )? $(,)? ) => {
        {
            use pop_launcher_toolkit::plugins;
            use pop_launcher_toolkit::launcher;
            use pop_launcher_toolkit::launcher::IconSource;
            use std::borrow::Cow;

            plugins::send(
                &mut launcher::async_stdout(),
                launcher::PluginResponse::Append(
                    launcher::PluginSearchResult {
                        id: 0 as launcher::Indice,
                        name: $error_name.to_owned(),
                        $(
                            description: $error_description.to_owned(),
                        )?
                        icon: Some(IconSource::Name(Cow::Borrowed("dialog-error"))),
                        ..Default::default()
                    }
                )
            ).await;
        }
    };
    ( $( $(,)? $n:expr)* $(,)? ) => {
        compile_error!("invalid type supplied to send! macro!")
    };
}


/// create ANY `PluginResponse`
/// (with `PluginSearchResult`)
/// ready to send to launcher
///
/// ## Examples
/// ### Send a `PluginSearchResult`
/// ```rust
/// let result = PluginSearchResult { ..Default::default() };
/// send!(result);
/// send!(PluginSearchResult { ..Default::default() });
/// ```
/// ### Send any `PluginResponse`
/// ```rust
/// use pop_launcher_toolkit::launcher::PluginResponse;
/// send!(PluginResponse::*);
/// ```
/// ### Send an error
/// It will be displayed in launcher (and logged)
/// ```rust
/// send!(some_error_title);
/// send!(some_error_title, some_error_description);
/// send!("Error Title!");
/// send!("Error Title!", "Error description");
/// ```
/// sends:
/// ```
/// PluginSearchResult {
///     id: 0 as Indice,
///     name: "Error!".to_owned(),
///     description: "Error description".to_owned(),
///     icon: Some(IconSource::Name(Cow::Borrowed(
///             "dialog-error"))),
///     ..Default::default()
/// }
/// ```
#[macro_export]
macro_rules! any {
    ( PluginResponse::$any:tt $(,)? ) => {
        pop_launcher_toolkit::launcher::PluginResponse::$any
    };
    ( PluginSearchResult $fields:tt $(,)? ) => {
        {
            use pop_launcher_toolkit::plugins;
            use pop_launcher_toolkit::launcher;
            launcher::PluginResponse::Append(
                launcher::PluginSearchResult $fields
            )
        }
    };
    ( $result:ident $(,)? ) => {
        pop_launcher_toolkit::launcher::PluginResponse::Append(
            $result
        )
    };
    ( $( $(,)? $n:expr)* $(,)? ) => {
        compile_error!("invalid type supplied to send! macro!")
    };
}
