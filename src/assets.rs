use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct UploadedAssetRegistry {
    explicit_paths: HashMap<String, PathBuf>,
    asset_dirs: HashSet<PathBuf>,
}

impl UploadedAssetRegistry {
    pub fn register_document_path(&mut self, path: &Path) {
        if let Some(parent) = path.parent() {
            self.asset_dirs.insert(parent.join("assets"));
        }
    }

    pub fn register_uploaded_asset(&mut self, url: impl Into<String>, path: PathBuf) {
        if let Some(parent) = path.parent() {
            self.asset_dirs.insert(parent.to_path_buf());
        }
        self.explicit_paths.insert(url.into(), path);
    }

    pub fn resolve_request_path(&self, request_path: &str) -> Option<PathBuf> {
        let key = request_path.trim_start_matches('/');
        if !key.starts_with("assets/") {
            return None;
        }

        if let Some(path) = self.explicit_paths.get(key) {
            return Some(path.clone());
        }

        let file_name = sanitize_upload_name(key.strip_prefix("assets/").unwrap_or(key));
        self.asset_dirs
            .iter()
            .map(|dir| dir.join(&file_name))
            .find(|path| path.is_file())
    }
}

pub fn sanitize_upload_name(name: &str) -> String {
    let basename = Path::new(name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("image.png");
    let sanitized = basename
        .chars()
        .map(|ch| {
            if ch.is_control() || matches!(ch, '/' | '\\' | ':') {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();
    let trimmed = sanitized.trim_matches('.').trim();
    if trimmed.is_empty() {
        "image.png".to_owned()
    } else {
        trimmed.to_owned()
    }
}

pub fn content_type_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "image/png",
    }
}

#[cfg(test)]
mod tests {
    use super::{UploadedAssetRegistry, sanitize_upload_name};
    use std::path::PathBuf;

    #[test]
    fn upload_name_is_reduced_to_safe_basename() {
        assert_eq!(sanitize_upload_name("../outside.png"), "outside.png");
        assert_eq!(sanitize_upload_name("nested/evil.jpg"), "evil.jpg");
        assert_eq!(sanitize_upload_name("bad\nname.jpg"), "bad_name.jpg");
        assert_eq!(sanitize_upload_name("..."), "image.png");
    }

    #[test]
    fn registry_only_resolves_assets_paths() {
        let mut registry = UploadedAssetRegistry::default();
        registry.register_uploaded_asset(
            "assets/photo.png",
            PathBuf::from("/tmp/md-bider/assets/photo.png"),
        );

        assert_eq!(
            registry.resolve_request_path("/assets/photo.png"),
            Some(PathBuf::from("/tmp/md-bider/assets/photo.png"))
        );
        assert_eq!(registry.resolve_request_path("/index.html"), None);
    }
}
