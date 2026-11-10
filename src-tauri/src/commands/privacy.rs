//! Commands for locking, notification history, memory use and erasing data.

use std::collections::HashMap;

use tauri::{AppHandle, Manager, Runtime};

use crate::error::AppResult;
use crate::features::history::HistoryItem;
use crate::runtime::{erase, lock, memory};
use crate::services::ServiceId;
use crate::snapshot::{self, Snapshot};
use crate::state::AppState;

/// Locks Cloister immediately (needs Windows Hello).
#[tauri::command]
pub fn lock_now<R: Runtime>(app: AppHandle<R>) -> Snapshot {
    lock::lock(&app);
    snapshot::build(&app)
}

/// Shows the Windows Hello prompt. Returns whether Cloister is now unlocked.
#[tauri::command]
pub async fn unlock<R: Runtime>(app: AppHandle<R>) -> AppResult<bool> {
    lock::unlock(&app)
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

/// Memory used by each loaded service, in bytes.
#[tauri::command]
pub async fn service_memory<R: Runtime>(app: AppHandle<R>) -> HashMap<ServiceId, u64> {
    memory::usage_by_service(&app)
}

/// Deletes one service's cookies, logins and cache. It stays in the sidebar, signed out.
#[tauri::command]
pub async fn erase_service_data<R: Runtime>(app: AppHandle<R>, id: ServiceId) -> AppResult<Snapshot> {
    erase::service_data(&app, &id)?;
    app.state::<AppState>().session().history.forget_service(&id);
    Ok(publish(&app))
}

/// Deletes every service's cookies, logins and cache.
#[tauri::command]
pub async fn erase_all_data<R: Runtime>(app: AppHandle<R>) -> AppResult<Snapshot> {
    erase::all_data(&app)?;
    app.state::<AppState>().session().history.clear();
    Ok(publish(&app))
}

fn publish<R: Runtime>(app: &AppHandle<R>) -> Snapshot {
    snapshot::publish(app);
    snapshot::build(app)
}
