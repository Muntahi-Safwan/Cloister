//! Per-service actions from the sidebar's right-click menu and shortcuts:
//! rename, mute, background mode, sleep, zoom and open in the browser.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_opener::OpenerExt;

use crate::error::AppResult;
use crate::features::schedule::{mute_until, MuteDuration};
use crate::runtime::{inject, webviews};
use crate::services::model::{validate_name, ZoomStep};
use crate::services::{BackgroundMode, ServiceId};
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

/// Chooses what a service does while it is not on screen.
#[tauri::command]
pub fn set_service_background<R: Runtime>(
    app: AppHandle<R>,
    id: ServiceId,
    mode: BackgroundMode,
) -> AppResult<Snapshot> {
    let state = app.state::<AppState>();
    state.registry().modify(&id, |s| s.options.background = mode)?;
    // Wake a page that leaves pause mode; the sweep pauses one that enters it.
    if mode != BackgroundMode::Pause && state.session().paused.remove(&id).is_some() {
        if let Some(webview) = app.get_webview(&webviews::label_for(&id)) {
            crate::platform::set_paused(&webview, false);
        }
    }
    webviews::sync_visibility(&app)?;
    Ok(publish(&app))
}

/// Hides or shows the sender and message in a service's notifications.
#[tauri::command]
pub fn set_service_hide_previews<R: Runtime>(
    app: AppHandle<R>,
    id: ServiceId,
    hide: bool,
) -> AppResult<Snapshot> {
    app.state::<AppState>().registry().modify(&id, |s| s.options.hide_previews = hide)?;
    Ok(publish(&app))
}

/// Opens the find bar inside the active service and moves keyboard focus to it.
#[tauri::command]
pub fn find_in_page<R: Runtime>(app: AppHandle<R>) -> AppResult<()> {
    let active = {
        let state = app.state::<AppState>();
        let session = state.session();
        if session.locked {
            return Ok(());
        }
        session.active.clone()
    };
    if let Some(webview) = active.and_then(|id| app.get_webview(&webviews::label_for(&id))) {
        webview.set_focus()?;
        webview.eval(inject::find_script())?;
    }
    Ok(())
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
