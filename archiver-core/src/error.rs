use std::path::PathBuf;
use thiserror::Error;

/// Единая ошибка ядра. Tauri-слой должен маппить эти варианты
/// в понятные пользователю сообщения (см. `impl From<ArchiverError> for String`
/// на стороне UI-обвязки).
#[derive(Debug, Error)]
pub enum ArchiverError {
    #[error("Не удалось определить формат архива: {0}")]
    UnknownFormat(PathBuf),

    #[error("Формат {0} пока не поддерживает эту операцию")]
    UnsupportedOperation(&'static str),

    #[error("Архив защищён паролем, либо введённый пароль неверен")]
    WrongOrMissingPassword,

    #[error("Файл не найден: {0}")]
    FileNotFound(PathBuf),

    #[error("Недостаточно прав для записи в: {0}")]
    PermissionDenied(PathBuf),

    #[error("Ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    #[error("Ошибка ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Ошибка 7z: {0}")]
    SevenZ(String),

    #[error("Ошибка RAR: {0}")]
    Rar(String),

    #[error("Операция отменена пользователем")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, ArchiverError>;
