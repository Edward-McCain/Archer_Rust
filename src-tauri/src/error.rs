use archiver_core::ArchiverError;

/// Map core errors to short, user-facing Russian messages for the UI.
pub fn user_message(err: ArchiverError) -> String {
    match err {
        ArchiverError::UnknownFormat(path) => {
            format!(
                "Не удалось определить формат архива: {}",
                path.display()
            )
        }
        ArchiverError::UnsupportedOperation(op) => {
            format!("Операция недоступна: {op}")
        }
        ArchiverError::WrongOrMissingPassword => {
            "Архив защищён паролем или введён неверный пароль.".to_string()
        }
        ArchiverError::FileNotFound(path) => {
            format!("Файл не найден: {}", path.display())
        }
        ArchiverError::PermissionDenied(path) => {
            format!("Недостаточно прав для записи в: {}", path.display())
        }
        ArchiverError::Io(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                format!("Нет прав доступа: {e}")
            } else if e.kind() == std::io::ErrorKind::NotFound {
                format!("Путь не найден: {e}")
            } else {
                format!("Ошибка файловой системы: {e}")
            }
        }
        ArchiverError::Zip(e) => format!("Ошибка ZIP: {e}"),
        ArchiverError::SevenZ(e) => format!("Ошибка 7-Zip: {e}"),
        ArchiverError::Rar(e) => format!("Ошибка RAR: {e}"),
        ArchiverError::Cancelled => "Операция отменена.".to_string(),
    }
}

pub type CommandResult<T> = Result<T, String>;

pub fn map_err(err: ArchiverError) -> String {
    user_message(err)
}
