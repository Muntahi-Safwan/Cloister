//! Delivers notifications coming from service pages.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

use crate::features::focus::HeldNotification;
use crate::features::history::{Delivery, HistoryItem};
use crate::features::notify::{redacted, route, NotifyContext, Route};
use crate::platform;
use crate::runtime::{activation, tray};
use crate::snapshot;
use crate::state::{now_ms, AppState};

/// Longest title and body forwarded to a toast. Longer text is cut.
const MAX_TITLE: usize = 120;
const MAX_BODY: usize = 400;

/// Routes one notification from `service_id` and shows, holds or drops it.
pub fn deliver<R: Runtime>(app: &AppHandle<R>, service_id: &str, title: &str, body: &str) {
    let title = truncate(title, MAX_TITLE);
    let body = truncate(body, MAX_BODY);
    let state = app.state::<AppState>();

    let Ok(service) = state.registry().get(service_id).cloned() else {
        log::warn!("notification from unknown service {service_id}");
        return;
    };
    let settings = state.settings().get().clone();
    let now = now_ms();
    let (veil, locked, focus) = {
        let session = state.session();
        (session.veil, session.locked, session.focus.as_ref().is_some_and(|f| f.is_running(now)))
    };

    let ctx = NotifyContext {
        service_name: &service.name,
        enabled: service.options.notifications,
        muted: service.is_muted(now),
        quiet: settings.quiet_hours.active_at(&chrono::Local::now()),
        hide_previews: service.options.hide_previews,
        // A locked Cloister never shows message text.
        veil: veil || locked,
        redact_in_veil: settings.redact_toasts_in_veil || locked,
        focus,
        vip_names: &settings.vip_names,
    };

    let decision = route(ctx, &title, &body);
    log::debug!("notification from {}: {:?}", service.id, std::mem::discriminant(&decision));
    let delivery = match &decision {
        Route::Show { .. } => Delivery::Shown,
        Route::Hold => Delivery::Held,
        Route::Drop => Delivery::Silenced,
    };
    // History keeps the original text in memory (the UI applies Veil when showing
    // it), except for services whose previews are hidden: their text is not kept.
    let (title, body) = if service.options.hide_previews { redacted(&service.name) } else { (title, body) };
    {
        let mut session = state.session();
        session.history.push(HistoryItem {
            service_id: service.id.clone(),
            title: title.clone(),
            body: body.clone(),
            at_ms: now,
            delivery,
        });
        if let Route::Hold = decision {
            if let Some(focus) = session.focus.as_mut() {
                focus.hold(HeldNotification { service_id: service.id.clone(), title, body, at_ms: now });
            }
        }
    }
    if let Route::Show { title, body } = decision {
        show_service_toast(app, &service.id, &title, &body);
    }
    snapshot::publish(app);
}

/// Shows a toast that opens `service_id` when clicked, falling back to a plain toast.
fn show_service_toast<R: Runtime>(app: &AppHandle<R>, service_id: &str, title: &str, body: &str) {
    let handle = app.clone();
    let service_id = service_id.to_string();
    let open_service = move || {
        tray::show_main(&handle);
        if let Err(err) = activation::activate(&handle, &service_id) {
            log::warn!("could not open {service_id} from a notification: {err}");
        }
        snapshot::publish(&handle);
    };
    if !platform::show_clickable_toast(app, title, body, open_service) {
        show_toast(app, title, body);
    }
}

/// Shows a native toast. Also used by the "send a test notification" button.
pub fn show_toast<R: Runtime>(app: &AppHandle<R>, title: &str, body: &str) {
    if let Err(err) = app.notification().builder().title(title).body(body).show() {
        log::warn!("could not show notification: {err}");
    }
}

/// Cuts `text` to at most `max` characters, adding an ellipsis when shortened.
fn truncate(text: &str, max: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut cut: String = text.chars().take(max.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

#[cfg(test)]
mod tests {
    use super::truncate;

    #[test]
    fn truncates_on_character_boundaries() {
        assert_eq!(truncate("  hello  ", 10), "hello");
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("ছবিছবিছবি", 4), "ছবি…");
    }
}
