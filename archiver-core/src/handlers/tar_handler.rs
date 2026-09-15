use crate::format::ArchiveFormat;
use crate::types::{Archiver, ArchiveEntry, PackOptions, ProgressCallback, ProgressEvent, UnpackOptions};
use crate::Result;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

/// Обработчик для TAR и его сжатых вариантов (.tar, .tar.gz, .tar.bz2, .tar.xz).
/// Пароли для TAR-семейства не поддерживаются (формат этого не умеет) —
/// если `options.password` задан, он молча игнорируется.
pub struct TarArchiver {
    pub variant: ArchiveFormat,
}

impl TarArchiver {
    fn writer_for<'a>(&self, file: File, level: i64) -> Result<Box<dyn Write + 'a>> {
        Ok(match self.variant {
            ArchiveFormat::Tar => Box::new(file),
            ArchiveFormat::TarGz => Box::new(GzEncoder::new(file, Compression::new(level as u32))),
            ArchiveFormat::TarBz2 => Box::new(BzEncoder::new(file, bzip2::Compression::new(level as u32))),
            ArchiveFormat::TarXz => Box::new(XzEncoder::new(file, level as u32)),
            _ => return Err(crate::ArchiverError::UnsupportedOperation("tar_handler: неверный вариант")),
        })
    }

    fn reader_for<'a>(&self, file: File) -> Result<Box<dyn Read + 'a>> {
        Ok(match self.variant {
            ArchiveFormat::Tar => Box::new(file),
            ArchiveFormat::TarGz => Box::new(GzDecoder::new(file)),
            ArchiveFormat::TarBz2 => Box::new(BzDecoder::new(file)),
            ArchiveFormat::TarXz => Box::new(XzDecoder::new(file)),
            _ => return Err(crate::ArchiverError::UnsupportedOperation("tar_handler: неверный вариант")),
        })
    }
}

impl Archiver for TarArchiver {
    fn pack(
        &self,
        sources: &[PathBuf],
        dest: &Path,
        options: &PackOptions,
        on_progress: &mut ProgressCallback,
    ) -> Result<()> {
        let file = File::create(dest)?;
        let level = options.level.map(|l| l.as_deflate_level()).unwrap_or(6);
        let writer = self.writer_for(file, level)?;
        let mut builder = Builder::new(writer);

        on_progress(ProgressEvent::Started { total_entries: None });

        let base_parent = sources
            .first()
            .and_then(|p| p.parent())
            .unwrap_or_else(|| Path::new(""));

        for (idx, src) in sources.iter().enumerate() {
            let name_in_tar = src.strip_prefix(base_parent).unwrap_or(src);
            if src.is_dir() {
                builder.append_dir_all(name_in_tar, src)?;
            } else {
                let mut f = File::open(src)?;
                builder.append_file(name_in_tar, &mut f)?;
            }
            on_progress(ProgressEvent::Entry {
                path: src.clone(),
                index: idx as u64,
            });
        }

        builder.into_inner()?.flush()?;
        on_progress(ProgressEvent::Finished);
        Ok(())
    }

    fn unpack(
        &self,
        archive: &Path,
        dest: &Path,
        _options: &UnpackOptions,
        on_progress: &mut ProgressCallback,
    ) -> Result<()> {
        let file = File::open(archive)?;
        let reader = self.reader_for(file)?;
        let mut tar = Archive::new(reader);

        on_progress(ProgressEvent::Started { total_entries: None });

        for (idx, entry) in tar.entries()?.enumerate() {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            entry.unpack_in(dest)?;
            on_progress(ProgressEvent::Entry {
                path,
                index: idx as u64,
            });
        }

        on_progress(ProgressEvent::Finished);
        Ok(())
    }

    fn list(&self, archive: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(archive)?;
        let reader = self.reader_for(file)?;
        let mut tar = Archive::new(reader);
        let mut entries = Vec::new();

        for entry in tar.entries()? {
            let entry = entry?;
            let header = entry.header();
            entries.push(ArchiveEntry {
                path: entry.path()?.to_string_lossy().to_string(),
                size_bytes: header.size().unwrap_or(0),
                is_dir: header.entry_type().is_dir(),
            });
        }

        Ok(entries)
    }
}
