use crate::error::ArchiverError;
use crate::types::{
    ArchiveEntry, Archiver, PackOptions, ProgressCallback, ProgressEvent, UnpackOptions,
};
use crate::Result;
use std::path::{Path, PathBuf};
use unrar::Archive;

/// RAR: только распаковка и листинг (формат проприетарный, запись не поддерживается).
pub struct RarArchiver;

impl RarArchiver {
    fn map_err(err: unrar::error::UnrarError) -> ArchiverError {
        use unrar::error::Code;
        match err.code {
            Code::BadPassword | Code::MissingPassword => ArchiverError::WrongOrMissingPassword,
            _ => ArchiverError::Rar(err.to_string()),
        }
    }
}

impl Archiver for RarArchiver {
    fn pack(
        &self,
        _sources: &[PathBuf],
        _dest: &Path,
        _options: &PackOptions,
        _on_progress: &mut ProgressCallback,
    ) -> Result<()> {
        Err(ArchiverError::UnsupportedOperation(
            "RAR: создание архивов не поддерживается (проприетарный формат)",
        ))
    }

    fn unpack(
        &self,
        archive: &Path,
        dest: &Path,
        options: &UnpackOptions,
        on_progress: &mut ProgressCallback,
    ) -> Result<()> {
        std::fs::create_dir_all(dest)?;

        let password_owned = options.password.clone();
        let opened = match password_owned.as_ref() {
            Some(pw) => Archive::with_password(archive, pw.as_bytes()).open_for_processing(),
            None => Archive::new(archive).open_for_processing(),
        }
        .map_err(Self::map_err)?;

        on_progress(ProgressEvent::Started {
            total_entries: None,
        });

        let mut archive = opened;
        let mut index = 0u64;

        loop {
            let header = match archive.read_header() {
                Ok(Some(h)) => h,
                Ok(None) => break,
                Err(err) => return Err(Self::map_err(err)),
            };

            let entry_path = header.entry().filename.clone();
            let out_path = dest.join(&entry_path);

            if out_path.exists() && !options.overwrite && header.entry().is_file() {
                archive = header.skip().map_err(Self::map_err)?;
            } else {
                archive = header
                    .extract_with_base(dest)
                    .map_err(Self::map_err)?;
            }

            on_progress(ProgressEvent::Entry {
                path: entry_path,
                index,
            });
            index += 1;
        }

        on_progress(ProgressEvent::Finished);
        Ok(())
    }

    fn list(&self, archive: &Path) -> Result<Vec<ArchiveEntry>> {
        let listing = Archive::new(archive)
            .open_for_listing()
            .map_err(Self::map_err)?;

        let mut entries = Vec::new();
        for item in listing {
            let header = item.map_err(Self::map_err)?;
            entries.push(ArchiveEntry {
                path: header.filename.to_string_lossy().to_string(),
                size_bytes: header.unpacked_size,
                is_dir: header.is_directory(),
            });
        }
        Ok(entries)
    }
}
