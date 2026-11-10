//! Per-service actions from the sidebar's right-click menu and shortcuts:
//! rename, mute, sleep, zoom and open in the browser.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_opener::OpenerExt;

use crate::error::AppResult;
use crate::features::schedule::{mute_until, MuteDuration};
use crate::runtime::webviews;
use crate::services::model::{validate_name, ZoomStep};
use crate::services::ServiceId;
use crate::snapshot::{self, Snapshot};
use crate::state::AppState;

/// Renames a service, e.g. "WhatsApp" to "WhatsApp · Work".
#[tauri::command]
pub fn rename_service<R: Runtime>(app: AppHandle<R>, id: ServiceId, name: String) -> AppResult<Snapshot> {
    let name = validate_name(&name)?;
    app.state::<AppState>().registry().modify(&id, |s| s.name = name)?;
    Ok(publish(&app))
}

/// Mutes or unmutes a service's notifications. Unread badges keep counting.
#[tauri::command]
pub fn mute_service<R: Runtime>(
    app: AppHandle<R>,
    id: ServiceId,
    duration: MuteDuration,
) -> AppResult<Snapshot> {
    let until = mute_until(duration, &chrono::Local::now());
    app.state::<AppState>().registry().modify(&id, |s| s.muted_until = until)?;
    Ok(publish(&app))
}

/// Closes a service's page right away to free its memory. Opening it reloads it.
#[tauri::command]
pub fn sleep_service<R: Runtime>(app: AppHandle<R>, id: ServiceId) -> AppResult<Snapshot> {
    webviews::unload(&app, &id)?;
    Ok(publish(&app))
}

/// Changes a service's page zoom and remembers it.
#[tauri::command]
pub fn zoom_service<R: Runtime>(app: AppHandle<R>, id: ServiceId, step: ZoomStep) -> AppResult<Snapshot> {
    let updated = app.state::<AppState>().registry().modify(&id, |s| s.zoom = step.apply(s.zoom))?;
    if let Some(webview) = app.get_webview(&webviews::label_for(&id)) {
        webview.set_zoom(updated.zoom)?;
    }
    Ok(publish(&app))
}

/// Opens the page the service is currently showing (or its start page) in the default browser.
#[tauri::command]
pub fn open_service_externally<R: Runtime>(app: AppHandle<R>, id: ServiceId) -> AppResult<()> {
    let start_url = app.state::<AppState>().registry().get(&id)?.url.clone();
    let current = app.get_webview(&webviews::label_for(&id)).and_then(|w| w.url().ok());
    let url = current.map_or(start_url, |u| u.to_string());
    app.opener().open_url(url, None::<&str>).map_err(|e| crate::error::AppError::InvalidInput(e.to_string()))
}

fn publish<R: Runtime>(app: &AppHandle<R>) -> Snapshot {
    snapshot::publish(app);
    snapshot::build(app)
}
