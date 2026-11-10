//! Decides what happens to a notification coming from a service page.
//!
//! The decision is a pure function so it can be tested without a running app;
//! the runtime then shows, holds or drops the notification accordingly.

use super::focus::is_vip;

/// The context a notification arrives in.
#[derive(Debug, Clone, Copy)]
pub struct NotifyContext<'a> {
    /// The service's display name.
    pub service_name: &'a str,
    /// Whether the user enabled notifications for this service.
    pub enabled: bool,
    /// Whether the service is muted right now.
    pub muted: bool,
    /// Whether quiet hours are active right now.
    pub quiet: bool,
    /// Whether Veil is on.
    pub veil: bool,
    /// Whether toasts should be redacted while Veil is on.
    pub redact_in_veil: bool,
    /// Whether a focus session is running.
    pub focus: bool,
    /// Names allowed to break through focus.
    pub vip_names: &'a [String],
}

/// What to do with an incoming notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// Show a native toast with this content.
    Show { title: String, body: String },
    /// Keep it for the focus summary.
    Hold,
    /// Discard it (disabled, muted, or quiet hours). Unread badges still count.
    Drop,
}

/// Routes a notification with the given `title` and `body`.
pub fn route(ctx: NotifyContext<'_>, title: &str, body: &str) -> Route {
    let vip = is_vip(title, ctx.vip_names);
    if !ctx.enabled || ctx.muted || (ctx.quiet && !vip) {
        return Route::Drop;
    }
    if ctx.focus && !vip {
        return Route::Hold;
    }
    if ctx.veil && ctx.redact_in_veil {
        return Route::Show { title: ctx.service_name.to_string(), body: "New message".to_string() };
    }
    let title = if title.trim().is_empty() { ctx.service_name } else { title };
    Route::Show { title: title.to_string(), body: body.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(vips: &'a [String]) -> NotifyContext<'a> {
        NotifyContext {
            service_name: "WhatsApp",
            enabled: true,
            muted: false,
            quiet: false,
            veil: false,
            redact_in_veil: true,
            focus: false,
            vip_names: vips,
        }
    }

    #[test]
    fn shows_by_default() {
        assert_eq!(
            route(ctx(&[]), "Ayesha", "hi"),
            Route::Show { title: "Ayesha".into(), body: "hi".into() }
        );
    }

    #[test]
    fn veil_redacts_sender_and_text() {
        let c = NotifyContext { veil: true, ..ctx(&[]) };
        assert_eq!(
            route(c, "Ayesha", "secret"),
            Route::Show { title: "WhatsApp".into(), body: "New message".into() }
        );
    }

    #[test]
    fn focus_holds_unless_vip() {
        let vips = vec!["Mom".to_string()];
        let c = NotifyContext { focus: true, ..ctx(&vips) };
        assert_eq!(route(c, "Design team", "x"), Route::Hold);
        assert!(matches!(route(c, "Mom", "call me"), Route::Show { .. }));
    }

    #[test]
    fn disabled_services_drop_everything() {
        let c = NotifyContext { enabled: false, ..ctx(&[]) };
        assert_eq!(route(c, "a", "b"), Route::Drop);
    }

    #[test]
    fn muted_services_drop_even_vips() {
        let vips = vec!["Mom".to_string()];
        let c = NotifyContext { muted: true, ..ctx(&vips) };
        assert_eq!(route(c, "Mom", "x"), Route::Drop);
    }

    #[test]
    fn quiet_hours_drop_unless_vip() {
        let vips = vec!["Mom".to_string()];
        let c = NotifyContext { quiet: true, ..ctx(&vips) };
        assert_eq!(route(c, "Design team", "x"), Route::Drop);
        assert!(matches!(route(c, "Mom", "call me"), Route::Show { .. }));
    }

    #[test]
    fn empty_titles_use_the_service_name() {
        assert_eq!(
            route(ctx(&[]), " ", "body"),
            Route::Show { title: "WhatsApp".into(), body: "body".into() }
        );
    }
}
