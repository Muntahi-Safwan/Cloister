//! Creates, positions, shows and destroys the child webview of each service.
//!
//! Each service lives in its own child webview inside the main window, with
//! its own browser profile on disk. Webviews are created lazily, the first
//! time a service is opened, and only the active one is visible.

use tauri::webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder};
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, Webview, WebviewUrl};
use tauri_plugin_opener::OpenerExt;

use super::{badge, capabilities, inject, user_agent, veil};
use crate::error::{AppError, AppResult};
use crate::services::{self, hosts, unread, ServiceConfig};
use crate::state::{AppState, ContentBounds};
use crate::{persistence, platform, snapshot, MAIN_LABEL};

/// Prefix of every service webview label. Labels look like `svc-whatsapp-k3x9`.
const LABEL_PREFIX: &str = "svc-";

/// Webview label for a service id.
pub fn label_for(service_id: &str) -> String {
    format!("{LABEL_PREFIX}{service_id}")
}

/// Service id for a webview label, if the label belongs to a service.
pub fn service_id_of(label: &str) -> Option<&str> {
    label.strip_prefix(LABEL_PREFIX)
}

/// Returns the service's webview, creating it if it does not exist yet.
///
/// Must be called from an `async` command or another non-main thread: on
/// Windows, creating a webview from a synchronous command can deadlock.
pub fn ensure_loaded<R: Runtime>(app: &AppHandle<R>, service: &ServiceConfig) -> AppResult<Webview<R>> {
    let label = label_for(&service.id);
    if let Some(existing) = app.get_webview(&label) {
        return Ok(existing);
    }

    log::info!("loading service {} ({})", service.id, service.url);
    let window = app.get_window(MAIN_LABEL).ok_or(AppError::MainWindowMissing)?;
    let start_url = service.parsed_url()?;
    let recipe = services::recipe_of(service);
    let domains = hosts::domains_for(&start_url, recipe.map(|r| r.hosts));
    capabilities::grant_bridge(app, &label, &domains, start_url.port())?;

    let (veil_on, bounds) = {
        let state = app.state::<AppState>();
        let session = state.session();
        (session.veil, session.bounds)
    };

    let mut builder = WebviewBuilder::new(&label, WebviewUrl::External(start_url))
        .data_directory(persistence::profiles_dir(app)?.join(&service.id))
        .initialization_script(inject::bridge_script(service, recipe, veil_on))
        .on_navigation(navigation_handler(app.clone()))
        .on_new_window(new_window_handler(app.clone(), domains))
        .on_document_title_changed(title_handler(app.clone(), service))
        // The init script carries the Veil state from creation time, so re-apply
        // the current state as soon as each new document starts, and again when
        // it finishes in case the page replaced its <head>.
        .on_page_load(|webview, payload| {
            if matches!(payload.event(), PageLoadEvent::Started | PageLoadEvent::Finished) {
                veil::apply_current(&webview);
            }
        });
    if let Some(agent) = user_agent::for_preset(service.options.user_agent) {
        builder = builder.user_agent(agent);
    }
    // WKWebView ignores data folders; a named data store keeps each service's logins apart.
    #[cfg(target_os = "macos")]
    {
        builder = builder.data_store_identifier(services::profile::store_id(&service.id));
    }

    let rect = bounds.unwrap_or(ContentBounds { x: 0.0, y: 0.0, width: 1.0, height: 1.0 });
    let created = window.add_child(
        builder,
        LogicalPosition::new(rect.x, rect.y),
        LogicalSize::new(rect.width, rect.height),
    );
    let webview = match created {
        Ok(webview) => webview,
        // Two quick clicks can race to create the same webview; the loser reuses the winner's.
        Err(err) => app.get_webview(&label).ok_or(err)?,
    };
    webview.hide()?;
    platform::adopt_child(&webview, rect);
    if (service.zoom - 1.0).abs() > f64::EPSILON {
        webview.set_zoom(service.zoom)?;
    }

    app.state::<AppState>().session().loaded.insert(service.id.clone());
    Ok(webview)
}

