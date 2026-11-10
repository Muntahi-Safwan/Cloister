//! Bringing a service on screen.

use tauri::{AppHandle, Manager, Runtime};

use super::{lifecycle, webviews};
use crate::error::AppResult;
use crate::state::AppState;

/// Loads (if needed), activates and focuses a service.
///
/// If another service was requested while this one was loading, the newer
/// request wins and this call only finishes loading in the background.
/// Must not run on the UI thread (see [`webviews::ensure_loaded`]).
pub fn activate<R: Runtime>(app: &AppHandle<R>, id: &str) -> AppResult<()> {
    let state = app.state::<AppState>();
    let service = state.registry().get(id)?.clone();
    state.session().requested = Some(service.id.clone());
    webviews::ensure_loaded(app, &service)?;

    let previous = {
        let mut session = state.session();
        if session.requested.as_deref() != Some(service.id.as_str()) {
            return Ok(());
        }
        session.active.replace(service.id.clone())
    };
    if let Some(previous) = previous {
        lifecycle::touch(app, &previous);
    }
    lifecycle::touch(app, &service.id);

    webviews::sync_visibility(app)?;
    webviews::focus_active(app);
    Ok(())
}
