#![cfg(feature = "send")]
#![warn(missing_docs)]


pub use crate::send;
pub use crate::display_err;


/// **Append a search result.**
///
/// The usual response to `Request::Search` from the launcher.
/// It appends the provided `PluginSearchResult` to the launcher display.
///
/// ## Examples
/// ```rust
/// use pop_launcher_toolkit::launcher::PluginSearchResult;
/// use helper::append;
///
/// # tokio_test::block_on(async {
/// append!(PluginSearchResult { ..Default::default() });
/// let result = PluginSearchResult { ..Default::default() };
/// append!(result);
/// # });
/// ```
///
/// *uses `async_stdout()` exported by the `pop_launcher_toolkit`.*
#[macro_export]
macro_rules! append {
    ( $result:expr $(,)? ) => (
        pop_launcher_toolkit::plugins::send(
            &mut pop_launcher_toolkit::launcher::async_stdout(),
            pop_launcher_toolkit::launcher::PluginResponse::Append(
                $result
            )
        ).await
    );
}


/// **Clear launcher display.**
///
/// If the results your plugin has computed and sent to the launcher
/// are no longer valid, you can clear the launcher display.
///
/// Usually used when using multiple threads together with
/// the plugin's interrupt() function.
///
/// ## Examples
/// ```rust
/// use helper::clear;
///
/// # tokio_test::block_on(async {
/// clear!();
/// # });
/// ```
///
/// *uses `async_stdout()` exported by the `pop_launcher_toolkit`.*
#[macro_export]
macro_rules! clear {
    () => (
        pop_launcher_toolkit::plugins::send(
            &mut pop_launcher_toolkit::launcher::async_stdout(),
            pop_launcher_toolkit::launcher::PluginResponse::Clear
        ).await
    );
}


/// **Close launcher.**
///
/// This is the usual response to `Request::Activate` from the launcher,
/// **after** the selected `PluginSearchResult` has been activated/executed.
///
/// ## Examples
/// ```rust
/// use helper::close;
///
/// # tokio_test::block_on(async {
/// close!();
/// # });
/// ```
///
/// *uses `async_stdout()` exported by the `pop_launcher_toolkit`.*
#[macro_export]
macro_rules! close {
    () => (
        pop_launcher_toolkit::plugins::send(
            &mut pop_launcher_toolkit::launcher::async_stdout(),
            pop_launcher_toolkit::launcher::PluginResponse::Close
        ).await
    );
}


/// **Send a context menu.**
///
/// ## Examples
/// ```rust
/// # tokio_test::block_on(async {
/// use helper::context;
/// use pop_launcher_toolkit::launcher::Indice;
///
/// let search_result_id: Indice = 0;
///
/// context!(search_result_id, "context");
/// context!(0, ["context1", "context2"]);
///
/// let option: &str = "context";
/// context!(search_result_id, [option]);
/// context!(0, [option]);
///
/// let option: String = option.to_string();
/// # let option2: String = option.clone();
/// context!(0, [option]);
/// # let option: String = option;
/// context!(search_result_id, [option]);
/// # });
/// ```
///
/// *uses `async_stdout()` exported by the `pop_launcher_toolkit`.*
#[macro_export]
macro_rules! context {
    // received single option as arg
    // (String, Slice or Literal): `[option]`
    ( $id:expr, [ $name:ident ] $(,)? ) => (
        {
            let option = vec![ pop_launcher_toolkit::launcher::ContextOption {
                id: 0,
                name: $name.to_string(),
            } ];
            $crate::context!( @send
                $id,
                option,
            )
        }
    );
    // received single arg as vec
    // `option_vec`: <Vec<ContextOption>>
    ( $id:expr, $option_vec:ident $(,)? ) => (
        $crate::context!( @send
            $id,
            $option_vec,
        )
    );
    // received single arg as String Literal: `"option"``
    ( $id:expr, $option:literal $(,)? ) => (
        $crate::context!( @send
            $id,
            vec![ pop_launcher_toolkit::launcher::ContextOption {
                id: 0,
                name: $option.to_string(),
            } ]
        )
    );

    // received multiple options in bracets:
    // `[option_1, option_2,... option_n]``
    ( $id:expr, $options:tt $(,)? ) => (
        {
            use pop_launcher_toolkit::launcher::ContextOption;
            let options: Vec<ContextOption> = $crate::context!(
                @block_to_vec_options
                $options
            );
            $crate::context!( @send
                $id,
                options,
            )
        }
    );
    // convert String Literals to vec of `ContextOption`s
    ( @block_to_vec_options [
        $option_1:literal
        $( $(,)? $option_n:literal )*
    ] ) => (
        {
            use pop_launcher_toolkit::launcher::ContextOption;
            let mut id: pop_launcher_toolkit::launcher::Indice = 0;
            let mut options: Vec<ContextOption> = vec![ ContextOption {
                id,
                name: $option_1.to_string(),
            } ];
            $(
                id += 1;
                options.push( ContextOption {
                    id,
                    name: $option_n.to_string(),
                } );
            )*
            options
        }
    );

    // send (converted) `ContextOption` to launcher
    ( @send $id:expr, $vec_options:expr $(,)? ) => (
        pop_launcher_toolkit::plugins::send(
            &mut pop_launcher_toolkit::launcher::async_stdout(),
            pop_launcher_toolkit::launcher::PluginResponse::Context {
                id: $id,
                options: $vec_options,
            }
        ).await
    );
}


