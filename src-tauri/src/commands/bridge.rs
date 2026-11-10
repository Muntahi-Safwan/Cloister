//! Commands callable from service pages through the injected bridge.
//!
//! These are the only commands a third-party page can reach (see
//! `runtime/capabilities.rs`). The calling service is identified from the
//! webview label, never from the payload, so a page cannot impersonate another.

use tauri::{AppHandle, Emitter, EventTarget, Manager, Runtime, Webview};

use crate::runtime::{notifications, webviews};
use crate::services::model::sanitize_icon_url;
use crate::state::AppState;
use crate::{snapshot, MAIN_LABEL, SHORTCUT_EVENT};

/// Actions the bridge may forward. Anything else is ignored.
const ALLOWED_ACTIONS: &[&str] = &["palette", "veil", "focus", "lock", "zoom:in", "zoom:out", "zoom:reset"];

/// A service page wants to show a notification.
#[tauri::command]
pub fn bridge_notify<R: Runtime>(app: AppHandle<R>, webview: Webview<R>, title: String, body: String) {
    if let Some(service_id) = webviews::service_id_of(webview.label()) {
        notifications::deliver(&app, service_id, &title, &body);
    }
}

/// A service page reported its own icon (favicon or touch icon).
#[tauri::command]
pub fn bridge_icon<R: Runtime>(app: AppHandle<R>, webview: Webview<R>, href: String) {
    let Some(service_id) = webviews::service_id_of(webview.label()) else { return };
    let Some(icon_url) = sanitize_icon_url(&href) else { return };
    let changed = app.state::<AppState>().registry().set_icon_url(service_id, icon_url);
    match changed {
        Ok(true) => snapshot::publish(&app),
        Ok(false) => {}
        Err(err) => log::warn!("could not store icon for {service_id}: {err}"),
    }
}

/// A Cloister keyboard shortcut was pressed inside a service page.
#[tauri::command]
pub fn bridge_shortcut<R: Runtime>(app: AppHandle<R>, action: String) {
    if is_allowed_action(&action) {
        let _ = app.emit_to(EventTarget::webview(MAIN_LABEL), SHORTCUT_EVENT, action);
    }
}

fn is_allowed_action(action: &str) -> bool {
    ALLOWED_ACTIONS.contains(&action)
        || action.strip_prefix("switch:").is_some_and(|n| matches!(n.parse::<u8>(), Ok(1..=9)))
}

#[cfg(test)]
mod tests {
    use super::is_allowed_action;

    #[test]
    fn only_known_actions_pass() {
        assert!(is_allowed_action("veil"));
        assert!(is_allowed_action("switch:3"));
        assert!(is_allowed_action("zoom:in"));
        assert!(!is_allowed_action("switch:0"));
        assert!(!is_allowed_action("switch:12"));
        assert!(!is_allowed_action("quit"));
    }
}
