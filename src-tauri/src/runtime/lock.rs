//! Locking Cloister behind the Cloister PIN, with Windows Hello as an optional
//! quicker way in.
//!
//! While locked, every service page is hidden and the UI shows only the lock
//! screen. Notifications still arrive, with their text hidden.
//!
//! The lock only works once a PIN is set. Windows Hello alone is never enough,
//! because when Hello breaks (a damaged PIN container, a removed fingerprint
//! reader) the user would have no way back in.

use std::time::Instant;

use tauri::{AppHandle, Manager, Runtime};

use super::webviews;
use crate::error::{AppError, AppResult};
use crate::features::passcode;
use crate::platform::{self, AuthOutcome};
use crate::state::AppState;
use crate::{snapshot, MAIN_LABEL};

/// Locks Cloister now. Does nothing until a PIN is set.
pub fn lock<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    if !state.can_lock() {
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
pub fn unlock_with_hello<R: Runtime>(app: &AppHandle<R>) -> AppResult<bool> {
    if !app.state::<AppState>().hello_available {
        return Err(AppError::InvalidInput(
            "Windows Hello isn't set up on this computer. Use your Cloister PIN.".into(),
        ));
    }
    let window = app.get_window(MAIN_LABEL).ok_or(AppError::MainWindowMissing)?;
    match platform::authenticate(&window, "Unlock Cloister") {
        AuthOutcome::Verified => {
            finish_unlock(app)?;
            Ok(true)
        }
        AuthOutcome::Rejected => Ok(false),
        AuthOutcome::Unavailable => Err(AppError::InvalidInput(
            "Windows Hello isn't working right now. Use your Cloister PIN instead.".into(),
        )),
    }
}

/// Checks the PIN and unlocks if it matches. Returns `Ok(false)` for a wrong PIN.
/// After several wrong guesses in a row, further guesses are refused for a while.
pub fn unlock_with_pin<R: Runtime>(app: &AppHandle<R>, pin: &str) -> AppResult<bool> {
    let state = app.state::<AppState>();
    if let Some(retry_at) = state.session().pin_retry_at {
        let now = Instant::now();
        if retry_at > now {
            let seconds = (retry_at - now).as_secs() + 1;
            return Err(AppError::InvalidInput(format!(
                "Too many wrong tries. Try again in {seconds} seconds."
            )));
        }
    }
    let Some(hash) = state.pin().hash().map(str::to_owned) else {
        return Err(AppError::InvalidInput("No Cloister PIN is set.".into()));
    };
    // Hashing takes a moment on purpose; do it without holding any lock.
    if passcode::verify(pin, &hash) {
        {
            let mut session = state.session();
            session.pin_failures = 0;
            session.pin_retry_at = None;
        }
        finish_unlock(app)?;
        return Ok(true);
    }
    let mut session = state.session();
    session.pin_failures += 1;
    let wait = passcode::wait_after(session.pin_failures);
    session.pin_retry_at = (!wait.is_zero()).then(|| Instant::now() + wait);
    log::info!("wrong Cloister PIN ({} in a row)", session.pin_failures);
    Ok(false)
}

/// Sets or changes the PIN. Changing it needs the current one.
/// Refused while locked, so the lock screen can't be used to replace it.
pub fn set_pin<R: Runtime>(app: &AppHandle<R>, current: Option<&str>, next: &str) -> AppResult<()> {
    let state = app.state::<AppState>();
    ensure_unlocked(&state)?;
    passcode::validate(next).map_err(AppError::InvalidInput)?;
    check_current(&state, current)?;
    let hash = passcode::hash(next)
        .ok_or_else(|| AppError::InvalidInput("Could not save the PIN. Try again.".into()))?;
    state.pin().set(Some(hash))?;
    snapshot::publish(app);
    Ok(())
}

/// Removes the PIN (needs the current one). App lock turns off with it.
pub fn remove_pin<R: Runtime>(app: &AppHandle<R>, current: &str) -> AppResult<()> {
    let state = app.state::<AppState>();
    ensure_unlocked(&state)?;
    check_current(&state, Some(current))?;
    state.pin().set(None)?;
    {
        let mut settings = state.settings();
        let mut next = settings.get().clone();
        next.app_lock = false;
        settings.replace(next)?;
    }
    snapshot::publish(app);
    Ok(())
}

fn ensure_unlocked(state: &AppState) -> AppResult<()> {
    if state.session().locked {
        return Err(AppError::InvalidInput("Unlock Cloister first.".into()));
    }
    Ok(())
}

/// When a PIN is already set, `current` must match it.
fn check_current(state: &AppState, current: Option<&str>) -> AppResult<()> {
    let Some(hash) = state.pin().hash().map(str::to_owned) else {
        return Ok(());
    };
    match current {
        Some(pin) if passcode::verify(pin, &hash) => Ok(()),
        _ => Err(AppError::InvalidInput("Your current PIN is not right.".into())),
    }
}

fn finish_unlock<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    app.state::<AppState>().session().locked = false;
    webviews::sync_visibility(app)?;
    webviews::focus_active(app);
    snapshot::publish(app);
    Ok(())
}
