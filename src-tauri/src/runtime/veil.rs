//! Applies Veil (privacy blur) to service pages.

use tauri::{AppHandle, Manager, Runtime, Webview};

use super::{inject, webviews};
use crate::state::AppState;

/// Turns Veil on or off for every loaded service.
pub fn set<R: Runtime>(app: &AppHandle<R>, on: bool) {
    app.state::<AppState>().session().veil = on;
    let script = inject::set_veil_script(on);
    for webview in webviews::service_webviews(app) {
        if let Err(err) = webview.eval(&script) {
            log::warn!("could not update Veil in {}: {err}", webview.label());
        }
    }
}

/// Re-applies the current Veil state to one page, e.g. after it navigated.
pub fn apply_current<R: Runtime>(webview: &Webview<R>) {
    let on = webview.app_handle().state::<AppState>().session().veil;
    if let Err(err) = webview.eval(inject::set_veil_script(on)) {
        log::warn!("could not apply Veil to {}: {err}", webview.label());
    }
}
