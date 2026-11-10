//! Everything that touches the live app: windows, webviews, the tray,
//! notifications and background work. Decisions are delegated to the pure
//! `services` and `features` modules; this layer only applies them.

pub mod activation;
pub mod away;
pub mod badge;
pub mod capabilities;
pub mod erase;
pub mod hotkeys;
pub mod inject;
pub mod lifecycle;
pub mod lock;
pub mod memory;
pub mod migrate;
pub mod notifications;
pub mod pause;
pub mod refresh;
pub mod startup;
pub mod tray;
pub mod user_agent;
pub mod veil;
pub mod webviews;

use tauri::{App, Manager, Runtime, WindowEvent};

use crate::state::AppState;
use crate::MAIN_LABEL;

/// Wires up the tray, window behaviour and background tasks at startup.
pub fn setup<R: Runtime>(app: &mut App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    tray::create(&handle)?;
    lifecycle::start(handle.clone());
    hotkeys::apply(&handle, handle.state::<AppState>().settings().get().global_shortcuts);

    // The window starts hidden (see tauri.conf.json) so a login launch stays in the tray.
    if !startup::launched_minimized() {
        tray::show_main(&handle);
    }

    if let Some(window) = app.get_window(MAIN_LABEL) {
        crate::platform::prepare_window(&window);
        if let Some(main) = handle.get_webview(MAIN_LABEL) {
            crate::platform::share_engine_from(&main);
        }
        migrate::remove_legacy_profiles(&handle);
        let handle = handle.clone();
        window.on_window_event(move |event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                let close_to_tray = handle.state::<AppState>().settings().get().close_to_tray;
                if close_to_tray {
                    api.prevent_close();
                    if let Some(window) = handle.get_window(MAIN_LABEL) {
                        let _ = window.hide();
                    }
                }
            }
            WindowEvent::Focused(focused) => away::on_focus_changed(&handle, *focused),
            _ => {}
        });
    }
    Ok(())
}
