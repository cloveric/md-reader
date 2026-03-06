use std::fs;
use std::path::PathBuf;

fn release_workflow() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(".github").join("workflows").join("release.yml"))
        .expect("read release workflow")
}

#[test]
fn keeps_windows_and_macos_release_jobs() {
    let workflow = release_workflow();
    assert!(
        workflow.contains("build-windows:"),
        "expected release workflow to keep the Windows packaging job"
    );
    assert!(
        workflow.contains("build-macos:"),
        "expected release workflow to keep the macOS packaging job"
    );
    assert!(
        workflow.contains("name: release-windows"),
        "expected release workflow to upload the Windows release artifact"
    );
    assert!(
        workflow.contains("name: release-macos"),
        "expected release workflow to upload the macOS release artifact"
    );
}