/// **Deactivate plugin.**
///
/// Deactivates the plugin until the launcher is restarted.
///
/// ## Examples
/// ```rust
/// use helper::deactivate;
///
/// # tokio_test::block_on(async {
/// deactivate!();
/// # });
/// ```
///
/// *uses `async_stdout()` exported by the `pop_launcher_toolkit`.*
#[macro_export]
macro_rules! deactivate {
    () => (
        pop_launcher_toolkit::plugins::send(
            &mut pop_launcher_toolkit::launcher::async_stdout(),
            pop_launcher_toolkit::launcher::PluginResponse::Deactivate
        ).await
    );
}


/// **Send a desktop entry.**
///
/// `GpuPreference::Default` is applied
/// if no 2nd argument is provided.
///
/// ## Examples
/// ```rust
/// # tokio_test::block_on(async {
/// use helper::desktop_entry;
///
/// desktop_entry!("/path/to/desktop/entry");
/// desktop_entry!("/p/t/desktop/e", Default);
/// desktop_entry!("/p/t/desktop/e", NonDefault);
///
/// use pop_launcher_toolkit::launcher::GpuPreference;
///
/// desktop_entry!("/p/t/desktop/e", GpuPreference::Default);
/// desktop_entry!("/p/t/desktop/e", GpuPreference::NonDefault);
///
/// let path: &str = "/path/to/desktop/entry";
/// desktop_entry!(path);
/// desktop_entry!(path, Default);
/// desktop_entry!(path, NonDefault);
/// desktop_entry!(path, GpuPreference::Default);
/// desktop_entry!(path, GpuPreference::NonDefault);
///
/// let path: String = path.to_string();
/// # let path2: String = path.clone();
/// # let path3: String = path2.clone();
/// # let path4: String = path3.clone();
/// # let path5: String = path4.clone();
/// desktop_entry!(path);
/// # let path = path2;
/// desktop_entry!(path, Default);
/// # let path = path3;
/// desktop_entry!(path, NonDefault);
/// # let path = path4;
/// desktop_entry!(path, GpuPreference::Default);
/// # let path = path5;
/// desktop_entry!(path, GpuPreference::NonDefault);
/// # });
/// ```
///
/// - *On Pop!_OS Launcher, this is when the user presses `Enter`.*
/// *uses `async_stdout()` exported by the `pop_launcher_toolkit`.*
#[macro_export]
macro_rules! desktop_entry {
    // shorthands
    ( $path:expr, Default ) => (
        $crate::desktop_entry!( $path, GpuPreference::Default )
    );
    ( $path:expr, NonDefault ) => (
        $crate::desktop_entry!( $path, GpuPreference::NonDefault )
    );

    // process args
    ( $path:expr ) => (
        {
            let path_buf = std::path::PathBuf::from( $path );
            $crate::desktop_entry!( @send
                path_buf,
                pop_launcher_toolkit::launcher::GpuPreference::Default
            )
        }
    );
    ( $path:expr, $gpu_preference:expr $(,)? ) => (
        {
            let path_buf = std::path::PathBuf::from( $path );
            $crate::desktop_entry!( @send
                path_buf,
                $gpu_preference,
            )
        }
    );

    // send desktop entry to launcher
    ( @send $path:expr, $gpu_preference:expr $(,)? ) => (
        pop_launcher_toolkit::plugins::send(
            &mut pop_launcher_toolkit::launcher::async_stdout(),
            pop_launcher_toolkit::launcher::PluginResponse::DesktopEntry {
                path: $path,
                gpu_preference: $gpu_preference,
                action_name: None,
            }
        ).await
    );

    ( $( $wrong:tt )? ) => (
        compile_error!("desktop_entry! macro requires a String, &str \
                        or String Literal as an argument)");
    );
}


