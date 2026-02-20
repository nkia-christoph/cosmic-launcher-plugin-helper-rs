#[test]
fn pop_launcher_plugin_helper_macros_test() {
    macrotest::expand("tests/cases/main*.rs");

    let t = trybuild::TestCases::new();
    t.compile_fail("tests/cases/other_plugin.rs");
    t.pass("tests/cases/main_plugi*.rs");
}