/// Shows the active service (unless a dialog or the lock screen covers it) and hides the rest.
pub fn sync_visibility<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    let state = app.state::<AppState>();
    let low_memory = state.settings().get().low_memory_background;
    let (active, covered, bounds) = {
        let session = state.session();
        (session.active.clone(), session.overlay_open || session.locked, session.bounds)
    };

    for webview in service_webviews(app) {
        let is_active = active.as_deref() == service_id_of(webview.label());
        if is_active && !covered {
            if let Some(bounds) = bounds {
                platform::set_child_bounds(&webview, bounds)?;
            }
            webview.show()?;
        } else {
            webview.hide()?;
        }
        // A page under an open dialog will be back in a moment, so keep it warm.
        platform::set_background_memory(&webview, low_memory && !is_active);
    }
    Ok(())
}

/// Moves every service webview to `bounds` (called when the window resizes).
pub fn apply_bounds<R: Runtime>(app: &AppHandle<R>, bounds: ContentBounds) -> AppResult<()> {
    for webview in service_webviews(app) {
        platform::set_child_bounds(&webview, bounds)?;
    }
    Ok(())
}

/// Moves keyboard focus into the active service page, unless a dialog covers it.
pub fn focus_active<R: Runtime>(app: &AppHandle<R>) {
    let active = {
        let state = app.state::<AppState>();
        let session = state.session();
        if session.overlay_open || session.locked {
            return;
        }
        session.active.clone()
    };
    if let Some(webview) = active.and_then(|id| app.get_webview(&label_for(&id))) {
        let _ = webview.set_focus();
    }
}

/// Closes a service's webview, freeing its memory. The profile on disk is kept.
pub fn unload<R: Runtime>(app: &AppHandle<R>, service_id: &str) -> AppResult<()> {
    if let Some(webview) = app.get_webview(&label_for(service_id)) {
        webview.close()?;
    }
    app.state::<AppState>().session().loaded.remove(service_id);
    Ok(())
}

/// Reloads a service page.
pub fn reload<R: Runtime>(app: &AppHandle<R>, service_id: &str) -> AppResult<()> {
    if let Some(webview) = app.get_webview(&label_for(service_id)) {
        webview.reload()?;
    }
    Ok(())
}

/// All currently existing service webviews.
pub fn service_webviews<R: Runtime>(app: &AppHandle<R>) -> Vec<Webview<R>> {
    app.webviews().into_values().filter(|w| service_id_of(w.label()).is_some()).collect()
}

/// Top-level navigations stay inside the service, except non-web schemes
/// (`mailto:`, `tel:`, deep links), which are handed to the OS.
fn navigation_handler<R: Runtime>(app: AppHandle<R>) -> impl Fn(&url::Url) -> bool + Send + 'static {
    move |url| {
        if hosts::is_app_origin(url) {
            log::warn!("blocked a service page from navigating to {url}");
            return false;
        }
        if hosts::navigation_leaves_app(url) {
            open_externally(&app, url);
            return false;
        }
        true
    }
}

/// Popups to the service's own domains (for example a sign-in window) open
/// in-app so they share the session; every other link opens in the default browser.
fn new_window_handler<R: Runtime>(
    app: AppHandle<R>,
    domains: Vec<String>,
) -> impl Fn(url::Url, tauri::webview::NewWindowFeatures) -> NewWindowResponse<R> + Send + 'static {
    move |url, _features| {
        if hosts::is_internal(&url, &domains) {
            NewWindowResponse::Allow
        } else {
            open_externally(&app, &url);
            NewWindowResponse::Deny
        }
    }
}

/// Reads unread counts from title changes and refreshes badges when they change.
fn title_handler<R: Runtime>(
    app: AppHandle<R>,
    service: &ServiceConfig,
) -> impl Fn(Webview<R>, String) + Send + 'static {
    let service_id = service.id.clone();
    let rule = services::unread_rule_of(service);
    move |_webview, title| {
        let count = unread::parse_title(&title, rule);
        let changed = {
            let state = app.state::<AppState>();
            let mut session = state.session();
            session.unread.insert(service_id.clone(), count) != Some(count)
        };
        if changed {
            badge::refresh(&app);
            snapshot::publish(&app);
        }
    }
}

fn open_externally<R: Runtime>(app: &AppHandle<R>, url: &url::Url) {
    if let Err(err) = app.opener().open_url(url.as_str(), None::<&str>) {
        log::warn!("could not open {url} externally: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_round_trip() {
        let label = label_for("whatsapp-abc");
        assert_eq!(label, "svc-whatsapp-abc");
        assert_eq!(service_id_of(&label), Some("whatsapp-abc"));
        assert_eq!(service_id_of("main"), None);
    }
}
