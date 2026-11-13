//! Commands callable from service pages through the injected bridge.
//!
//! These are the only commands a third-party page can reach (see
//! `runtime/capabilities.rs`). The calling service is identified from the
//! webview label, never from the payload, so a page cannot impersonate another.

use std::time::Instant;

use tauri::{AppHandle, Emitter, EventTarget, Manager, Runtime, Webview};
use tauri_plugin_opener::OpenerExt;

use crate::features::link_limit;
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

/// The user clicked a link that should open in the default browser.
///
/// Only web, email and phone links are opened, wrapped links (such as
/// `google.com/url?q=…`) are opened at their real destination, and each
/// service may open at most a few links in a short time.
#[tauri::command]
pub fn bridge_open_link<R: Runtime>(app: AppHandle<R>, webview: Webview<R>, url: String) {
    let Some(service_id) = webviews::service_id_of(webview.label()) else { return };
    let Some(target) = link_target(&url) else {
        log::warn!("{service_id} asked to open an unsupported link; ignored");
        return;
    };
    if !may_open_link(&app, service_id) {
        log::warn!("{service_id} opened too many links at once; ignored one");
        return;
    }
    log::info!("opening a link from {service_id} in the default browser ({})", describe(&target));
    if let Err(err) = app.opener().open_url(target.as_str(), None::<&str>) {
        log::warn!("could not open a link from {service_id} in the default browser: {err}");
    }
}

/// Records a link opened by `service_id` and says whether it is within the limit.
pub(crate) fn may_open_link<R: Runtime>(app: &AppHandle<R>, service_id: &str) -> bool {
    let state = app.state::<AppState>();
    let mut session = state.session();
    let recent = session.link_opens.entry(service_id.to_string()).or_default();
    link_limit::allow(recent, Instant::now())
}

/// The address to open for `raw`, or `None` for anything other than a web,
/// email or phone link.
fn link_target(raw: &str) -> Option<url::Url> {
    let url = url::Url::parse(raw).ok()?;
    let target = crate::services::hosts::unwrap_redirect(&url).unwrap_or(url);
    matches!(target.scheme(), "https" | "http" | "mailto" | "tel").then_some(target)
}

/// What the log says about a link: its host, never the full address.
pub(crate) fn describe(url: &url::Url) -> String {
    url.host_str().map_or_else(|| format!("{} link", url.scheme()), str::to_string)
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
    use super::{is_allowed_action, link_target};

    #[test]
    fn only_web_mail_and_phone_links_open() {
        let open = |s: &str| link_target(s).map(|u| u.to_string());
        assert_eq!(open("https://example.com/a"), Some("https://example.com/a".into()));
        assert_eq!(
            open("https://www.google.com/url?q=https://example.com/"),
            Some("https://example.com/".into())
        );
        assert_eq!(open("mailto:a@b.com"), Some("mailto:a@b.com".into()));
        assert_eq!(open("file:///C:/Windows/System32/calc.exe"), None);
        assert_eq!(open("ms-msdt:/id PCWDiagnostic"), None);
        assert_eq!(open("javascript:alert(1)"), None);
        assert_eq!(open("not a url"), None);
    }

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
