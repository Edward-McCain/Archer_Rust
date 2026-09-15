use crate::error::ArchiverError;
use crate::types::{Archiver, ArchiveEntry, PackOptions, ProgressCallback, ProgressEvent, UnpackOptions};
use crate::Result;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::unstable::write::FileOptionsExt;
use zip::write::SimpleFileOptions;
use zip::ZipArchive;

pub struct ZipArchiver;

impl Archiver for ZipArchiver {
    fn pack(
        &self,
        sources: &[PathBuf],
        dest: &Path,
        options: &PackOptions,
        on_progress: &mut ProgressCallback,
    ) -> Result<()> {
        let file = File::create(dest)?;
        let mut writer = zip::ZipWriter::new(file);

        let level = options
            .level
            .map(|l| l.as_deflate_level())
            .unwrap_or(6);

        let mut file_options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(level));

        if let Some(password) = &options.password {
            file_options = file_options.with_deprecated_encryption(password.as_bytes());
        }

        // Собираем плоский список файлов, разворачивая директории.
        let mut all_entries: Vec<PathBuf> = Vec::new();
        for src in sources {
            if src.is_dir() {
                for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
                    all_entries.push(entry.path().to_path_buf());
                }
            } else {
                all_entries.push(src.clone());
            }
        }

        on_progress(ProgressEvent::Started {
            total_entries: Some(all_entries.len() as u64),
        });

        for (idx, entry_path) in all_entries.iter().enumerate() {
            // Имя внутри архива — относительно родителя первого источника,
            // чтобы сохранить структуру папок.
            let base_parent = sources
                .first()
                .and_then(|p| p.parent())
                .unwrap_or_else(|| Path::new(""));
            let name_in_zip = entry_path
                .strip_prefix(base_parent)
                .unwrap_or(entry_path)
                .to_string_lossy()
                .replace('\\', "/");

            if entry_path.is_dir() {
                writer.add_directory(format!("{name_in_zip}/"), file_options)?;
            } else {
                writer.start_file(name_in_zip, file_options)?;
                let mut f = File::open(entry_path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                writer.write_all(&buf)?;
            }

            on_progress(ProgressEvent::Entry {
                path: entry_path.clone(),
                index: idx as u64,
            });
        }

        writer.finish()?;
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
        let file = File::open(archive)?;
        let mut zip = ZipArchive::new(file)?;

        on_progress(ProgressEvent::Started {
            total_entries: Some(zip.len() as u64),
        });

        for i in 0..zip.len() {
            let mut entry = match &options.password {
                Some(pw) => zip
                    .by_index_decrypt(i, pw.as_bytes())
                    .map_err(|_| ArchiverError::WrongOrMissingPassword)?,
                None => zip
                    .by_index(i)
                    .map_err(|_| ArchiverError::WrongOrMissingPassword)?,
            };

            let out_path = match entry.enclosed_name() {
                Some(p) => dest.join(p),
                None => continue, // пропускаем потенциально небезопасные пути
            };

            if entry.name().ends_with('/') {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                if out_path.exists() && !options.overwrite {
                    continue;
                }
                let mut out_file = File::create(&out_path)?;
                std::io::copy(&mut entry, &mut out_file)?;
            }

            on_progress(ProgressEvent::Entry {
                path: out_path,
                index: i as u64,
            });
        }

        on_progress(ProgressEvent::Finished);
        Ok(())
    }

    fn list(&self, archive: &Path) -> Result<Vec<ArchiveEntry>> {
        let file = File::open(archive)?;
        let mut zip = ZipArchive::new(file)?;
        let mut entries = Vec::with_capacity(zip.len());

        for i in 0..zip.len() {
            let entry = zip.by_index(i)?;
            entries.push(ArchiveEntry {
                path: entry.name().to_string(),
                size_bytes: entry.size(),
                is_dir: entry.name().ends_with('/'),
            });
        }

        Ok(entries)
    }
}
