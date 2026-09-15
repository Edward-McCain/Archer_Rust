use crate::error::ArchiverError;
use crate::types::{
    ArchiveEntry, Archiver, PackOptions, ProgressCallback, ProgressEvent, UnpackOptions,
};
use crate::Result;
use sevenz_rust::{
    Password, SevenZArchiveEntry, SevenZMethod, SevenZReader, SevenZWriter, AesEncoderOptions,
};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub struct SevenZArchiver;

impl SevenZArchiver {
    fn password_from(options_password: &Option<String>) -> Password {
        match options_password {
            Some(pw) if !pw.is_empty() => Password::from(pw.as_str()),
            _ => Password::empty(),
        }
    }

    fn map_err(err: sevenz_rust::Error) -> ArchiverError {
        let msg = err.to_string();
        let lower = msg.to_lowercase();
        if lower.contains("password") || lower.contains("encrypted") {
            ArchiverError::WrongOrMissingPassword
        } else {
            ArchiverError::SevenZ(msg)
        }
    }
}

impl Archiver for SevenZArchiver {
    fn pack(
        &self,
        sources: &[PathBuf],
        dest: &Path,
        options: &PackOptions,
        on_progress: &mut ProgressCallback,
    ) -> Result<()> {
        if sources.is_empty() {
            return Err(ArchiverError::UnsupportedOperation(
                "7z: нужен хотя бы один файл или папка",
            ));
        }

        if let Some(parent) = dest.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Один источник — готовые хелперы крейта (корректные имена записей).
        if sources.len() == 1 {
            let src = &sources[0];
            if !src.exists() {
                return Err(ArchiverError::FileNotFound(src.clone()));
            }
            on_progress(ProgressEvent::Started {
                total_entries: Some(1),
            });
            let result = match &options.password {
                Some(pw) if !pw.is_empty() => sevenz_rust::compress_to_path_encrypted(
                    src,
                    dest,
                    Password::from(pw.as_str()),
                ),
                _ => sevenz_rust::compress_to_path(src, dest),
            };
            result.map_err(Self::map_err)?;
            on_progress(ProgressEvent::Entry {
                path: src.clone(),
                index: 0,
            });
            on_progress(ProgressEvent::Finished);
            return Ok(());
        }

        let file = File::create(dest)?;
        let mut writer = SevenZWriter::new(file).map_err(Self::map_err)?;

        if let Some(password) = &options.password {
            if !password.is_empty() {
                writer.set_content_methods(vec![
                    AesEncoderOptions::new(Password::from(password.as_str())).into(),
                    SevenZMethod::LZMA2.into(),
                ]);
            }
        }

        on_progress(ProgressEvent::Started {
            total_entries: Some(sources.len() as u64),
        });

        for (idx, src) in sources.iter().enumerate() {
            if !src.exists() {
                return Err(ArchiverError::FileNotFound(src.clone()));
            }

            let name = src
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| format!("item-{idx}"));

            if src.is_dir() {
                writer
                    .push_source_path_non_solid(src, |_| true)
                    .map_err(Self::map_err)?;
            } else {
                let entry = SevenZArchiveEntry::from_path(src, name);
                let f = File::open(src)?;
                writer
                    .push_archive_entry(entry, Some(f))
                    .map_err(Self::map_err)?;
            }

            on_progress(ProgressEvent::Entry {
                path: src.clone(),
                index: idx as u64,
            });
        }

        writer.finish().map_err(ArchiverError::Io)?;
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
        std::fs::create_dir_all(dest)?;
        let password = Self::password_from(&options.password);
        let mut index = 0u64;

        let file = File::open(archive)?;
        let result = if password.is_empty() {
            sevenz_rust::decompress_with_extract_fn(file, dest, |entry, reader, path| {
                on_progress(ProgressEvent::Entry {
                    path: path.clone(),
                    index,
                });
                index += 1;
                if path.exists() && !options.overwrite && !entry.is_directory() {
                    return Ok(false);
                }
                sevenz_rust::default_entry_extract_fn(entry, reader, path)
            })
        } else {
            sevenz_rust::decompress_with_extract_fn_and_password(
                file,
                dest,
                password,
                |entry, reader, path| {
                    on_progress(ProgressEvent::Entry {
                        path: path.clone(),
                        index,
                    });
                    index += 1;
                    if path.exists() && !options.overwrite && !entry.is_directory() {
                        return Ok(false);
                    }
                    sevenz_rust::default_entry_extract_fn(entry, reader, path)
                },
            )
        };

        result.map_err(Self::map_err)?;
        on_progress(ProgressEvent::Finished);
        Ok(())
    }

    fn list(&self, archive: &Path) -> Result<Vec<ArchiveEntry>> {
        let mut file = File::open(archive)?;
        let pos = file.stream_position()?;
        let len = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(pos))?;

        let reader = SevenZReader::new(file, len, Password::empty()).map_err(Self::map_err)?;
        Ok(reader
            .archive()
            .files
            .iter()
            .map(|e: &SevenZArchiveEntry| ArchiveEntry {
                path: e.name().to_string(),
                size_bytes: e.size,
                is_dir: e.is_directory(),
            })
            .collect())
    }
}
