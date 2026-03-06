use std::fs;
use std::path::PathBuf;

fn repo_file(relative: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(relative)).expect("read repository file")
}

#[test]
fn configures_stable_webview_data_directory() {
    let main_rs = repo_file("src/main.rs");
    assert!(
        main_rs.contains("WebViewBuilder::with_web_context"),
        "expected main.rs to build the webview with an explicit WebContext"
    );
    assert!(
        main_rs.contains("webview_data_directory()"),
        "expected main.rs to use a stable webview data directory helper"
    );
}

#[test]
fn serves_editor_shell_from_custom_protocol_url() {
    let main_rs = repo_file("src/main.rs");
    assert!(
        main_rs.contains(".with_custom_protocol("),
        "expected main.rs to register a custom protocol for the editor shell"
    );
    assert!(
        main_rs.contains(".with_url(APP_INDEX_URL)"),
        "expected main.rs to load the editor shell from a fixed custom-protocol URL"
    );
    assert!(
        !main_rs.contains(".with_html(INDEX_HTML_TEMPLATE.to_owned())"),
        "expected main.rs not to use with_html because that gives the page a null origin"
    );
}
