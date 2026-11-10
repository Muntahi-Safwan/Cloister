//! Launching with Windows.
//!
//! When "Start with Windows" is on, Cloister is registered to launch at login
//! with `--minimized`, so it starts quietly in the tray.

use tauri::{AppHandle, Runtime};
use tauri_plugin_autostart::ManagerExt;

/// Command-line flag passed by the login item.
pub const MINIMIZED_FLAG: &str = "--minimized";

/// Whether this process was started by the login item.
pub fn launched_minimized() -> bool {
    std::env::args().any(|arg| arg == MINIMIZED_FLAG)
}

/// Turns launch-at-login on or off. Failures are logged, not fatal.
pub fn set_launch_at_login<R: Runtime>(app: &AppHandle<R>, enabled: bool) {
    let autolaunch = app.autolaunch();
    let result = if enabled { autolaunch.enable() } else { autolaunch.disable() };
    if let Err(err) = result {
        log::warn!("could not change launch at login: {err}");
    }
}