/// **Fill launcher search bar.**
///
/// The usual response to `Request::Complete`.
/// It replaces the current search bar text with the provided string.
///
/// ## Examples
/// ```rust
/// # use helper::fill;
/// # tokio_test::block_on(async {
/// fill!("autocomplete");
/// let autocomplete: &str = "autocomplete";
/// fill!(autocomplete);
/// let autocomplete: String = autocomplete.to_string();
/// fill!(autocomplete);
/// let autocomplete: &String = &autocomplete;
/// fill!(autocomplete);
/// # });
/// ```
///
/// - *On Pop!_OS Launcher, this is when the user presses `Tab`.*
/// *uses `async_stdout()` exported by the `pop_launcher_toolkit`.*
#[macro_export]
macro_rules! fill {
    ( $text:ident $(,)? ) => (
        ::pop_launcher_toolkit::plugins::send(
            &mut pop_launcher_toolkit::launcher::async_stdout(),
            pop_launcher_toolkit::launcher::PluginResponse::Fill(
                $text.to_string()
            )
        ).await
    );
    ( $text:literal $(,)? ) => (
        {
            let text = $text.to_string();
            $crate::fill!(text)
        }
    );
    ( $( $wrong:tt )? ) => (
        compile_error!("fill! macro requires a String, &str \
                        or String Literal as an argument)");
    );
}


/// **Finish search.**
///
/// The usual response to `Request::Search` from the launcher,
/// after the last `PluginSearchResult` has been sent.
/// It tells the launcher that the plugin is done sending results.
///
/// ## Examples
/// ```rust
/// # use helper::finished;
/// # tokio_test::block_on(async {
/// finished!();
/// # });
/// ```
///
/// *uses `async_stdout()` exported by the `pop_launcher_toolkit`.*
#[macro_export]
macro_rules! finished {
    () => (
        pop_launcher_toolkit::plugins::send(
            &mut pop_launcher_toolkit::launcher::async_stdout(),
            pop_launcher_toolkit::launcher::PluginResponse::Finished
        ).await
    );
}


/// **send an error to be displayed in launcher**
///
///
/// ## Examples
///
/// ```rust
/// # use helper::display_err;
/// # tokio_test::block_on(async {
/// let some_error_title = "Error Title!";
/// let some_error_description = "Error description";
/// display_err!(some_error_title);
/// display_err!(some_error_title, some_error_description);
/// display_err!("Error Title!");
/// display_err!("Error Title!", "Error description");
/// # });
/// ```
/// sends:
/// ```ignore
/// PluginSearchResult {
///     id: 0 as Indice,
///     name: "Error!".to_string(),
///     description: "Error description".to_string(),
///     icon: Some(IconSource::Name(Cow::Borrowed(
///             "dialog-error"))),
///     ..Default::default()
/// }
/// ```
///
/// *uses `async_stdout()` exported by the `pop_launcher_toolkit`.*
#[macro_export]
macro_rules! display_err {
    ( $name:expr $(, $description:expr )? $(,)? ) => (
        $crate::append!((
            pop_launcher_toolkit::launcher::PluginSearchResult {
                id: 0 as pop_launcher_toolkit::launcher::Indice,
                name: format!("{}", $name),
                $(
                    description: format!("{}", $description),
                )?
                icon: Some(
                    pop_launcher_toolkit::launcher::IconSource::Name(
                        std::borrow::Cow::Borrowed("dialog-error"))),
                ..Default::default()
            }
        ))
    );
}
