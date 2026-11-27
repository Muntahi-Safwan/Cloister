//! Commands for locking, notification history, downloads, memory use and erasing data.

use tauri::{AppHandle, Manager, Runtime};

use crate::error::AppResult;
use crate::features::history::HistoryItem;
use crate::runtime::memory::{self, MemoryReport};
use crate::runtime::{downloads, erase, lock};
use crate::services::ServiceId;
use crate::snapshot::{self, Snapshot};
use crate::state::AppState;

/// Locks Cloister immediately (needs a Cloister PIN).
#[tauri::command]
pub fn lock_now<R: Runtime>(app: AppHandle<R>) -> Snapshot {
    lock::lock(&app);
    snapshot::build(&app)
}

/// Shows the Windows Hello prompt. Returns whether Cloister is now unlocked.
#[tauri::command]
pub async fn unlock<R: Runtime>(app: AppHandle<R>) -> AppResult<bool> {
    lock::unlock_with_hello(&app)
}

/// Unlocks with the Cloister PIN. Returns `false` for a wrong PIN.
#[tauri::command]
pub async fn unlock_with_pin<R: Runtime>(app: AppHandle<R>, pin: String) -> AppResult<bool> {
    lock::unlock_with_pin(&app, &pin)
}

/// Sets the Cloister PIN, or changes it (`current` is needed when one is set).
#[tauri::command]
pub async fn set_pin<R: Runtime>(
    app: AppHandle<R>,
    current: Option<String>,
    pin: String,
) -> AppResult<Snapshot> {
    lock::set_pin(&app, current.as_deref(), &pin)?;
    Ok(snapshot::build(&app))
}

/// Removes the Cloister PIN and turns app lock off.
#[tauri::command]
pub async fn remove_pin<R: Runtime>(app: AppHandle<R>, current: String) -> AppResult<Snapshot> {
    lock::remove_pin(&app, &current)?;
    Ok(snapshot::build(&app))
}

/// Recent notifications, newest first. Kept in memory only.
#[tauri::command]
pub fn get_history<R: Runtime>(app: AppHandle<R>) -> Vec<HistoryItem> {
    app.state::<AppState>().session().history.items()
}

/// Forgets all recent notifications.
#[tauri::command]
pub fn clear_history<R: Runtime>(app: AppHandle<R>) -> Snapshot {
    app.state::<AppState>().session().history.clear();
    publish(&app)
}

/// Opens a finished download with its default app (or shows it, if opening would run code).
#[tauri::command]
pub fn open_download<R: Runtime>(app: AppHandle<R>, id: u64) -> AppResult<()> {
    downloads::open(&app, id)
}

/// Shows a finished download in the system file manager.
#[tauri::command]
pub fn reveal_download<R: Runtime>(app: AppHandle<R>, id: u64) -> AppResult<()> {
    downloads::reveal(&app, id)
}

/// Clears finished downloads from the list. The files stay where they are.
#[tauri::command]
pub fn clear_downloads<R: Runtime>(app: AppHandle<R>) -> Snapshot {
    app.state::<AppState>().session().downloads.clear();
    publish(&app)
}

/// Memory used by each loaded service, and by the shared engine, in bytes.
#[tauri::command]
pub async fn service_memory<R: Runtime>(app: AppHandle<R>) -> MemoryReport {
    memory::report(&app)
}

/// Deletes one service's cookies, logins and cache. It stays in the sidebar, signed out.
#[tauri::command]
pub async fn erase_service_data<R: Runtime>(app: AppHandle<R>, id: ServiceId) -> AppResult<Snapshot> {
    erase::service_data(&app, &id)?;
    {
        let state = app.state::<AppState>();
        let mut session = state.session();
        session.history.forget_service(&id);
        session.downloads.forget_service(&id);
    }
    Ok(publish(&app))
}

/// Deletes every service's cookies, logins and cache.
#[tauri::command]
pub async fn erase_all_data<R: Runtime>(app: AppHandle<R>) -> AppResult<Snapshot> {
    erase::all_data(&app)?;
    {
        let state = app.state::<AppState>();
        let mut session = state.session();
        session.history.clear();
        session.downloads.clear();
    }
    Ok(publish(&app))
}

fn publish<R: Runtime>(app: &AppHandle<R>) -> Snapshot {
    snapshot::publish(app);
    snapshot::build(app)
}
