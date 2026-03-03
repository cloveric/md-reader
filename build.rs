#[cfg(target_os = "windows")]
const APP_ICON_PNG: &[u8] = include_bytes!("assets/app_icon.png");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/app_icon.png");

    #[cfg(target_os = "windows")]
    if let Err(err) = embed_windows_icon() {
        panic!("failed to embed Windows icon: {err}");
    }
}

#[cfg(target_os = "windows")]
fn embed_windows_icon() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let ico_path = out_dir.join("md-beader.ico");
    write_ico(&ico_path)?;

    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico_path.to_string_lossy().as_ref());
    res.compile()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn write_ico(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let decoded = image::load_from_memory_with_format(APP_ICON_PNG, image::ImageFormat::Png)?;
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    let icon_image = ico::IconImage::from_rgba_data(width, height, rgba.into_raw());
    let entry = ico::IconDirEntry::encode(&icon_image)?;
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    dir.add_entry(entry);

    let mut file = std::fs::File::create(path)?;
    dir.write(&mut file)?;
    Ok(())
}
