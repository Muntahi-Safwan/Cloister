//! Build script: generates Tauri context and permissions for app commands.
//!
//! Every `#[tauri::command]` must be listed here. Tauri generates an
//! `allow-<command>` permission for each one, which capabilities then grant.

const COMMANDS: &[&str] = &[
    // Called by Cloister's own UI.
    "get_snapshot",
    "list_catalog",
    "add_service",
    "remove_service",
    "reorder_services",
    "activate_service",
    "reload_service",
    "set_service_icon",
    "rename_service",
    "mute_service",
    "sleep_service",
    "set_service_background",
    "set_service_hide_previews",
    "find_in_page",
    "zoom_service",
    "open_service_externally",
    "set_content_bounds",
    "set_overlay_open",
    "set_veil",
    "start_focus",
    "end_focus",
    "update_settings",
    "send_test_notification",
    "lock_now",
    "unlock",
    "unlock_with_pin",
    "set_pin",
    "remove_pin",
    "get_history",
    "clear_history",
    "service_memory",
    "erase_service_data",
    "erase_all_data",
    // Called by service pages through the bridge.
    "bridge_notify",
    "bridge_icon",
    "bridge_shortcut",
    "bridge_open_link",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("failed to run the Tauri build script");
}
