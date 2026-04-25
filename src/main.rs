#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use base64::Engine as _;

use md_bider::app_init::build_initialization_script;
use md_bider::assets::{UploadedAssetRegistry, content_type_for_path, sanitize_upload_name};
use md_bider::desktop::{HostEvent, IpcCommand, to_webview_script};
use md_bider::io::{read_text_with_fallback, write_text_utf8};
use md_bider::runtime_paths::webview_data_directory;
use rfd::FileDialog;
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tao::window::{Icon, WindowBuilder};
use wry::http::{Response, StatusCode, header::CONTENT_TYPE};
use wry::{WebContext, WebView, WebViewBuilder};

#[derive(Debug)]
enum UserEvent {
    Ipc(String),
}

const INDEX_HTML_TEMPLATE: &str = include_str!("../assets/editor_shell.html");
const APP_ICON_PNG: &[u8] = include_bytes!("../assets/app_icon.png");
const APP_CUSTOM_PROTOCOL: &str = "md-bider";
const APP_INDEX_URL: &str = "md-bider://localhost/index.html";
type SharedAssetRegistry = Arc<Mutex<UploadedAssetRegistry>>;

fn send_event(webview: &WebView, event: HostEvent) {
    if let Ok(script) = to_webview_script(&event) {
        let _ = webview.evaluate_script(&script);
    }
}

fn app_protocol_response(path: &str, assets: &SharedAssetRegistry) -> Response<Vec<u8>> {
    let asset_path = assets
        .lock()
        .ok()
        .and_then(|registry| registry.resolve_request_path(path));
    if let Some(asset_path) = asset_path
        && let Ok(body) = std::fs::read(&asset_path)
    {
        return build_response(StatusCode::OK, content_type_for_path(&asset_path), body);
    }

    let (status, content_type, body) = match path {
        "/" | "/index.html" => (
            StatusCode::OK,
            "text/html; charset=utf-8",
            INDEX_HTML_TEMPLATE.as_bytes().to_vec(),
        ),
        _ => (
            StatusCode::NOT_FOUND,
            "text/plain; charset=utf-8",
            b"Not Found".to_vec(),
        ),
    };

    build_response(status, content_type, body)
}

fn build_response(status: StatusCode, content_type: &str, body: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, content_type)
        .body(body)
        .expect("build app protocol response")
}

fn load_window_icon() -> Option<Icon> {
    let image = image::load_from_memory_with_format(APP_ICON_PNG, image::ImageFormat::Png).ok()?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    Icon::from_rgba(rgba.into_raw(), width, height).ok()
}

fn next_tab_id(tab_seq: &mut u64) -> String {
    *tab_seq += 1;
    format!("tab-{}", tab_seq)
}

fn normalize_path(path: Option<String>) -> Option<PathBuf> {
    let raw = path?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

fn default_name_from_path(path: Option<&PathBuf>) -> &str {
    path.and_then(|p| p.file_name().and_then(|name| name.to_str()))
        .unwrap_or("untitled.md")
}

fn unique_file_path(dir: &Path, name: &str) -> PathBuf {
    let base = Path::new(name);
    let stem = base.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
    let ext = base.extension().and_then(|e| e.to_str()).unwrap_or("png");

    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }

    let mut counter: u32 = 1;
    loop {
        let numbered = dir.join(format!("{stem}_{counter}.{ext}"));
        if !numbered.exists() {
            return numbered;
        }
        counter += 1;
    }
}

