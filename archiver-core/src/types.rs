use std::path::PathBuf;

/// Уровень сжатия — единая шкала для всех форматов, каждый обработчик
/// сам мапит её на свои внутренние значения (0-9 для deflate, 0-9 для xz и т.д.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompressionLevel {
    Fast,
    Balanced,
    Maximum,
}

impl CompressionLevel {
    pub fn as_deflate_level(&self) -> i64 {
        match self {
            CompressionLevel::Fast => 1,
            CompressionLevel::Balanced => 6,
            CompressionLevel::Maximum => 9,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackOptions {
    pub level: Option<CompressionLevel>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnpackOptions {
    pub password: Option<String>,
    /// Если true — существующие файлы в целевой папке перезаписываются.
    pub overwrite: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEntry {
    pub path: String,
    pub size_bytes: u64,
    pub is_dir: bool,
}

/// События прогресса, которые ядро шлёт наверх (в Tauri-обвязке
/// транслируются в Channel / `window.emit("archive-progress", ev)`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ProgressEvent {
    #[serde(rename_all = "camelCase")]
    Started { total_entries: Option<u64> },
    #[serde(rename_all = "camelCase")]
    Entry { path: PathBuf, index: u64 },
    Finished,
}

pub type ProgressCallback<'a> = dyn FnMut(ProgressEvent) + 'a;

/// Единый интерфейс, который реализует каждый обработчик формата.
/// Tauri-команды работают только через этот трейт и `ArchiveFormat::detect`,
/// не зная о деталях конкретного формата.
pub trait Archiver {
    /// Упаковать список файлов/папок в архив по пути `dest`.
    fn pack(
        &self,
        sources: &[PathBuf],
        dest: &std::path::Path,
        options: &PackOptions,
        on_progress: &mut ProgressCallback,
    ) -> crate::Result<()>;

    /// Распаковать архив в папку `dest`.
    fn unpack(
        &self,
        archive: &std::path::Path,
        dest: &std::path::Path,
        options: &UnpackOptions,
        on_progress: &mut ProgressCallback,
    ) -> crate::Result<()>;

    /// Получить список файлов внутри архива без полной распаковки.
    fn list(&self, archive: &std::path::Path) -> crate::Result<Vec<ArchiveEntry>>;
}
