//! Reloads hidden pages that have grown too large (see `features::refresh`).

use std::time::Instant;

use tauri::{AppHandle, Manager, Runtime};

use super::{memory, webviews};
use crate::features::refresh::{self, Candidate};
use crate::state::AppState;

/// Measures loaded services and reloads hidden ones above the size limit.
/// Blocks briefly; call from a background thread.
pub fn check<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let limit_mb = state.settings().get().refresh_over_mb;
    if limit_mb == 0 {
        return;
    }
    let candidates: Vec<Candidate> = {
        let session = state.session();
        let now = Instant::now();
        session
            .loaded
            .iter()
            .map(|id| Candidate {
                id: id.clone(),
                hidden_for: session.last_seen.get(id).map(|seen| now.duration_since(*seen)),
                exempt: session.active.as_deref() == Some(id.as_str()) || session.paused.contains_key(id),
            })
            .collect()
    };
    if candidates.is_empty() {
        return;
    }
    let usage = memory::report(app).services;
    for id in refresh::pick(&candidates, &usage, u64::from(limit_mb) * 1024 * 1024) {
        log::info!("refreshing {id}: it grew past {limit_mb} MB while hidden");
        if let Err(err) = webviews::reload(app, &id) {
            log::warn!("could not refresh {id}: {err}");
        }
    }
}
