//! Deleting service data (cookies, logins, caches).
//!
//! Pages first clear their browsing data through the webview itself. That is
//! the only way on macOS (profiles are WebKit data stores) and on Windows
//! (profiles live inside the shared WebView2 engine). Then the page closes and
//! any profile folder Cloister owns is deleted (Linux, and folders left over
//! from Cloister 0.3 on Windows). Browser processes keep files open for a moment
//! after a webview closes, so deletion is retried for a few seconds. Callers
//! run on a background thread (async commands), and the call returns only once
//! the folder is gone, so a page cannot reopen on a half-deleted profile.

use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime};

use super::webviews;
use crate::error::AppResult;
use crate::persistence;
use crate::services::profile;
use crate::state::AppState;

const ATTEMPTS: u32 = 20;
const RETRY_DELAY: Duration = Duration::from_millis(500);

/// Closes a service's page and deletes its browser profile. The service stays
/// in the sidebar and starts signed out next time it opens.
pub fn service_data<R: Runtime>(app: &AppHandle<R>, service_id: &str) -> AppResult<()> {
    clear_in_webview(app, service_id);
    webviews::unload(app, service_id)?;
    let profiles = persistence::profiles_dir(app)?;
    remove_with_retry(&profiles.join(profile::folder_name(service_id)))?;
    remove_with_retry(&profiles.join(service_id))?;
    Ok(())
}

/// Closes every page and deletes every browser profile.
pub fn all_data<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    let ids: Vec<String> = {
        let state = app.state::<AppState>();
        let registry = state.registry();
        registry.all().iter().map(|s| s.id.clone()).collect()
    };
    for id in &ids {
        clear_in_webview(app, id);
        webviews::unload(app, id)?;
    }
    remove_with_retry(&persistence::profiles_dir(app)?)?;
    Ok(())
}

/// Clears a service's cookies and storage through its webview.
///
/// On macOS and Windows this is the only way to reach the data, so a service
/// that is not loaded is loaded briefly first. On Linux, unloaded services are
/// handled by deleting their folder.
fn clear_in_webview<R: Runtime>(app: &AppHandle<R>, service_id: &str) {
    let label = webviews::label_for(service_id);
    let mut webview = app.get_webview(&label);
    if cfg!(any(target_os = "macos", target_os = "windows")) && webview.is_none() {
        // Copy the config out so the registry lock is not held while the page loads.
        let service = {
            let state = app.state::<AppState>();
            let registry = state.registry();
            registry.get(service_id).ok().cloned()
        };
        webview = service.and_then(|service| webviews::ensure_loaded(app, &service).ok());
    }
    if let Some(webview) = webview {
        if let Err(err) = webview.clear_all_browsing_data() {
            log::warn!("could not clear browsing data for {service_id}: {err}");
        }
    }
}

/// Deletes `dir`, retrying while files are still held open. A missing folder is success.
fn remove_with_retry(dir: &Path) -> io::Result<()> {
    let mut attempt = 1;
    loop {
        match std::fs::remove_dir_all(dir) {
            Ok(()) => return Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(err) if attempt >= ATTEMPTS => {
                log::warn!("could not delete {}: {err}", dir.display());
                return Err(err);
            }
            Err(_) => {
                attempt += 1;
                thread::sleep(RETRY_DELAY);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_folders_and_accepts_missing_ones() {
        let dir = std::env::temp_dir().join(format!("cloister-erase-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("EBWebView/Default")).unwrap();
        std::fs::write(dir.join("EBWebView/Default/Cookies"), b"x").unwrap();

        remove_with_retry(&dir).unwrap();
        assert!(!dir.exists());
        remove_with_retry(&dir).unwrap();
    }
}
