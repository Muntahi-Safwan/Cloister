//! Clickable Windows toasts.
//!
//! The notification plugin cannot report clicks on Windows, so Cloister
//! shows its toasts directly through WinRT and opens the right service when
//! one is clicked. Clicks are handled while Cloister is running.

use tauri::{AppHandle, Runtime};
use tauri_winrt_notification::Toast;

/// Shows a toast. Returns `false` if Windows refused it, so the caller can fall back.
pub fn show<R: Runtime>(
    app: &AppHandle<R>,
    title: &str,
    body: &str,
    on_click: impl Fn() + Send + 'static,
) -> bool {
    let result = Toast::new(&app_id(app))
        .title(title)
        .text1(body)
        .on_activated(move |_action| {
            on_click();
            Ok::<(), tauri_winrt_notification::Error>(())
        })
        .show();
    match result {
        Ok(()) => true,
        Err(err) => {
            log::warn!("could not show toast: {err:?}");
            false
        }
    }
}

/// The app identity toasts are shown under. Windows only accepts the real
/// identifier for installed apps; development builds borrow PowerShell's,
/// exactly like `tauri-plugin-notification` does.
fn app_id<R: Runtime>(app: &AppHandle<R>) -> String {
    let installed = tauri::utils::platform::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.display().to_string()))
        .is_some_and(|dir| !(dir.ends_with("\\target\\debug") || dir.ends_with("\\target\\release")));
    if installed {
        app.config().identifier.clone()
    } else {
        Toast::POWERSHELL_APP_ID.to_string()
    }
}
