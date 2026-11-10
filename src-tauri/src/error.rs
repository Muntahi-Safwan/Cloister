//! Application-wide error type.
//!
//! Every fallible operation in the backend returns [`AppResult`]. Errors are
//! serialised as plain strings so they can cross the IPC boundary and be shown
//! to the user by the frontend.

use serde::{Serialize, Serializer};

/// Errors that can occur anywhere in the Cloister backend.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A service id did not match any configured service.
    #[error("service `{0}` was not found")]
    ServiceNotFound(String),

    /// User input failed validation (bad URL, empty name, ...).
    #[error("{0}")]
    InvalidInput(String),

    /// A catalog recipe id did not match any built-in recipe.
    #[error("recipe `{0}` is not in the catalog")]
    RecipeNotFound(String),

    /// The main window is missing (should only happen during shutdown).
    #[error("the main window is not available")]
    MainWindowMissing,

    /// Reading or writing a file on disk failed.
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),

    /// A persisted file could not be parsed or serialised.
    #[error("data error: {0}")]
    Json(#[from] serde_json::Error),

    /// An error reported by Tauri itself.
    #[error("runtime error: {0}")]
    Tauri(#[from] tauri::Error),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// Convenience alias used throughout the crate.
pub type AppResult<T> = Result<T, AppError>;
