use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Window layer: {0}")]
    WindowLayer(String),
    #[error("Updater: {0}")]
    Updater(String),
    #[error("Validation: {0}")]
    Validation(String),
    #[error("OAuth: {0}")]
    OAuth(String),
    #[error("Media: {0}")]
    Media(String),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}

// Serialize as string for backwards compatibility — frontend already handles string errors.
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
