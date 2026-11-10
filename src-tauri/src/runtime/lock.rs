//! Locking Cloister behind Windows Hello.
//!
//! While locked, every service page is hidden and the UI shows only the lock
//! screen. Notifications still arrive, with their text hidden.

use tauri::{AppHandle, Manager, Runtime};

use super::webviews;
use crate::error::{AppError, AppResult};
use crate::platform::{self, AuthOutcome};
use crate::state::AppState;
use crate::{snapshot, MAIN_LABEL};

/// Locks Cloister now. Does nothing if Windows Hello is not available,
/// because the user would have no way back in.
pub fn lock<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    if !state.auth_available {
        return;
    }
    state.session().locked = true;
    if let Err(err) = webviews::sync_visibility(app) {
        log::warn!("could not hide pages while locking: {err}");
    }
    snapshot::publish(app);
}

/// Asks for Windows Hello and unlocks on success. Returns whether Cloister is now unlocked.
/// Blocks while the prompt is open; call from an async command.
pub fn unlock<R: Runtime>(app: &AppHandle<R>) -> AppResult<bool> {
    let window = app.get_window(MAIN_LABEL).ok_or(AppError::MainWindowMissing)?;
    match platform::authenticate(&window, "Unlock Cloister") {
        AuthOutcome::Verified => {
            app.state::<AppState>().session().locked = false;
            webviews::sync_visibility(app)?;
            webviews::focus_active(app);
            snapshot::publish(app);
            Ok(true)
        }
        AuthOutcome::Rejected => Ok(false),
        AuthOutcome::Unavailable => Err(AppError::InvalidInput(
            "Windows Hello is not available. Set it up in Windows Settings.".into(),
        )),
    }
}
