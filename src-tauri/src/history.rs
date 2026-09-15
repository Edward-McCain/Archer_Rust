use crate::error::CommandResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const HISTORY_FILE: &str = "history.json";
const MAX_HISTORY: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: String,
    pub kind: String, // "pack" | "unpack"
    pub path: String,
    pub secondary: Option<String>,
    pub format: Option<String>,
    pub timestamp: u64,
}

fn history_path(app: &AppHandle) -> CommandResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Не удалось получить каталог данных приложения: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("Не удалось создать каталог данных: {e}"))?;
    Ok(dir.join(HISTORY_FILE))
}

fn read_history(path: &PathBuf) -> Vec<HistoryItem> {
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

#[tauri::command]
pub fn get_history(app: AppHandle) -> CommandResult<Vec<HistoryItem>> {
    let path = history_path(&app)?;
    Ok(read_history(&path))
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> CommandResult<()> {
    let path = history_path(&app)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Не удалось очистить историю: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub fn push_history(app: AppHandle, item: HistoryItem) -> CommandResult<Vec<HistoryItem>> {
    let path = history_path(&app)?;
    let mut items = read_history(&path);
    items.retain(|existing| !(existing.kind == item.kind && existing.path == item.path));
    items.insert(0, item);
    items.truncate(MAX_HISTORY);
    let raw = serde_json::to_string_pretty(&items)
        .map_err(|e| format!("Не удалось сериализовать историю: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("Не удалось сохранить историю: {e}"))?;
    Ok(items)
}
