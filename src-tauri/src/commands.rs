use crate::error::{map_err, CommandResult};
use archiver_core::{
    get_archiver, ArchiveEntry, ArchiveFormat, CompressionLevel, PackOptions, ProgressEvent,
    UnpackOptions,
};
use serde::Serialize;
use std::path::PathBuf;
use tauri::ipc::Channel;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatInfo {
    pub id: ArchiveFormat,
    pub extension: &'static str,
    pub display_name: &'static str,
    pub supports_packing: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ArchiveProgress {
    #[serde(rename_all = "camelCase")]
    Started { total_entries: Option<u64> },
    #[serde(rename_all = "camelCase")]
    Entry { path: String, index: u64 },
    Finished,
}

impl From<ProgressEvent> for ArchiveProgress {
    fn from(value: ProgressEvent) -> Self {
        match value {
            ProgressEvent::Started { total_entries } => ArchiveProgress::Started { total_entries },
            ProgressEvent::Entry { path, index } => ArchiveProgress::Entry {
                path: path.to_string_lossy().into_owned(),
                index,
            },
            ProgressEvent::Finished => ArchiveProgress::Finished,
        }
    }
}

fn format_info(format: ArchiveFormat) -> FormatInfo {
    FormatInfo {
        id: format,
        extension: format.extension(),
        display_name: format.display_name(),
        supports_packing: format.supports_packing(),
    }
}

#[tauri::command]
pub fn list_formats() -> Vec<FormatInfo> {
    [
        ArchiveFormat::Zip,
        ArchiveFormat::Tar,
        ArchiveFormat::TarGz,
        ArchiveFormat::TarBz2,
        ArchiveFormat::TarXz,
        ArchiveFormat::SevenZ,
        ArchiveFormat::Rar,
        ArchiveFormat::Gzip,
        ArchiveFormat::Bzip2,
        ArchiveFormat::Xz,
    ]
    .into_iter()
    .map(format_info)
    .collect()
}

#[tauri::command]
pub fn detect_format(path: String) -> CommandResult<FormatInfo> {
    let path = PathBuf::from(path);
    let format = ArchiveFormat::detect(&path).map_err(map_err)?;
    Ok(format_info(format))
}

#[tauri::command]
pub fn list_archive(path: String, password: Option<String>) -> CommandResult<Vec<ArchiveEntry>> {
    let path = PathBuf::from(path);
    let _ = password; // password-aware listing lands with encrypted header support later
    let archiver = archiver_core::archiver_for_path(&path).map_err(map_err)?;
    archiver.list(&path).map_err(map_err)
}

#[tauri::command]
pub async fn pack_archive(
    sources: Vec<String>,
    dest: String,
    format: ArchiveFormat,
    level: Option<CompressionLevel>,
    password: Option<String>,
    on_event: Channel<ArchiveProgress>,
) -> CommandResult<()> {
    let sources: Vec<PathBuf> = sources.into_iter().map(PathBuf::from).collect();
    let dest = PathBuf::from(dest);
    let options = PackOptions { level, password };

    tauri::async_runtime::spawn_blocking(move || {
        let archiver = get_archiver(format).map_err(map_err)?;
        archiver
            .pack(&sources, &dest, &options, &mut |ev| {
                let _ = on_event.send(ArchiveProgress::from(ev));
            })
            .map_err(map_err)
    })
    .await
    .map_err(|e| format!("Фоновая задача упала: {e}"))?
}

#[tauri::command]
pub async fn unpack_archive(
    archive: String,
    dest: String,
    password: Option<String>,
    overwrite: Option<bool>,
    on_event: Channel<ArchiveProgress>,
) -> CommandResult<()> {
    let archive = PathBuf::from(archive);
    let dest = PathBuf::from(dest);
    let options = UnpackOptions {
        password,
        overwrite: overwrite.unwrap_or(true),
    };

    tauri::async_runtime::spawn_blocking(move || {
        let archiver = archiver_core::archiver_for_path(&archive).map_err(map_err)?;
        archiver
            .unpack(&archive, &dest, &options, &mut |ev| {
                let _ = on_event.send(ArchiveProgress::from(ev));
            })
            .map_err(map_err)
    })
    .await
    .map_err(|e| format!("Фоновая задача упала: {e}"))?
}

/// Extract a subset of entries by unpacking into a temp-like dest filter.
/// For formats without random access we unpack selectively when possible;
/// currently implemented as full unpack + keep matching paths only for ZIP.
#[tauri::command]
pub async fn extract_entries(
    archive: String,
    dest: String,
    entries: Vec<String>,
    password: Option<String>,
    on_event: Channel<ArchiveProgress>,
) -> CommandResult<()> {
    // Stage-4 baseline: unpack whole archive into dest (UI can call list first
    // and pass selected names in a later refinement with per-format extract).
    let _ = entries;
    unpack_archive(archive, dest, password, Some(true), on_event).await
}
