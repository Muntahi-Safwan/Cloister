//! Cloister: a quiet, private, lightweight home for your messaging apps.
//!
//! # Architecture
//!
//! * [`services`]: what a service is (config, catalog, persistence, pure rules).
//! * [`features`]: product logic with no Tauri dependency (focus, notification routing).
//! * [`runtime`]: applies decisions to the live app (webviews, tray, badges).
//! * [`commands`]: the IPC surface for the UI and for service pages.
//! * [`snapshot`]: the single view model pushed to the UI.
//!
//! See `docs/architecture.md` for the full picture.

pub mod commands;
pub mod error;
pub mod features;
pub mod persistence;
pub mod pin_store;
pub mod platform;
pub mod runtime;
pub mod services;
pub mod settings;
pub mod snapshot;
pub mod state;

use tauri::Manager;

use pin_store::PinStore;
use services::ServiceRegistry;
use settings::SettingsStore;
use state::AppState;

/// Label of the main window and of its UI webview.
pub const MAIN_LABEL: &str = "main";

/// Event the UI listens to for keyboard shortcuts and tray actions.
pub const SHORTCUT_EVENT: &str = "cloister://shortcut";

/// Builds and runs the application.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            runtime::tray::show_main(app);
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Info })
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::Builder::new().args([runtime::startup::MINIMIZED_FLAG]).build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let config_dir = persistence::config_dir(app.handle())?;
            let registry = ServiceRegistry::load(config_dir.join("services.json"))?;
            let settings = SettingsStore::load(config_dir.join("settings.json"))?;
            let pin = PinStore::load(config_dir.join("lock.json"))?;
            let hello_available = platform::authentication_available();
            app.manage(AppState::new(registry, settings, pin, hello_available));
            runtime::setup(app)
        })
        .invoke_handler(tauri::generate_handler![
            commands::services::get_snapshot,
            commands::services::list_catalog,
            commands::services::add_service,
            commands::services::remove_service,
            commands::services::reorder_services,
            commands::services::activate_service,
            commands::services::reload_service,
            commands::services::set_service_icon,
            commands::service_actions::rename_service,
            commands::service_actions::mute_service,
            commands::service_actions::sleep_service,
            commands::service_actions::set_service_background,
            commands::service_actions::set_service_hide_previews,
            commands::service_actions::find_in_page,
            commands::service_actions::zoom_service,
            commands::service_actions::open_service_externally,
            commands::session::set_content_bounds,
            commands::session::set_overlay_open,
            commands::session::set_veil,
            commands::session::start_focus,
            commands::session::end_focus,
            commands::session::update_settings,
            commands::session::send_test_notification,
            commands::privacy::lock_now,
            commands::privacy::unlock,
            commands::privacy::unlock_with_pin,
            commands::privacy::set_pin,
            commands::privacy::remove_pin,
            commands::privacy::get_history,
            commands::privacy::clear_history,
            commands::privacy::service_memory,
            commands::privacy::erase_service_data,
            commands::privacy::erase_all_data,
            commands::bridge::bridge_notify,
            commands::bridge::bridge_icon,
            commands::bridge::bridge_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Cloister");
}