fn upload_image(
    webview: &WebView,
    assets: &SharedAssetRegistry,
    tab_id: String,
    name: &str,
    data: &str,
    dir: Option<String>,
) {
    let bytes = match base64::engine::general_purpose::STANDARD.decode(data) {
        Ok(b) => b,
        Err(err) => {
            send_event(
                webview,
                HostEvent::Error {
                    message: format!("图片解码失败: {err}"),
                },
            );
            return;
        }
    };

    let assets_dir = match dir.as_deref().filter(|d| !d.trim().is_empty()) {
        Some(d) => PathBuf::from(d).join("assets"),
        None => std::env::temp_dir().join("md-bider-uploads").join("assets"),
    };

    if let Err(err) = std::fs::create_dir_all(&assets_dir) {
        send_event(
            webview,
            HostEvent::Error {
                message: format!("创建目录失败: {err}"),
            },
        );
        return;
    }

    let safe_name = sanitize_upload_name(name);
    let dest = unique_file_path(&assets_dir, &safe_name);
    if let Err(err) = std::fs::write(&dest, &bytes) {
        send_event(
            webview,
            HostEvent::Error {
                message: format!("图片保存失败: {err}"),
            },
        );
        return;
    }

    let relative_url = format!(
        "assets/{}",
        dest.file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(&safe_name)
    );
    if let Ok(mut registry) = assets.lock() {
        registry.register_uploaded_asset(relative_url.clone(), dest);
    }

    send_event(
        webview,
        HostEvent::ImageUploaded {
            tab_id,
            url: relative_url,
        },
    );
}

fn register_document_assets(assets: &SharedAssetRegistry, path: &Path) {
    if let Ok(mut registry) = assets.lock() {
        registry.register_document_path(path);
    }
}

fn open_file_into_editor(
    webview: &WebView,
    assets: &SharedAssetRegistry,
    tab_id: String,
    path: PathBuf,
) {
    match read_text_with_fallback(&path) {
        Ok(content) => {
            register_document_assets(assets, &path);
            send_event(
                webview,
                HostEvent::FileOpened {
                    tab_id,
                    path: path.display().to_string(),
                    content,
                },
            );
        }
        Err(err) => {
            send_event(
                webview,
                HostEvent::Error {
                    message: format!("打开失败: {err}"),
                },
            );
        }
    }
}

fn save_content_to_path(
    webview: &WebView,
    assets: &SharedAssetRegistry,
    tab_id: String,
    path: PathBuf,
    content: &str,
) {
    match write_text_utf8(&path, content) {
        Ok(()) => {
            register_document_assets(assets, &path);
            send_event(
                webview,
                HostEvent::FileSaved {
                    tab_id,
                    path: path.display().to_string(),
                },
            );
        }
        Err(err) => {
            send_event(
                webview,
                HostEvent::Error {
                    message: format!("保存失败: {err}"),
                },
            );
        }
    }
}

