mod commands;
mod error;
mod history;

use commands::{
    detect_format, extract_entries, list_archive, list_formats, pack_archive, unpack_archive,
};
use history::{clear_history, get_history, push_history};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            list_formats,
            detect_format,
            list_archive,
            pack_archive,
            unpack_archive,
            extract_entries,
            get_history,
            push_history,
            clear_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
