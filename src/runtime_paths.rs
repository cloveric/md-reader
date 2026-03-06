use std::path::PathBuf;

fn preferred_webview_data_directory(
    os: &str,
    local_app_data: Option<PathBuf>,
    home: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
) -> PathBuf {
    match os {
        "windows" => local_app_data
            .unwrap_or_else(std::env::temp_dir)
            .join("md-bider")
            .join("webview"),
        "macos" => home
            .unwrap_or_else(std::env::temp_dir)
            .join("Library")
            .join("Application Support")
            .join("md-bider")
            .join("webview"),
        _ => {
            if let Some(xdg) = xdg_data_home {
                xdg.join("md-bider").join("webview")
            } else {
                home.unwrap_or_else(std::env::temp_dir)
                    .join(".local")
                    .join("share")
                    .join("md-bider")
                    .join("webview")
            }
        }
    }
}

pub fn webview_data_directory() -> PathBuf {
    let preferred = preferred_webview_data_directory(
        std::env::consts::OS,
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
    );
    if std::fs::create_dir_all(&preferred).is_ok() {
        return preferred;
    }

    let fallback = std::env::temp_dir().join("md-bider").join("webview");
    let _ = std::fs::create_dir_all(&fallback);
    fallback
}

#[cfg(test)]
mod tests {
    use super::preferred_webview_data_directory;
    use std::path::PathBuf;

    #[test]
    fn runtime_paths_windows_use_local_app_data() {
        let path = preferred_webview_data_directory(
            "windows",
            Some(PathBuf::from(r"C:\Users\hangw\AppData\Local")),
            None,
            None,
        );
        assert_eq!(
            path,
            PathBuf::from(r"C:\Users\hangw\AppData\Local")
                .join("md-bider")
                .join("webview")
        );
    }

    #[test]
    fn runtime_paths_macos_use_application_support() {
        let path = preferred_webview_data_directory(
            "macos",
            None,
            Some(PathBuf::from("/Users/hangw")),
            None,
        );
        assert_eq!(
            path,
            PathBuf::from("/Users/hangw")
                .join("Library")
                .join("Application Support")
                .join("md-bider")
                .join("webview")
        );
    }

    #[test]
    fn runtime_paths_linux_fall_back_to_xdg_or_local_share() {
        let xdg_path = preferred_webview_data_directory(
            "linux",
            None,
            Some(PathBuf::from("/home/hangw")),
            Some(PathBuf::from("/home/hangw/.local/share")),
        );
        assert_eq!(
            xdg_path,
            PathBuf::from("/home/hangw/.local/share")
                .join("md-bider")
                .join("webview")
        );

        let fallback_path = preferred_webview_data_directory(
            "linux",
            None,
            Some(PathBuf::from("/home/hangw")),
            None,
        );
        assert_eq!(
            fallback_path,
            PathBuf::from("/home/hangw")
                .join(".local")
                .join("share")
                .join("md-bider")
                .join("webview")
        );
    }
}
