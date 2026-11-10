//! Shows the total unread count on the taskbar and tray.

use tauri::{AppHandle, Manager, Runtime};

use super::tray;
use crate::state::AppState;
use crate::MAIN_LABEL;

/// Updates the taskbar badge and tray tooltip from the current unread counts.
pub fn refresh<R: Runtime>(app: &AppHandle<R>) {
    let total = app.state::<AppState>().session().unread_total();

    if let Some(window) = app.get_window(MAIN_LABEL) {
        if let Err(err) = set_taskbar_badge(&window, total) {
            log::debug!("taskbar badge not updated: {err}");
        }
    }
    tray::set_unread(app, total);
}

/// Windows has no numeric taskbar badge; it shows a small overlay icon instead.
#[cfg(target_os = "windows")]
fn set_taskbar_badge<R: Runtime>(window: &tauri::Window<R>, total: u32) -> tauri::Result<()> {
    const BADGE_PNG: &[u8] = include_bytes!("../../icons/badge.png");
    let icon = if total > 0 { Some(tauri::image::Image::from_bytes(BADGE_PNG)?) } else { None };
    window.set_overlay_icon(icon)
}

/// macOS and Linux support a numeric badge on the dock or launcher icon.
#[cfg(not(target_os = "windows"))]
fn set_taskbar_badge<R: Runtime>(window: &tauri::Window<R>, total: u32) -> tauri::Result<()> {
    window.set_badge_count((total > 0).then_some(i64::from(total)))
}
