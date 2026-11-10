//! The system tray icon and its menu.

use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, EventTarget, Manager, Runtime};

use super::badge;
use crate::{MAIN_LABEL, SHORTCUT_EVENT};

const TRAY_ID: &str = "cloister-tray";
const MENU_SHOW: &str = "show";
const MENU_VEIL: &str = "veil";
const MENU_FOCUS: &str = "focus";
const MENU_LOCK: &str = "lock";
const MENU_QUIT: &str = "quit";

/// Creates the tray icon. Left click shows the window; right click opens the menu.
pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, MENU_SHOW, "Show Cloister", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_VEIL, "Toggle Veil", true, None::<&str>)?,
            &MenuItem::with_id(app, MENU_FOCUS, "Start or end focus", true, None::<&str>)?,
            &MenuItem::with_id(app, MENU_LOCK, "Lock now", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_QUIT, "Quit Cloister", true, None::<&str>)?,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Cloister")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu)
        .on_tray_icon_event(handle_click);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Shows the unread total in the tray tooltip.
pub fn set_unread<R: Runtime>(app: &AppHandle<R>, total: u32) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = match total {
            0 => "Cloister".to_string(),
            1 => "Cloister · 1 unread".to_string(),
            n => format!("Cloister · {n} unread"),
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

/// Brings the main window to the front.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_window(MAIN_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    // Hiding the window removes its taskbar button, and the overlay badge with it.
    badge::refresh(app);
}

fn handle_menu<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    match event.id().as_ref() {
        MENU_SHOW => show_main(app),
        MENU_VEIL => forward_action(app, "veil"),
        MENU_FOCUS => forward_action(app, "focus"),
        MENU_LOCK => super::lock::lock(app),
        MENU_QUIT => app.exit(0),
        _ => {}
    }
}

fn handle_click<R: Runtime>(tray: &TrayIcon<R>, event: TrayIconEvent) {
    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event
    {
        show_main(tray.app_handle());
    }
}

/// Lets the UI handle tray actions exactly like keyboard shortcuts.
fn forward_action<R: Runtime>(app: &AppHandle<R>, action: &str) {
    let _ = app.emit_to(EventTarget::webview(MAIN_LABEL), SHORTCUT_EVENT, action);
}
