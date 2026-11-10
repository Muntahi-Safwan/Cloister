//! Platform-specific integrations. Each function has a fallback so the rest
//! of the app never needs `cfg` attributes.

#[cfg(target_os = "linux")]
mod gtk_layer;
#[cfg(target_os = "windows")]
mod hello;
#[cfg(target_os = "windows")]
mod toast;
#[cfg(target_os = "windows")]
mod webview2;

use tauri::{AppHandle, Runtime, Webview, Window};

use crate::state::ContentBounds;

/// Prepares `window` to host service pages beside the main UI. Call once at startup.
pub fn prepare_window<R: Runtime>(window: &Window<R>) {
    #[cfg(target_os = "linux")]
    gtk_layer::install(window);

    #[cfg(not(target_os = "linux"))]
    let _ = window;
}

/// Finishes placing a newly created service page.
pub fn adopt_child<R: Runtime>(webview: &Webview<R>, bounds: ContentBounds) {
    #[cfg(target_os = "linux")]
    gtk_layer::adopt(webview, bounds);

    #[cfg(not(target_os = "linux"))]
    let _ = (webview, bounds);
}

/// Moves and resizes a service page within its window.
pub fn set_child_bounds<R: Runtime>(webview: &Webview<R>, bounds: ContentBounds) -> tauri::Result<()> {
    #[cfg(target_os = "linux")]
    {
        gtk_layer::set_bounds(webview, bounds);
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    webview.set_bounds(tauri::Rect {
        position: tauri::LogicalPosition::new(bounds.x, bounds.y).into(),
        size: tauri::LogicalSize::new(bounds.width, bounds.height).into(),
    })
}

/// Result of asking the user to prove it is them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthOutcome {
    Verified,
    Rejected,
    Unavailable,
}

/// Hints the browser engine that a page is in the background and can give
/// memory back. Scripts and connections keep running, so notifications still
/// arrive. Pass `false` when the page comes back on screen.
pub fn set_background_memory<R: Runtime>(webview: &Webview<R>, low: bool) {
    #[cfg(target_os = "windows")]
    webview2::set_memory_target(webview, low);

    #[cfg(not(target_os = "windows"))]
    let _ = (webview, low);
}

/// Whether the system can verify the user (Windows Hello). Blocks briefly.
pub fn authentication_available() -> bool {
    #[cfg(target_os = "windows")]
    return hello::available();

    #[cfg(not(target_os = "windows"))]
    false
}

/// Asks the user to verify with Windows Hello. Blocks until answered.
pub fn authenticate<R: Runtime>(window: &Window<R>, reason: &str) -> AuthOutcome {
    #[cfg(target_os = "windows")]
    return hello::verify(window, reason);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window, reason);
        AuthOutcome::Unavailable
    }
}

/// Shows a toast that calls `on_click` when clicked. Returns `false` when the
/// platform has no clickable toasts, so the caller can use a plain one.
pub fn show_clickable_toast<R: Runtime>(
    app: &AppHandle<R>,
    title: &str,
    body: &str,
    on_click: impl Fn() + Send + 'static,
) -> bool {
    #[cfg(target_os = "windows")]
    return toast::show(app, title, body, on_click);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, title, body, on_click);
        false
    }
}
