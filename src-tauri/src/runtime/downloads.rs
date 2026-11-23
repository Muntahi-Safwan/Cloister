//! Saves files that service pages download, and opens them afterwards.
//!
//! Every download goes to the user's Downloads folder without a prompt, the
//! way a browser does by default. Progress and the result show in Cloister's
//! title bar instead of the engine's own download UI.

use std::path::{Path, PathBuf};

use tauri::webview::DownloadEvent;
use tauri::{AppHandle, Manager, Runtime, Webview};
use tauri_plugin_opener::OpenerExt;

use crate::error::{AppError, AppResult};
use crate::features::downloads::{self, DownloadState};
use crate::services::ServiceId;
use crate::snapshot;
use crate::state::{now_ms, AppState};

/// The download handler for one service's webview.
pub fn handler<R: Runtime>(
    app: AppHandle<R>,
    service_id: ServiceId,
) -> impl Fn(Webview<R>, DownloadEvent<'_>) -> bool + Send + Sync + 'static {
    move |_webview, event| {
        match event {
            DownloadEvent::Requested { url, destination } => {
                let fallback = downloads_dir(&app);
                *destination = downloads::destination(destination, &fallback, Path::exists);
                log::info!("{service_id}: saving a download to {}", destination.display());
                app.state::<AppState>().session().downloads.start(
                    &service_id,
                    url.as_str(),
                    destination,
                    now_ms(),
                );
            }
            DownloadEvent::Finished { url, path, success } => {
                if !success {
                    log::warn!("{service_id}: a download failed");
                }
                app.state::<AppState>().session().downloads.finish(
                    &service_id,
                    url.as_str(),
                    path.as_deref(),
                    success,
                );
            }
            _ => {}
        }
        snapshot::publish(&app);
        true
    }
}

/// Opens a finished download with its default app. Files that would run code
/// are shown in their folder instead.
pub fn open<R: Runtime>(app: &AppHandle<R>, id: u64) -> AppResult<()> {
    let (path, risky) = finished(app, id)?;
    if risky {
        return reveal_path(app, &path);
    }
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|err| AppError::InvalidInput(format!("Could not open the file: {err}")))
}

/// Shows a download in the system file manager.
pub fn reveal<R: Runtime>(app: &AppHandle<R>, id: u64) -> AppResult<()> {
    let (path, _) = finished(app, id)?;
    reveal_path(app, &path)
}

fn reveal_path<R: Runtime>(app: &AppHandle<R>, path: &Path) -> AppResult<()> {
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|err| AppError::InvalidInput(format!("Could not show the file: {err}")))
}

/// The path of a finished download that still exists. Only paths Cloister
/// saved itself can be opened, never one sent from the UI.
fn finished<R: Runtime>(app: &AppHandle<R>, id: u64) -> AppResult<(PathBuf, bool)> {
    let item = app
        .state::<AppState>()
        .session()
        .downloads
        .get(id)
        .cloned()
        .ok_or_else(|| AppError::InvalidInput("That download is no longer in the list.".into()))?;
    if item.state != DownloadState::Done {
        return Err(AppError::InvalidInput("That download has not finished.".into()));
    }
    if !item.path.exists() {
        return Err(AppError::InvalidInput(format!("{} was moved or deleted.", item.file_name)));
    }
    Ok((item.path, item.risky))
}

fn downloads_dir<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    let path = app.path();
    path.download_dir().or_else(|_| path.home_dir()).unwrap_or_else(|_| std::env::temp_dir())
}
