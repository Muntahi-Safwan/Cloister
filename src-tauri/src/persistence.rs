//! Small helpers for reading and writing JSON files in the app's config folder.
//!
//! Writes are atomic: data is written to a temporary file first and then
//! renamed over the target, so a crash mid-write never corrupts user data.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Serialize};
use tauri::{AppHandle, Manager, Runtime};

use crate::error::AppResult;

/// Returns the directory where Cloister stores its configuration, creating it if needed.
pub fn config_dir<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
    let dir = app.path().app_config_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns the directory that holds one isolated browser profile per service.
pub fn profiles_dir<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
    let dir = app.path().app_local_data_dir()?.join("profiles");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Loads a JSON file, returning `T::default()` when the file does not exist yet.
pub fn load_json<T: DeserializeOwned + Default>(path: &Path) -> AppResult<T> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(serde_json::from_str(&text)?),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(err) => Err(err.into()),
    }
}

/// Saves a value as pretty-printed JSON using an atomic write.
pub fn save_json<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(value)?)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Sample {
        name: String,
        count: u32,
    }

    #[test]
    fn missing_file_yields_default() {
        let path = std::env::temp_dir().join("cloister-missing-file-test.json");
        let _ = fs::remove_file(&path);
        let value: Sample = load_json(&path).unwrap();
        assert_eq!(value, Sample::default());
    }

    #[test]
    fn round_trips_through_disk() {
        let path = std::env::temp_dir().join("cloister-roundtrip-test.json");
        let value = Sample { name: "cloister".into(), count: 3 };
        save_json(&path, &value).unwrap();
        let loaded: Sample = load_json(&path).unwrap();
        assert_eq!(loaded, value);
        let _ = fs::remove_file(&path);
    }
}
