//! Shared application state managed by Tauri.
//!
//! # Locking rules
//!
//! State is split into three independent mutexes. To rule out deadlocks,
//! code must never hold more than one of these locks at the same time: copy
//! what you need out of one lock, release it, then take the next.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use crate::features::focus::FocusSession;
use crate::features::history::History;
use crate::features::pause::PauseState;
use crate::pin_store::PinStore;
use crate::services::{ServiceId, ServiceRegistry};
use crate::settings::SettingsStore;

/// The rectangle (in logical pixels) where service pages are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct ContentBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Volatile, per-run state. Nothing here is persisted.
#[derive(Debug, Default)]
pub struct Session {
    /// The service currently on screen.
    pub active: Option<ServiceId>,
    /// The most recently requested service. A slow page load for an older
    /// request must not override a newer click.
    pub requested: Option<ServiceId>,
    /// Whether Veil is on.
    pub veil: bool,
    /// The running focus session, if any.
    pub focus: Option<FocusSession>,
    /// Latest unread count per service.
    pub unread: HashMap<ServiceId, u32>,
    /// Where service pages should be drawn.
    pub bounds: Option<ContentBounds>,
    /// Whether a dialog in the main UI currently covers the content area.
    pub overlay_open: bool,
    /// Services whose webview currently exists.
    pub loaded: HashSet<ServiceId>,
    /// When each service was last on screen, for idle unloading.
    pub last_seen: HashMap<ServiceId, Instant>,
    /// Webview labels that already received their bridge capability.
    pub bridge_granted: HashSet<String>,
    /// Whether Cloister is locked (unlocked with the PIN or Windows Hello).
    pub locked: bool,
    /// Wrong PIN guesses in a row, and when the next guess is allowed.
    pub pin_failures: u32,
    pub pin_retry_at: Option<Instant>,
    /// When the main window last lost focus (`None` while focused).
    pub unfocused_since: Option<Instant>,
    /// Recent notifications, in memory only.
    pub history: History,
    /// Hidden pause-mode services and where they are in their sleep/check cycle.
    pub paused: HashMap<ServiceId, PauseState>,
}

impl Session {
    /// Total unread count across all services.
    pub fn unread_total(&self) -> u32 {
        self.unread.values().sum()
    }
}

/// Everything the backend shares between commands, handlers and threads.
pub struct AppState {
    pub registry: Mutex<ServiceRegistry>,
    pub settings: Mutex<SettingsStore>,
    pub session: Mutex<Session>,
    revision: AtomicU64,
    /// The Cloister PIN. App lock only works once one is set, so there is always a way back in.
    pub pin: Mutex<PinStore>,
    /// Whether Windows Hello is set up on this computer (checked once at startup).
    /// It is an optional quicker way to unlock; the PIN always works.
    pub hello_available: bool,
}

impl AppState {
    pub fn new(
        registry: ServiceRegistry,
        settings: SettingsStore,
        pin: PinStore,
        hello_available: bool,
    ) -> Self {
        let veil = settings.get().veil_on_launch;
        // Start locked only when the lock is on and a PIN exists to unlock it.
        let locked = settings.get().app_lock && pin.is_set();
        Self {
            registry: Mutex::new(registry),
            settings: Mutex::new(settings),
            session: Mutex::new(Session { veil, locked, ..Session::default() }),
            revision: AtomicU64::new(0),
            pin: Mutex::new(pin),
            hello_available,
        }
    }

    /// Locks the service registry.
    pub fn registry(&self) -> MutexGuard<'_, ServiceRegistry> {
        lock(&self.registry)
    }

    /// Locks the settings store.
    pub fn settings(&self) -> MutexGuard<'_, SettingsStore> {
        lock(&self.settings)
    }

    /// Returns a new, strictly increasing snapshot revision.
    pub fn next_revision(&self) -> u64 {
        self.revision.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Locks the PIN store.
    pub fn pin(&self) -> MutexGuard<'_, PinStore> {
        lock(&self.pin)
    }

    /// Whether Cloister can be locked: only once a PIN is set.
    pub fn can_lock(&self) -> bool {
        self.pin().is_set()
    }

    /// Locks the volatile session state.
    pub fn session(&self) -> MutexGuard<'_, Session> {
        lock(&self.session)
    }
}

/// Locks a mutex, recovering the data if a previous holder panicked. A panic
/// in one command should not take the whole app down with poisoned locks.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Current Unix time in milliseconds.
pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or_default()
}
