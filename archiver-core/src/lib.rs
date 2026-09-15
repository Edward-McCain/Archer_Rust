//! Ядро логики архивации/разархивации.
//!
//! Не знает ничего про UI — вызывается из Tauri-команд (см. src-tauri/).
//! Публичный API: `ArchiveFormat::detect`, `get_archiver`, трейт `Archiver`.

mod error;
mod format;
mod handlers;
mod types;

pub use error::{ArchiverError, Result};
pub use format::ArchiveFormat;
pub use types::{
    Archiver, ArchiveEntry, CompressionLevel, PackOptions, ProgressCallback, ProgressEvent,
    UnpackOptions,
};

use std::path::Path;

/// Единственная точка получения обработчика нужного формата.
/// Tauri-команды должны использовать только эту функцию, не создавая
/// конкретные структуры (`ZipArchiver` и т.д.) напрямую — так формат
/// RAR/7z можно добавить, не трогая вызывающий код.
pub fn get_archiver(format: ArchiveFormat) -> Result<Box<dyn Archiver>> {
    use handlers::{TarArchiver, ZipArchiver};

    match format {
        ArchiveFormat::Zip => Ok(Box::new(ZipArchiver)),
        ArchiveFormat::Tar
        | ArchiveFormat::TarGz
        | ArchiveFormat::TarBz2
        | ArchiveFormat::TarXz => Ok(Box::new(TarArchiver { variant: format })),
        ArchiveFormat::SevenZ => Err(ArchiverError::UnsupportedOperation(
            "7z: обработчик ещё не реализован (см. handlers/mod.rs TODO)",
        )),
        ArchiveFormat::Rar => Err(ArchiverError::UnsupportedOperation(
            "RAR: обработчик ещё не реализован (см. handlers/mod.rs TODO)",
        )),
        ArchiveFormat::Gzip | ArchiveFormat::Bzip2 | ArchiveFormat::Xz => Err(
            ArchiverError::UnsupportedOperation("одиночные .gz/.bz2/.xz без TAR пока не поддержаны"),
        ),
    }
}

/// Удобная обёртка: определить формат по пути и сразу получить обработчик.
pub fn archiver_for_path(path: &Path) -> Result<Box<dyn Archiver>> {
    let format = ArchiveFormat::detect(path)?;
    get_archiver(format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn zip_roundtrip() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("hello.txt");
        std::fs::File::create(&src_file)
            .unwrap()
            .write_all(b"privet mir")
            .unwrap();

        let archive_path = dir.path().join("out.zip");
        let extract_dir = dir.path().join("extracted");
        std::fs::create_dir_all(&extract_dir).unwrap();

        let archiver = get_archiver(ArchiveFormat::Zip).unwrap();
        archiver
            .pack(&[src_file.clone()], &archive_path, &PackOptions::default(), &mut |_| {})
            .unwrap();

        assert!(archive_path.exists());

        archiver
            .unpack(
                &archive_path,
                &extract_dir,
                &UnpackOptions {
                    overwrite: true,
                    ..Default::default()
                },
                &mut |_| {},
            )
            .unwrap();

        let restored = extract_dir.join("hello.txt");
        assert!(restored.exists());
        assert_eq!(std::fs::read_to_string(restored).unwrap(), "privet mir");
    }

    #[test]
    fn detect_by_extension() {
        assert_eq!(
            ArchiveFormat::from_extension(Path::new("archive.tar.gz")),
            Some(ArchiveFormat::TarGz)
        );
        assert_eq!(
            ArchiveFormat::from_extension(Path::new("archive.zip")),
            Some(ArchiveFormat::Zip)
        );
    }
}
