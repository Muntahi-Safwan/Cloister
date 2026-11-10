//! Runs the sleep/check cycle of services in "pause in background" mode.
//!
//! Called by the background sweep. Decisions come from `features::pause`; this
//! module applies them to webviews. Where pausing is unsupported the mode
//! behaves like "keep notifying" and nothing happens here.

use std::time::Instant;

use tauri::{AppHandle, Manager, Runtime};

use super::webviews;
use crate::features::pause::{self, PauseAction, PauseState};
use crate::platform;
use crate::services::BackgroundMode;
use crate::state::AppState;

/// Suspends, wakes or re-suspends each hidden pause-mode service as due.
pub fn tick<R: Runtime>(app: &AppHandle<R>) {
    if !platform::CAN_PAUSE {
        return;
    }
    let state = app.state::<AppState>();
    let check_every = pause::interval(state.settings().get().pause_check_minutes);
    let ids: Vec<String> = state
        .registry()
        .all()
        .iter()
        .filter(|s| s.options.background == BackgroundMode::Pause)
        .map(|s| s.id.clone())
        .collect();

    let now = Instant::now();
    for id in ids {
        let (visible, loaded, current) = {
            let session = state.session();
            let visible =
                session.active.as_deref() == Some(id.as_str()) && !session.overlay_open && !session.locked;
            (visible, session.loaded.contains(&id), session.paused.get(&id).copied())
        };
        if !loaded || visible {
            // On screen pages are awake (WebView2 resumes them when shown).
            state.session().paused.remove(&id);
            continue;
        }
        let Some(webview) = app.get_webview(&webviews::label_for(&id)) else { continue };
        let next = match pause::next_action(current, now, check_every) {
            PauseAction::Suspend => {
                platform::set_paused(&webview, true);
                Some(PauseState::Paused(now))
            }
            PauseAction::Wake => {
                platform::set_paused(&webview, false);
                Some(PauseState::Checking(now))
            }
            PauseAction::Wait => None,
        };
        if let Some(next) = next {
            state.session().paused.insert(id, next);
        }
    }
}
