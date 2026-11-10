//! Commands for layout, Veil, focus sessions and settings.

use tauri::{AppHandle, Manager, Runtime};

use crate::error::AppResult;
use crate::features::focus::{FocusSession, HeldNotification};
use crate::runtime::{hotkeys, notifications, startup, veil, webviews};
use crate::settings::AppSettings;
use crate::snapshot::{self, Snapshot};
use crate::state::{now_ms, AppState, ContentBounds};

/// The UI reports where service pages should be drawn whenever its layout changes.
#[tauri::command]
pub fn set_content_bounds<R: Runtime>(app: AppHandle<R>, bounds: ContentBounds) -> AppResult<()> {
    let first_time = app.state::<AppState>().session().bounds.replace(bounds).is_none();
    webviews::apply_bounds(&app, bounds)?;
    if first_time {
        webviews::sync_visibility(&app)?;
    }
    Ok(())
}

/// Hides the active page while a dialog is open.
///
/// Service pages are native child webviews that always draw above the main
/// UI, so dialogs would otherwise appear underneath them.
#[tauri::command]
pub fn set_overlay_open<R: Runtime>(app: AppHandle<R>, open: bool) -> AppResult<()> {
    app.state::<AppState>().session().overlay_open = open;
    webviews::sync_visibility(&app)?;
    if !open {
        webviews::focus_active(&app);
    }
    Ok(())
}

/// Turns Veil on or off everywhere.
#[tauri::command]
pub fn set_veil<R: Runtime>(app: AppHandle<R>, on: bool) -> Snapshot {
    veil::set(&app, on);
    publish(&app)
}

/// Starts a focus session. Uses the default length from settings when `minutes` is omitted.
#[tauri::command]
pub fn start_focus<R: Runtime>(app: AppHandle<R>, minutes: Option<u32>) -> Snapshot {
    let state = app.state::<AppState>();
    let default_minutes = state.settings().get().focus_minutes;
    let minutes = minutes.unwrap_or(default_minutes).clamp(1, 180);
    state.session().focus = Some(FocusSession::start(now_ms(), minutes));
    publish(&app)
}

/// Ends the focus session and returns the notifications it held.
#[tauri::command]
pub fn end_focus<R: Runtime>(app: AppHandle<R>) -> Vec<HeldNotification> {
    let held = app.state::<AppState>().session().focus.take().map(FocusSession::finish).unwrap_or_default();
    snapshot::publish(&app);
    held
}

/// Saves new settings and applies the ones that affect the running app.
#[tauri::command]
pub fn update_settings<R: Runtime>(app: AppHandle<R>, mut settings: AppSettings) -> AppResult<Snapshot> {
    let (before, after) = {
        let state = app.state::<AppState>();
        // App lock needs a PIN, so there is always a way back in.
        settings.app_lock &= state.can_lock();
        let mut store = state.settings();
        let before = store.get().clone();
        (before, store.replace(settings)?)
    };
    if before.global_shortcuts != after.global_shortcuts {
        hotkeys::apply(&app, after.global_shortcuts);
    }
    if before.launch_at_login != after.launch_at_login {
        startup::set_launch_at_login(&app, after.launch_at_login);
    }
    webviews::sync_visibility(&app)?;
    Ok(publish(&app))
}

/// Shows a sample toast so users can check Windows notification settings.
#[tauri::command]
pub fn send_test_notification<R: Runtime>(app: AppHandle<R>) {
    notifications::show_toast(
        &app,
        "Cloister",
        "Notifications are working. New messages will appear like this.",
    );
}

fn publish<R: Runtime>(app: &AppHandle<R>) -> Snapshot {
    snapshot::publish(app);
    snapshot::build(app)
}
