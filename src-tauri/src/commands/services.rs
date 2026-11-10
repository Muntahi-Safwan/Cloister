//! Commands for adding, removing, ordering and opening services.

use tauri::{AppHandle, Manager, Runtime};

use crate::error::AppResult;
use crate::runtime::{activation, badge, webviews};
use crate::services::catalog::{self, Recipe};
use crate::services::model::IconKind;
use crate::services::{NewService, ServiceId};
use crate::snapshot::{self, Snapshot};
use crate::state::AppState;

/// Returns the current snapshot. Called once when the UI starts.
#[tauri::command]
pub fn get_snapshot<R: Runtime>(app: AppHandle<R>) -> Snapshot {
    snapshot::build(&app)
}

/// Returns the built-in service catalog.
#[tauri::command]
pub fn list_catalog() -> Vec<Recipe> {
    catalog::RECIPES.to_vec()
}

/// Adds a service and opens it.
#[tauri::command]
pub async fn add_service<R: Runtime>(app: AppHandle<R>, input: NewService) -> AppResult<Snapshot> {
    let added = app.state::<AppState>().registry().add(input)?;
    activation::activate(&app, &added.id)?;
    Ok(publish(&app))
}

/// Removes a service, closes its page and forgets its unread count.
///
/// The service's browser profile stays on disk, so re-adding the same
/// service later keeps the user signed in.
#[tauri::command]
pub async fn remove_service<R: Runtime>(app: AppHandle<R>, id: ServiceId) -> AppResult<Snapshot> {
    webviews::unload(&app, &id)?;
    let state = app.state::<AppState>();
    state.registry().remove(&id)?;
    let next = state.registry().all().first().map(|s| s.id.clone());

    let was_active = {
        let mut session = state.session();
        session.unread.remove(&id);
        session.history.forget_service(&id);
        session.last_seen.remove(&id);
        session.active.as_deref() == Some(id.as_str())
    };
    if was_active {
        match next {
            Some(next_id) => activation::activate(&app, &next_id)?,
            None => {
                let mut session = state.session();
                session.active = None;
                session.requested = None;
            }
        }
    }
    badge::refresh(&app);
    Ok(publish(&app))
}

/// Saves a new sidebar order.
#[tauri::command]
pub fn reorder_services<R: Runtime>(app: AppHandle<R>, ids: Vec<ServiceId>) -> AppResult<Snapshot> {
    app.state::<AppState>().registry().reorder(&ids)?;
    Ok(publish(&app))
}

/// Brings a service on screen, loading it first if needed.
#[tauri::command]
pub async fn activate_service<R: Runtime>(app: AppHandle<R>, id: ServiceId) -> AppResult<Snapshot> {
    activation::activate(&app, &id)?;
    Ok(publish(&app))
}

/// Switches a service between its initials and the site's own icon.
#[tauri::command]
pub fn set_service_icon<R: Runtime>(app: AppHandle<R>, id: ServiceId, icon: IconKind) -> AppResult<Snapshot> {
    app.state::<AppState>().registry().set_icon(&id, icon)?;
    Ok(publish(&app))
}

/// Reloads a service page.
#[tauri::command]
pub fn reload_service<R: Runtime>(app: AppHandle<R>, id: ServiceId) -> AppResult<()> {
    webviews::reload(&app, &id)
}

/// Publishes a snapshot and returns it to the caller.
fn publish<R: Runtime>(app: &AppHandle<R>) -> Snapshot {
    snapshot::publish(app);
    snapshot::build(app)
}
