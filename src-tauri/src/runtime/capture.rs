//! Hiding Cloister from screen sharing and screen recording.
//!
//! The operating system does the work: on Windows the window is excluded from
//! capture (Windows 10 version 2004 and later), and on macOS it is marked as
//! not shareable, which older capture tools respect. Video calls and recorders
//! then show a blank area where Cloister is. Linux has no such switch.

use tauri::{AppHandle, Manager, Runtime};

use crate::state::AppState;
use crate::MAIN_LABEL;

/// Whether this system can hide a window from capture.
pub const SUPPORTED: bool = cfg!(any(target_os = "windows", target_os = "macos"));

/// Whether Cloister should be hidden right now: always, by setting, or while presenting.
pub fn wanted<R: Runtime>(app: &AppHandle<R>) -> bool {
    let state = app.state::<AppState>();
    let always = state.settings().get().hide_from_capture;
    always || state.session().presenting.is_some()
}

/// Applies the current choice to the main window.
pub fn apply<R: Runtime>(app: &AppHandle<R>) {
    if !SUPPORTED {
        return;
    }
    let hide = wanted(app);
    if let Some(window) = app.get_webview_window(MAIN_LABEL) {
        if let Err(err) = window.set_content_protected(hide) {
            log::warn!("could not change screen-capture protection: {err}");
        }
    }
}
