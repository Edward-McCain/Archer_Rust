use crate::error::ArchiverError;
use crate::format::ArchiveFormat;
use crate::types::{
    ArchiveEntry, Archiver, PackOptions, ProgressCallback, ProgressEvent, UnpackOptions,
};
use crate::Result;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{copy, Read, Write};
use std::path::{Path, PathBuf};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

/// Одиночные сжатые файлы (.gz / .bz2 / .xz), не TAR-контейнеры.
/// Pack принимает ровно один обычный файл; unpack пишет один файл в dest.
pub struct SingleCompressArchiver {
    pub variant: ArchiveFormat,
}

impl SingleCompressArchiver {
    fn ensure_single_file(sources: &[PathBuf]) -> Result<&PathBuf> {
        match sources {
            [one] if one.is_file() => Ok(one),
            [one] if one.is_dir() => Err(ArchiverError::UnsupportedOperation(
                "одиночное сжатие не поддерживает папки — используйте TAR.GZ / 7Z / ZIP",
            )),
            _ => Err(ArchiverError::UnsupportedOperation(
                "одиночное сжатие принимает ровно один файл",
            )),
        }
    }

    fn encoder_for<'a>(&self, file: File, level: u32) -> Result<Box<dyn Write + 'a>> {
        Ok(match self.variant {
            ArchiveFormat::Gzip => Box::new(GzEncoder::new(file, Compression::new(level))),
            ArchiveFormat::Bzip2 => Box::new(BzEncoder::new(file, bzip2::Compression::new(level))),
            ArchiveFormat::Xz => Box::new(XzEncoder::new(file, level)),
            _ => {
                return Err(ArchiverError::UnsupportedOperation(
                    "single_compress: неверный вариант",
                ))
            }
        })
    }

    fn decoder_for<'a>(&self, file: File) -> Result<Box<dyn Read + 'a>> {
        Ok(match self.variant {
            ArchiveFormat::Gzip => Box::new(GzDecoder::new(file)),
            ArchiveFormat::Bzip2 => Box::new(BzDecoder::new(file)),
            ArchiveFormat::Xz => Box::new(XzDecoder::new(file)),
            _ => {
                return Err(ArchiverError::UnsupportedOperation(
                    "single_compress: неверный вариант",
                ))
            }
        })
    }

    fn output_name(archive: &Path) -> String {
        let name = archive
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive");
        let lower = name.to_lowercase();
        for ext in [".gz", ".bz2", ".xz"] {
            if lower.ends_with(ext) {
                return name[..name.len() - ext.len()].to_string();
            }
        }
        format!("{name}.out")
    }
}

impl Archiver for SingleCompressArchiver {
    fn pack(
        &self,
        sources: &[PathBuf],
        dest: &Path,
        options: &PackOptions,
        on_progress: &mut ProgressCallback,
    ) -> Result<()> {
        let src = Self::ensure_single_file(sources)?;
        let level = options
            .level
            .map(|l| l.as_deflate_level() as u32)
            .unwrap_or(6);

        if let Some(parent) = dest.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        on_progress(ProgressEvent::Started {
            total_entries: Some(1),
        });

        let mut input = File::open(src)?;
        let output = File::create(dest)?;
        let mut encoder = self.encoder_for(output, level)?;
        copy(&mut input, &mut encoder)?;
        encoder.flush()?;

        on_progress(ProgressEvent::Entry {
            path: src.clone(),
            index: 0,
        });
        on_progress(ProgressEvent::Finished);
        Ok(())
    }

    fn unpack(
        &self,
        archive: &Path,
        dest: &Path,
        options: &UnpackOptions,
        on_progress: &mut ProgressCallback,
    ) -> Result<()> {
        let out_path = if dest.is_dir() || dest.extension().is_none() {
            std::fs::create_dir_all(dest)?;
            dest.join(Self::output_name(archive))
        } else {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            dest.to_path_buf()
        };

        if out_path.exists() && !options.overwrite {
            return Ok(());
        }

        on_progress(ProgressEvent::Started {
            total_entries: Some(1),
        });

        let input = File::open(archive)?;
        let mut decoder = self.decoder_for(input)?;
        let mut output = File::create(&out_path)?;
        copy(&mut decoder, &mut output)?;

        on_progress(ProgressEvent::Entry {
            path: out_path,
            index: 0,
        });
        on_progress(ProgressEvent::Finished);
        Ok(())
    }

    fn list(&self, archive: &Path) -> Result<Vec<ArchiveEntry>> {
        let meta = std::fs::metadata(archive)?;
        Ok(vec![ArchiveEntry {
            path: Self::output_name(archive),
            // Для одиночных потоков точный uncompressed size часто неизвестен.
            size_bytes: meta.len(),
            is_dir: false,
        }])
    }
}
