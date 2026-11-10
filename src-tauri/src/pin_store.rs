//! Where the Cloister PIN's hash lives: `lock.json`, next to `settings.json`.
//!
//! It is kept apart from the settings on purpose: settings are sent to the
//! interface in every snapshot, and the hash never should be.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::persistence::{load_json, save_json};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct LockFile {
    /// Argon2id hash of the PIN in PHC format, or `None` when no PIN is set.
    pin_hash: Option<String>,
}

/// The PIN hash plus the file it is saved in.
#[derive(Debug)]
pub struct PinStore {
    path: PathBuf,
    file: LockFile,
}

impl PinStore {
    /// Loads `lock.json`; a missing file means no PIN is set.
    pub fn load(path: PathBuf) -> AppResult<Self> {
        let file = load_json::<LockFile>(&path)?;
        Ok(Self { path, file })
    }

    /// Whether a PIN has been set.
    pub fn is_set(&self) -> bool {
        self.file.pin_hash.is_some()
    }

    /// The stored hash, if a PIN is set.
    pub fn hash(&self) -> Option<&str> {
        self.file.pin_hash.as_deref()
    }

    /// Saves a new hash, or removes the PIN with `None`.
    pub fn set(&mut self, hash: Option<String>) -> AppResult<()> {
        let next = LockFile { pin_hash: hash };
        save_json(&self.path, &next)?;
        self.file = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_means_no_pin() {
        let path = std::env::temp_dir().join("cloister-pin-missing-test.json");
        let _ = std::fs::remove_file(&path);
        let store = PinStore::load(path).unwrap();
        assert!(!store.is_set());
    }

    #[test]
    fn set_and_clear_survive_reload() {
        let path = std::env::temp_dir().join("cloister-pin-roundtrip-test.json");
        let mut store = PinStore::load(path.clone()).unwrap();
        store.set(Some("$argon2id$example".into())).unwrap();
        assert_eq!(PinStore::load(path.clone()).unwrap().hash(), Some("$argon2id$example"));
        store.set(None).unwrap();
        assert!(!PinStore::load(path.clone()).unwrap().is_set());
        let _ = std::fs::remove_file(&path);
    }
}
