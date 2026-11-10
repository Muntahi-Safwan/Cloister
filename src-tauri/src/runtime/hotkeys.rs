//! System-wide shortcuts that work even when Cloister is in the background.
//!
//! * `Ctrl+Alt+Space` shows Cloister, or hides it if it is already in front.
//! * `Ctrl+Alt+H` (panic) turns on Veil and hides Cloister instantly.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use super::{tray, veil};
use crate::{snapshot, MAIN_LABEL};

pub const TOGGLE_SHORTCUT: &str = "CommandOrControl+Alt+Space";
pub const PANIC_SHORTCUT: &str = "CommandOrControl+Alt+H";

/// Registers or removes the global shortcuts. A shortcut already taken by
/// another app is logged and skipped rather than treated as an error.
pub fn apply<R: Runtime>(app: &AppHandle<R>, enabled: bool) {
    let shortcuts = app.global_shortcut();
    if let Err(err) = shortcuts.unregister_all() {
        log::warn!("could not clear global shortcuts: {err}");
    }
    if !enabled {
        return;
    }

    let toggle = shortcuts.on_shortcut(TOGGLE_SHORTCUT, |app, _, event| {
        if event.state == ShortcutState::Pressed {
            toggle_window(app);
        }
    });
    let panic = shortcuts.on_shortcut(PANIC_SHORTCUT, |app, _, event| {
        if event.state == ShortcutState::Pressed {
            hide_everything(app);
        }
    });
    for (name, result) in [(TOGGLE_SHORTCUT, toggle), (PANIC_SHORTCUT, panic)] {
        if let Err(err) = result {
            log::warn!("global shortcut {name} is unavailable: {err}");
        }
    }
}

fn toggle_window<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_window(MAIN_LABEL) else { return };
    let in_front = window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false);
    if in_front {
        let _ = window.hide();
    } else {
        tray::show_main(app);
    }
}

/// Veil first, so nothing is readable even if hiding takes a moment.
fn hide_everything<R: Runtime>(app: &AppHandle<R>) {
    veil::set(app, true);
    snapshot::publish(app);
    if let Some(window) = app.get_window(MAIN_LABEL) {
        let _ = window.hide();
    }
}
