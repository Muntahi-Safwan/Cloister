//! One-time clean-ups when Cloister's storage layout changes.

use std::fs;
use std::path::Path;
use std::thread;

use tauri::{AppHandle, Runtime};

use crate::persistence;

/// Marker written once the per-service browser folders of Cloister 0.3 and
/// earlier have been removed.
const SHARED_ENGINE_MARKER: &str = ".shared-engine";

/// Windows: since 0.4 every service is a profile inside one shared WebView2
/// engine, so the old per-service browser folders are no longer read. Their
/// logins cannot be carried over (each folder encrypts its cookies with its
/// own key), so they are deleted once, in the background, to free the disk.
pub fn remove_legacy_profiles<R: Runtime>(app: &AppHandle<R>) {
    if !cfg!(target_os = "windows") {
        return;
    }
    let Ok(profiles) = persistence::profiles_dir(app) else { return };
    let marker = profiles.join(SHARED_ENGINE_MARKER);
    if marker.exists() {
        return;
    }
    let _ = thread::Builder::new().name("cloister-migrate".into()).spawn(move || {
        let removed = remove_legacy_folders(&profiles);
        log::info!("removed {removed} browser folders from before the shared engine");
        let _ = fs::write(&marker, b"");
    });
}

/// Deletes every sub-folder that is not a shared-engine profile (`svc-*`).
fn remove_legacy_folders(profiles: &Path) -> usize {
    let Ok(entries) = fs::read_dir(profiles) else { return 0 };
    entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with("svc-"))
        .filter(|entry| fs::remove_dir_all(entry.path()).is_ok())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_shared_engine_profiles_and_files() {
        let dir = std::env::temp_dir().join(format!("cloister-migrate-{}", std::process::id()));
        fs::create_dir_all(dir.join("whatsapp-abc/EBWebView")).unwrap();
        fs::create_dir_all(dir.join("svc-discord-xyz")).unwrap();
        fs::write(dir.join("notes.txt"), b"x").unwrap();

        assert_eq!(remove_legacy_folders(&dir), 1);
        assert!(!dir.join("whatsapp-abc").exists());
        assert!(dir.join("svc-discord-xyz").exists());
        assert!(dir.join("notes.txt").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
