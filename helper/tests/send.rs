use helper::{
    append,
    clear,
    close,
    context,
    deactivate,
    desktop_entry,
    fill,
    finished,
    display_err
};


#[tokio::test]
#[allow(clippy::items_after_statements)]
async fn append() {
    {
        append!(pop_launcher_toolkit::launcher::PluginSearchResult {
            ..Default::default()
        });
    }
    {
        use pop_launcher_toolkit::launcher::PluginSearchResult;
        append!(PluginSearchResult{
            ..Default::default()
        });
    }
    {
        use pop_launcher_toolkit::launcher;
        let result = launcher::PluginSearchResult {
            ..Default::default()
        };
        append!(result);
    }
}


#[tokio::test]
async fn clear() {
    clear!();
}


#[tokio::test]
async fn close() {
    close!();
}


#[tokio::test]
#[allow(clippy::items_after_statements)]
async fn context() {
    let search_result_id: pop_launcher_toolkit::launcher::Indice = 0;

    // single context options
    {
        context!(0, "test");

        let option: &str = "test";
        context!(0, [option]);
        context!(search_result_id, [option]);

        let option: String = option.to_string();
        let option2: String = option.clone();
        context!(0, [option]);
        context!(search_result_id, [option2]);
    }

    // 2 or more context options
    {
        use pop_launcher_toolkit::launcher::ContextOption;
        let options: Vec<ContextOption> = vec![
            ContextOption {
                id: 0,
                name: "test1".to_string(),
            },
            ContextOption {
                id: 1,
                name: "test2".to_string(),
            },
        ];
        let options2 = options.clone();
        context!(0, options);
        context!(search_result_id, options2);

        context!(0, ["test1", "test2"]);
        context!(search_result_id, ["test1", "test2", "test3"]);
    }
}


#[tokio::test]
async fn deactivate() {
    deactivate!();
}


#[tokio::test]
async fn desktop_entry() {
    use pop_launcher_toolkit::launcher::GpuPreference;

    desktop_entry!("/path/to/desktop/entry");
    desktop_entry!("/path/to/desktop/entry", GpuPreference::Default);
    desktop_entry!("/path/to/desktop/entry", GpuPreference::NonDefault);
    desktop_entry!("/path/to/desktop/entry", Default);
    desktop_entry!("/path/to/desktop/entry", NonDefault);

    let path: &str = "/path/to/desktop/entry";
    desktop_entry!(path);
    desktop_entry!(path, GpuPreference::Default);
    desktop_entry!(path, GpuPreference::NonDefault);
    desktop_entry!(path, Default);
    desktop_entry!(path, NonDefault);

    let path: String = path.to_string();
    let path2: String = path.clone();
    let path3: String = path2.clone();
    let path4: String = path3.clone();
    let path5: String = path4.clone();
    desktop_entry!(path);
    desktop_entry!(path2, GpuPreference::Default);
    desktop_entry!(path3, GpuPreference::NonDefault);
    desktop_entry!(path4, Default);
    desktop_entry!(path5, NonDefault);
}


#[tokio::test]
async fn fill() {
    fill!("completion");

    let completion: &str = "completion";
    fill!(completion);

    let completion: String = completion.to_string();
    fill!(completion);
}


#[tokio::test]
async fn finished() {
    finished!();
}


#[tokio::test]
async fn display_err() {
    display_err!("Success!");
    display_err!("Success!", "Success description");

    let name: &str = "Success!";
    let description: &str = "Success description";
    display_err!(name);
    display_err!(name, description);
    display_err!(name, "Success description");
    display_err!("Success!", description);

    let name: String = name.to_string();
    display_err!(name);
    display_err!(name, description);
    display_err!(name, "Success description");
    display_err!("Success!", description);

    let description: String = description.to_string();
    display_err!(name);
    display_err!(name, description);
    display_err!(name, "Success description");
    display_err!("Success!", description);
}
