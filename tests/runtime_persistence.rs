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
