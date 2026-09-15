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
    ArchiveEntry, Archiver, CompressionLevel, PackOptions, ProgressCallback, ProgressEvent,
    UnpackOptions,
};

use std::path::Path;

/// Единственная точка получения обработчика нужного формата.
/// Tauri-команды должны использовать только эту функцию, не создавая
/// конкретные структуры (`ZipArchiver` и т.д.) напрямую — так формат
/// RAR/7z можно добавить, не трогая вызывающий код.
pub fn get_archiver(format: ArchiveFormat) -> Result<Box<dyn Archiver>> {
    use handlers::{RarArchiver, SevenZArchiver, SingleCompressArchiver, TarArchiver, ZipArchiver};

    match format {
        ArchiveFormat::Zip => Ok(Box::new(ZipArchiver)),
        ArchiveFormat::Tar
        | ArchiveFormat::TarGz
        | ArchiveFormat::TarBz2
        | ArchiveFormat::TarXz => Ok(Box::new(TarArchiver { variant: format })),
        ArchiveFormat::SevenZ => Ok(Box::new(SevenZArchiver)),
        ArchiveFormat::Rar => Ok(Box::new(RarArchiver)),
        ArchiveFormat::Gzip | ArchiveFormat::Bzip2 | ArchiveFormat::Xz => {
            Ok(Box::new(SingleCompressArchiver { variant: format }))
        }
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
            .pack(
                &[src_file.clone()],
                &archive_path,
                &PackOptions::default(),
                &mut |_| {},
            )
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
        assert_eq!(
            ArchiveFormat::from_extension(Path::new("a.7z")),
            Some(ArchiveFormat::SevenZ)
        );
    }

    #[test]
    fn sevenz_roundtrip() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("note.txt");
        std::fs::write(&src_file, b"sevenz-ok").unwrap();

        let archive_path = dir.path().join("out.7z");
        let extract_dir = dir.path().join("extracted7z");

        let archiver = get_archiver(ArchiveFormat::SevenZ).unwrap();
        archiver
            .pack(
                &[src_file],
                &archive_path,
                &PackOptions::default(),
                &mut |_| {},
            )
            .unwrap();

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

        let entries = archiver.list(&archive_path).unwrap();
        assert!(!entries.is_empty());
        assert!(extract_dir.join("note.txt").exists() || extract_dir.join("note.txt").is_file()
            || std::fs::read_to_string(extract_dir.join("note.txt")).is_ok()
            || {
                // sevenz may nest under relative path; search recursively
                walkdir::WalkDir::new(&extract_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .any(|e| e.file_name() == "note.txt")
            });
    }

    #[test]
    fn gzip_single_roundtrip() {
        let dir = tempdir().unwrap();
        let src_file = dir.path().join("data.bin");
        std::fs::write(&src_file, b"gzip-payload").unwrap();

        let archive_path = dir.path().join("data.bin.gz");
        let extract_dir = dir.path().join("out");
        std::fs::create_dir_all(&extract_dir).unwrap();

        let archiver = get_archiver(ArchiveFormat::Gzip).unwrap();
        archiver
            .pack(
                &[src_file],
                &archive_path,
                &PackOptions::default(),
                &mut |_| {},
            )
            .unwrap();

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

        let restored = extract_dir.join("data.bin");
        assert_eq!(std::fs::read_to_string(restored).unwrap(), "gzip-payload");
    }

    #[test]
    fn rar_pack_rejected() {
        let archiver = get_archiver(ArchiveFormat::Rar).unwrap();
        let err = archiver
            .pack(&[], Path::new("/tmp/x.rar"), &PackOptions::default(), &mut |_| {})
            .unwrap_err();
        assert!(matches!(err, ArchiverError::UnsupportedOperation(_)));
    }
}
