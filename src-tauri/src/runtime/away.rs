//! Locks Cloister or turns on Veil after the user has been away.
//!
//! "Away" means the main window has been out of focus (including hidden in
//! the tray). The idle sweeper calls [`check`] periodically.

use std::time::Instant;

use tauri::{AppHandle, Manager, Runtime};

use super::{lock, veil};
use crate::features::away::{due, minutes, AwayPolicy};
use crate::snapshot;
use crate::state::AppState;

/// Records focus changes of the main window and applies "Veil on blur".
pub fn on_focus_changed<R: Runtime>(app: &AppHandle<R>, focused: bool) {
    let state = app.state::<AppState>();
    let veil_on_blur = state.settings().get().veil_on_blur;
    let already_veiled = {
        let mut session = state.session();
        session.unfocused_since =
            if focused { None } else { session.unfocused_since.or(Some(Instant::now())) };
        session.veil
    };
    if !focused && veil_on_blur && !already_veiled {
        veil::set(app, true);
        snapshot::publish(app);
    }
}

/// Applies any away actions that are due.
pub fn check<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let settings = state.settings().get().clone();
    let policy = AwayPolicy {
        lock_after: if settings.app_lock { minutes(settings.lock_after_minutes) } else { None },
        veil_after: minutes(settings.veil_after_minutes),
    };
    let actions = {
        let session = state.session();
        let Some(since) = session.unfocused_since else { return };
        due(policy, since.elapsed(), session.locked, session.veil)
    };
    if actions.veil {
        veil::set(app, true);
        snapshot::publish(app);
    }
    if actions.lock {
        lock::lock(app);
    }
}
