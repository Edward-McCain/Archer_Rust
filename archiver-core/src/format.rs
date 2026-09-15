use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Поддерживаемые форматы архивов.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    Gzip,
    Bzip2,
    Xz,
    SevenZ,
    Rar,
}

impl ArchiveFormat {
    /// Может ли ядро СОЗДАВАТЬ архивы этого формата (не только читать).
    pub fn supports_packing(&self) -> bool {
        !matches!(self, ArchiveFormat::Rar)
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::Tar => "tar",
            ArchiveFormat::TarGz => "tar.gz",
            ArchiveFormat::TarBz2 => "tar.bz2",
            ArchiveFormat::TarXz => "tar.xz",
            ArchiveFormat::Gzip => "gz",
            ArchiveFormat::Bzip2 => "bz2",
            ArchiveFormat::Xz => "xz",
            ArchiveFormat::SevenZ => "7z",
            ArchiveFormat::Rar => "rar",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "ZIP",
            ArchiveFormat::Tar => "TAR",
            ArchiveFormat::TarGz => "TAR.GZ",
            ArchiveFormat::TarBz2 => "TAR.BZ2",
            ArchiveFormat::TarXz => "TAR.XZ",
            ArchiveFormat::Gzip => "GZIP",
            ArchiveFormat::Bzip2 => "BZIP2",
            ArchiveFormat::Xz => "XZ",
            ArchiveFormat::SevenZ => "7-Zip",
            ArchiveFormat::Rar => "RAR",
        }
    }

    /// Определение формата по расширению файла. Это первичный, быстрый путь —
    /// используется для UI-подсказок (иконки, выбор обработчика по умолчанию).
    pub fn from_extension(path: &Path) -> Option<ArchiveFormat> {
        let name = path.file_name()?.to_str()?.to_lowercase();

        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            return Some(ArchiveFormat::TarGz);
        }
        if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
            return Some(ArchiveFormat::TarBz2);
        }
        if name.ends_with(".tar.xz") || name.ends_with(".txz") {
            return Some(ArchiveFormat::TarXz);
        }

        match path.extension()?.to_str()?.to_lowercase().as_str() {
            "zip" => Some(ArchiveFormat::Zip),
            "tar" => Some(ArchiveFormat::Tar),
            "gz" => Some(ArchiveFormat::Gzip),
            "bz2" => Some(ArchiveFormat::Bzip2),
            "xz" => Some(ArchiveFormat::Xz),
            "7z" => Some(ArchiveFormat::SevenZ),
            "rar" => Some(ArchiveFormat::Rar),
            _ => None,
        }
    }

    /// Надёжное определение формата по сигнатуре файла (magic bytes).
    /// Используется как fallback, когда расширение отсутствует, нестандартное
    /// или пользователь мог его переименовать/потерять.
    pub fn detect(path: &Path) -> super::Result<ArchiveFormat> {
        if let Some(fmt) = Self::from_extension(path) {
            return Ok(fmt);
        }

        let mut file = File::open(path).map_err(super::ArchiverError::Io)?;
        let mut buf = [0u8; 8];
        let n = file.read(&mut buf).map_err(super::ArchiverError::Io)?;
        let buf = &buf[..n];

        let fmt = match buf {
            [0x50, 0x4B, 0x03, 0x04, ..] | [0x50, 0x4B, 0x05, 0x06, ..] => ArchiveFormat::Zip,
            [0x1F, 0x8B, ..] => ArchiveFormat::Gzip,
            [0x42, 0x5A, 0x68, ..] => ArchiveFormat::Bzip2,
            [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00, ..] => ArchiveFormat::Xz,
            [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, ..] => ArchiveFormat::SevenZ,
            [0x52, 0x61, 0x72, 0x21, ..] => ArchiveFormat::Rar,
            _ => return Err(super::ArchiverError::UnknownFormat(path.to_path_buf())),
        };

        Ok(fmt)
    }
}