fn main() -> wry::Result<()> {
    let initial_file = std::env::args().nth(1).map(PathBuf::from);
    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window_icon = load_window_icon();

    let window = WindowBuilder::new()
        .with_title("md-bider · 马得笔")
        .with_window_icon(window_icon)
        .with_inner_size(LogicalSize::new(1280.0, 860.0))
        .build(&event_loop)
        .map_err(|_| wry::Error::InitScriptError)?;

    let mut web_context = WebContext::new(Some(webview_data_directory()));
    let assets: SharedAssetRegistry = Arc::new(Mutex::new(UploadedAssetRegistry::default()));
    let protocol_assets = Arc::clone(&assets);

    let webview = WebViewBuilder::with_web_context(&mut web_context)
        .with_custom_protocol(APP_CUSTOM_PROTOCOL.into(), move |_webview_id, request| {
            app_protocol_response(request.uri().path(), &protocol_assets).map(Into::into)
        })
        .with_initialization_script(build_initialization_script())
        .with_url(APP_INDEX_URL)
        .with_ipc_handler(move |request| {
            let _ = proxy.send_event(UserEvent::Ipc(request.body().to_owned()));
        })
        .build(&window)?;

    let mut pending_initial_file = initial_file;
    let mut tab_seq: u64 = 0;

    event_loop.run(move |event, _window_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                send_event(&webview, HostEvent::CloseRequested);
            }
            Event::UserEvent(UserEvent::Ipc(payload)) => match IpcCommand::parse(&payload) {
                Ok(IpcCommand::AppReady { tab_id }) => {
                    send_event(
                        &webview,
                        HostEvent::Status {
                            message: "编辑器已就绪".to_owned(),
                        },
                    );
                    if let Some(path) = pending_initial_file.take() {
                        let target_tab_id = tab_id.unwrap_or_else(|| next_tab_id(&mut tab_seq));
                        open_file_into_editor(&webview, &assets, target_tab_id, path);
                    }
                }
                Ok(IpcCommand::NewFile { tab_id }) => {
                    let target_tab_id = tab_id.unwrap_or_else(|| next_tab_id(&mut tab_seq));
                    send_event(
                        &webview,
                        HostEvent::FileOpened {
                            tab_id: target_tab_id,
                            path: String::new(),
                            content: String::new(),
                        },
                    );
                }
                Ok(IpcCommand::OpenFile { tab_id }) => {
                    let file = FileDialog::new()
                        .add_filter("Markdown", &["md", "markdown", "txt"])
                        .pick_file();
                    if let Some(path) = file {
                        let target_tab_id = tab_id.unwrap_or_else(|| next_tab_id(&mut tab_seq));
                        open_file_into_editor(&webview, &assets, target_tab_id, path);
                    }
                }
                Ok(IpcCommand::SaveFile {
                    tab_id,
                    path,
                    content,
                }) => {
                    let target_tab_id = tab_id.unwrap_or_else(|| next_tab_id(&mut tab_seq));
                    let current_path = normalize_path(path);
                    if let Some(path) = current_path {
                        save_content_to_path(&webview, &assets, target_tab_id, path, &content);
                    } else {
                        let file = FileDialog::new()
                            .add_filter("Markdown", &["md", "markdown", "txt"])
                            .set_file_name(default_name_from_path(None))
                            .save_file();
                        if let Some(path) = file {
                            save_content_to_path(&webview, &assets, target_tab_id, path, &content);
                        }
                    }
                }
                Ok(IpcCommand::SaveAs {
                    tab_id,
                    path,
                    content,
                }) => {
                    let target_tab_id = tab_id.unwrap_or_else(|| next_tab_id(&mut tab_seq));
                    let current_path = normalize_path(path);
                    let default_name = default_name_from_path(current_path.as_ref());
                    let file = FileDialog::new()
                        .add_filter("Markdown", &["md", "markdown", "txt"])
                        .set_file_name(default_name)
                        .save_file();
                    if let Some(path) = file {
                        save_content_to_path(&webview, &assets, target_tab_id, path, &content);
                    }
                }
                Ok(IpcCommand::UploadImage {
                    tab_id,
                    name,
                    data,
                    dir,
                }) => {
                    let target_tab_id = tab_id.unwrap_or_else(|| next_tab_id(&mut tab_seq));
                    upload_image(&webview, &assets, target_tab_id, &name, &data, dir);
                }
                Ok(IpcCommand::CloseConfirmed) => {
                    *control_flow = ControlFlow::Exit;
                }
                Err(err) => {
                    send_event(
                        &webview,
                        HostEvent::Error {
                            message: format!("无法识别指令: {err}"),
                        },
                    );
                }
            },
            _ => {}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{SharedAssetRegistry, app_protocol_response};
    use md_bider::assets::UploadedAssetRegistry;
    use std::sync::{Arc, Mutex};
    use wry::http::StatusCode;

    #[test]
    fn custom_protocol_serves_registered_uploaded_assets() {
        let dir =
            std::env::temp_dir().join(format!("md-bider-protocol-test-{}", std::process::id()));
        let assets_dir = dir.join("assets");
        std::fs::create_dir_all(&assets_dir).expect("create assets dir");
        let image_path = assets_dir.join("photo.png");
        std::fs::write(&image_path, b"png").expect("write image");

        let assets: SharedAssetRegistry = Arc::new(Mutex::new(UploadedAssetRegistry::default()));
        assets
            .lock()
            .expect("lock registry")
            .register_uploaded_asset("assets/photo.png", image_path);

        let response = app_protocol_response("/assets/photo.png", &assets);
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.body().as_slice(), b"png");

        let _ = std::fs::remove_dir_all(dir);
    }
}
