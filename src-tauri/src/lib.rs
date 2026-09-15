// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use archiver_core::ArchiveFormat;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! Archer core is wired in.")
}

/// Smoke-test command: detect archive format from a path string.
#[tauri::command]
fn detect_format(path: String) -> Result<String, String> {
    let format = ArchiveFormat::from_extension(std::path::Path::new(&path))
        .ok_or_else(|| "Unknown or unsupported archive format".to_string())?;
    Ok(format.display_name().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, detect_format])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
