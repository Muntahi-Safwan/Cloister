//! The single view model the frontend renders from.
//!
//! Instead of many fine-grained events, the backend pushes one [`Snapshot`]
//! whenever anything visible changes. The UI replaces its state with it, which
//! keeps the two sides trivially in sync.

use serde::Serialize;
use tauri::{AppHandle, Emitter, EventTarget, Manager, Runtime};

use crate::features::focus::FocusView;
use crate::services::{ServiceConfig, ServiceId};
use crate::settings::AppSettings;
use crate::state::{now_ms, AppState};

/// Event name for snapshot pushes.
pub const SNAPSHOT_EVENT: &str = "cloister://snapshot";

/// A configured service plus its live status.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceView {
    #[serde(flatten)]
    pub config: ServiceConfig,
    pub unread: u32,
    pub loaded: bool,
}

/// Everything the main window needs to draw itself.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// Increases with every snapshot, so the UI can ignore stale ones that arrive late.
    pub revision: u64,
    pub services: Vec<ServiceView>,
    pub active: Option<ServiceId>,
    pub veil: bool,
    pub focus: Option<FocusView>,
    pub unread_total: u32,
    pub settings: AppSettings,
    /// Cloister is locked behind Windows Hello.
    pub locked: bool,
    /// Windows Hello can be used on this computer.
    pub auth_available: bool,
    /// Number of notifications in the in-memory history.
    pub history_count: usize,
    /// When the newest notification in the history arrived (Unix ms).
    pub history_latest_ms: Option<u64>,
}

/// Builds a snapshot from current state, taking each lock in turn.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> Snapshot {
    let state = app.state::<AppState>();
    let configs = state.registry().all().to_vec();
    let settings = state.settings().get().clone();
    let session = state.session();
    let now = now_ms();

    let services = configs
        .into_iter()
        .map(|config| ServiceView {
            unread: session.unread.get(&config.id).copied().unwrap_or(0),
            loaded: session.loaded.contains(&config.id),
            config,
        })
        .collect();

    Snapshot {
        revision: state.next_revision(),
        services,
        active: session.active.clone(),
        veil: session.veil,
        // An expired session stays visible until the UI ends it, so its held
        // notifications always reach the summary.
        focus: session.focus.as_ref().map(|f| f.view(now)),
        unread_total: session.unread_total(),
        settings,
        locked: session.locked,
        auth_available: state.auth_available,
        history_count: session.history.len(),
        history_latest_ms: session.history.latest_ms(),
    }
}

/// Pushes a fresh snapshot to the main UI webview only (never to service pages).
pub fn publish<R: Runtime>(app: &AppHandle<R>) {
    if let Err(err) = app.emit_to(EventTarget::webview(crate::MAIN_LABEL), SNAPSHOT_EVENT, build(app)) {
        log::warn!("failed to publish snapshot: {err}");
    }
}
