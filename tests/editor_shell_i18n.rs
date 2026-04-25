use std::fs;
use std::path::PathBuf;

fn shell_html() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("assets").join("editor_shell.html");
    fs::read_to_string(path).expect("read editor_shell.html")
}

#[test]
fn has_language_selector_control() {
    let html = shell_html();
    assert!(
        html.contains("id=\"langSelect\""),
        "expected a language selector element with id=langSelect"
    );
}

#[test]
fn includes_english_and_chinese_ui_dictionary() {
    let html = shell_html();
    assert!(
        html.contains("const UI_TEXT ="),
        "expected UI_TEXT dictionary in editor shell"
    );
    assert!(html.contains("Open"), "expected English UI labels");
    assert!(html.contains("打开"), "expected Chinese UI labels");
}

#[test]
fn defaults_to_english_without_saved_locale() {
    let html = shell_html();
    assert!(
        html.contains("const storedLocale = readStoredLocale();"),
        "expected bootstrap variable for saved locale"
    );
    assert!(
        html.contains("let appLocale = storedLocale ? normalizeLocale(storedLocale) : \"en\";"),
        "expected default locale to be English when no saved preference exists"
    );
}

#[test]
fn only_forces_preview_container_visible_in_split_mode() {
    let html = shell_html();
    assert!(
        !html.contains("display: flex !important;"),
        "expected no CSS override that forces the preview container visible"
    );
    assert!(
        html.contains("if (currentMode !== \"sv\")"),
        "expected a split-mode guard before preview visibility overrides"
    );
    assert!(
        html.contains("previewEl.style.display = 'flex';"),
        "expected the preview container to be forced into flex layout in split mode"
    );
}

#[test]
fn toolbar_contains_version_badge() {
    let html = shell_html();
    assert!(
        html.contains("id=\"appVersion\""),
        "expected a toolbar element with id=appVersion"
    );
    assert!(
        html.contains("class=\"version-badge\""),
        "expected a dedicated version badge class in the editor shell"
    );
}

#[test]
fn app_shell_handles_redo_shortcuts_when_toolbar_redo_exists() {
    let html = shell_html();
    assert!(
        html.contains("function runEditorHistoryCommand(command)"),
        "expected app shell history shortcut bridge"
    );
    assert!(
        html.contains("key === \"y\"") && html.contains("runEditorHistoryCommand(\"redo\")"),
        "expected Ctrl/Cmd+Y to trigger editor redo"
    );
    assert!(
        html.contains("key === \"z\" && event.shiftKey")
            && html.contains("runEditorHistoryCommand(\"redo\")"),
        "expected Ctrl/Cmd+Shift+Z to trigger editor redo"
    );
}

#[test]
fn app_shell_confirms_native_close_when_tabs_are_dirty() {
    let html = shell_html();
    assert!(
        html.contains("function hasDirtyTabs()"),
        "expected helper that checks dirty tabs before native close"
    );
    assert!(
        html.contains("message.event === \"close_requested\""),
        "expected host close request handling"
    );
    assert!(
        html.contains("cmd: \"close_confirmed\""),
        "expected renderer to notify host after close is confirmed"
    );
}
