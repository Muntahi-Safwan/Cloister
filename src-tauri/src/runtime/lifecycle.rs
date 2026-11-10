//! Background housekeeping.
//!
//! A background thread wakes up every 30 seconds to:
//! * close the webview of every "unload when idle" service that has not been
//!   on screen for the configured number of minutes (it reloads when opened);
//! * apply away actions such as auto-lock and auto-Veil (see `away.rs`).

use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, Runtime};

use super::webviews;
use crate::services::BackgroundMode;
use crate::snapshot;
use crate::state::AppState;

const SWEEP_INTERVAL: Duration = Duration::from_secs(30);

/// Starts the idle sweeper thread.
pub fn start<R: Runtime>(app: AppHandle<R>) {
    thread::Builder::new()
        .name("cloister-idle-sweeper".into())
        .spawn(move || loop {
            thread::sleep(SWEEP_INTERVAL);
            sweep(&app);
            super::away::check(&app);
        })
        .map(|_| ())
        .unwrap_or_else(|err| log::error!("could not start idle sweeper: {err}"));
}

/// Records that a service was just on screen.
pub fn touch<R: Runtime>(app: &AppHandle<R>, service_id: &str) {
    app.state::<AppState>().session().last_seen.insert(service_id.to_string(), Instant::now());
}

fn sweep<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let idle_limit = Duration::from_secs(u64::from(state.settings().get().unload_after_minutes) * 60);
    let unloadable: Vec<String> = state
        .registry()
        .all()
        .iter()
        .filter(|s| s.options.background == BackgroundMode::UnloadWhenIdle)
        .map(|s| s.id.clone())
        .collect();

    let expired: Vec<String> = {
        let session = state.session();
        unloadable
            .into_iter()
            .filter(|id| session.active.as_deref() != Some(id.as_str()) && session.loaded.contains(id))
            .filter(|id| session.last_seen.get(id).is_none_or(|seen| seen.elapsed() >= idle_limit))
            .collect()
    };

    if expired.is_empty() {
        return;
    }
    for id in &expired {
        // The user may have opened this service since the list was built.
        let still_idle = {
            let session = state.session();
            session.active.as_deref() != Some(id.as_str())
                && session.last_seen.get(id).is_none_or(|seen| seen.elapsed() >= idle_limit)
        };
        if !still_idle {
            continue;
        }
        if let Err(err) = webviews::unload(app, id) {
            log::warn!("could not unload idle service {id}: {err}");
        }
    }
    snapshot::publish(app);
}
