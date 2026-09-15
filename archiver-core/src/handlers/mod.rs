pub mod zip_handler;
pub mod tar_handler;

pub use zip_handler::ZipArchiver;
pub use tar_handler::TarArchiver;

// TODO: sevenz_handler (крейт sevenz-rust) и rar_handler (крейт unrar,
// только unpack/list — RAR-формат проприетарный и запись не лицензирована)
// реализуются по тому же трейту `Archiver`, см. zip_handler.rs как образец.
