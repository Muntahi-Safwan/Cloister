//! Data types that describe a configured service.
//!
//! These types are persisted to `services.json` and sent to the frontend, so
//! they use `camelCase` field names on the wire.

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{AppError, AppResult};

/// Stable identifier of a configured service, e.g. `whatsapp-k3x9`.
pub type ServiceId = String;

/// How a service's sidebar icon is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum IconKind {
    /// Two-letter initials derived from the service name.
    #[default]
    Initials,
    /// The website's own favicon.
    Favicon,
}

/// What happens to a service while it is not on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum BackgroundMode {
    /// Stay loaded so notifications keep arriving. Uses a little memory.
    #[default]
    KeepNotifying,
    /// Close the page after a period of inactivity and reload it on demand.
    UnloadWhenIdle,
}

/// Which browser the service should believe it is talking to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum UserAgentPreset {
    /// The engine's own user agent (Microsoft Edge on Windows).
    #[default]
    Default,
    /// A current desktop Chrome user agent.
    Chrome,
    /// A current desktop Firefox user agent.
    Firefox,
}

/// Per-service behaviour switches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ServiceOptions {
    /// Forward the site's notifications as native toasts.
    pub notifications: bool,
    /// Read the unread count from the page title, e.g. `(3) Inbox`.
    pub unread_from_title: bool,
    /// Blur the entire page in Veil instead of selected regions.
    pub veil_whole_page: bool,
    /// Hide this service from the sidebar during focus sessions.
    pub hide_during_focus: bool,
    /// Background behaviour.
    pub background: BackgroundMode,
    /// User agent preset.
    pub user_agent: UserAgentPreset,
}

impl Default for ServiceOptions {
    fn default() -> Self {
        Self {
            notifications: true,
            unread_from_title: true,
            veil_whole_page: false,
            hide_during_focus: false,
            background: BackgroundMode::KeepNotifying,
            user_agent: UserAgentPreset::Default,
        }
    }
}

/// A service the user has added to Cloister.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceConfig {
    /// Stable unique id. Also used to name the service's browser profile.
    pub id: ServiceId,
    /// Display name shown in the sidebar.
    pub name: String,
    /// The page that is loaded when the service starts.
    pub url: String,
    /// Catalog recipe this service was created from, if any.
    #[serde(default)]
    pub recipe_id: Option<String>,
    /// Icon style.
    #[serde(default)]
    pub icon: IconKind,
    /// Index into the frontend's tile colour palette.
    #[serde(default)]
    pub color: u8,
    /// The site's own icon, reported by the page once it has loaded.
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Notifications are silenced until this Unix time in ms (`u64::MAX` = until unmuted).
    #[serde(default)]
    pub muted_until: Option<u64>,
    /// Page zoom factor, 1.0 = 100 %.
    #[serde(default = "default_zoom")]
    pub zoom: f64,
    /// Behaviour switches.
    #[serde(default)]
    pub options: ServiceOptions,
}

fn default_zoom() -> f64 {
    1.0
}

impl ServiceConfig {
    /// Whether notifications from this service are muted at `now_ms`.
    pub fn is_muted(&self, now_ms: u64) -> bool {
        self.muted_until.is_some_and(|until| now_ms < until)
    }

    /// Parses the stored start URL.
    pub fn parsed_url(&self) -> AppResult<Url> {
        Url::parse(&self.url).map_err(|e| AppError::InvalidInput(format!("invalid URL: {e}")))
    }
}

/// Input for creating a new service, sent by the frontend.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewService {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub recipe_id: Option<String>,
    #[serde(default)]
    pub icon: IconKind,
    #[serde(default)]
    pub color: u8,
    #[serde(default)]
    pub options: ServiceOptions,
}

/// Validates a user-supplied start URL. Only `https` URLs with a real host are accepted.
pub fn validate_url(raw: &str) -> AppResult<Url> {
    let url = Url::parse(raw.trim())
        .map_err(|_| AppError::InvalidInput("Enter a full address, like https://chat.example.com".into()))?;
    if url.scheme() != "https" {
        return Err(AppError::InvalidInput("Only secure https:// addresses are supported".into()));
    }
    match url.host_str() {
        Some(host) if host.contains('.') => Ok(url),
        _ => Err(AppError::InvalidInput("The address needs a domain name, like example.com".into())),
    }
}

/// A zoom change requested from the keyboard or menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ZoomStep {
    In,
    Out,
    Reset,
}

impl ZoomStep {
    const MIN: f64 = 0.5;
    const MAX: f64 = 2.0;
    const STEP: f64 = 0.1;

    /// Applies this step to `current`, keeping the result between 50 % and 200 %.
    pub fn apply(self, current: f64) -> f64 {
        let next = match self {
            ZoomStep::In => current + Self::STEP,
            ZoomStep::Out => current - Self::STEP,
            ZoomStep::Reset => 1.0,
        };
        // Round to one decimal so repeated steps do not drift (1.1 + 0.1 = 1.2000000000000002).
        ((next.clamp(Self::MIN, Self::MAX)) * 10.0).round() / 10.0
    }
}

/// Validates a display name: trimmed, not empty, at most 60 characters.
pub fn validate_name(raw: &str) -> AppResult<String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput("The name cannot be empty".into()));
    }
    Ok(name.chars().take(60).collect())
}

/// Largest icon URL accepted from a page (data URLs can be big).
const MAX_ICON_URL_LEN: usize = 64 * 1024;

/// Accepts a site icon URL only if it is `https` or an inline image, and not huge.
pub fn sanitize_icon_url(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.len() > MAX_ICON_URL_LEN {
        return None;
    }
    if raw.starts_with("data:image/") {
        return Some(raw.to_string());
    }
    let url = Url::parse(raw).ok()?;
    (url.scheme() == "https").then(|| url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_steps_are_clamped_and_rounded() {
        assert_eq!(ZoomStep::In.apply(1.0), 1.1);
        assert_eq!(ZoomStep::In.apply(1.1), 1.2);
        assert_eq!(ZoomStep::Out.apply(0.5), 0.5);
        assert_eq!(ZoomStep::In.apply(2.0), 2.0);
        assert_eq!(ZoomStep::Reset.apply(1.7), 1.0);
    }

    #[test]
    fn mute_state_follows_time() {
        let mut cfg: ServiceConfig =
            serde_json::from_str(r#"{"id":"x","name":"X","url":"https://x.com"}"#).unwrap();
        assert!(!cfg.is_muted(100));
        assert_eq!(cfg.zoom, 1.0);
        cfg.muted_until = Some(200);
        assert!(cfg.is_muted(100));
        assert!(!cfg.is_muted(200));
    }

    #[test]
    fn names_are_trimmed_and_required() {
        assert_eq!(validate_name("  Work  ").unwrap(), "Work");
        assert!(validate_name("   ").is_err());
        assert_eq!(validate_name(&"a".repeat(80)).unwrap().len(), 60);
    }

    #[test]
    fn icon_urls_are_sanitized() {
        assert!(sanitize_icon_url("https://static.whatsapp.net/icon.png").is_some());
        assert!(sanitize_icon_url("data:image/png;base64,AAAA").is_some());
        assert!(sanitize_icon_url("http://insecure.com/i.png").is_none());
        assert!(sanitize_icon_url("javascript:alert(1)").is_none());
        assert!(sanitize_icon_url(&"a".repeat(70_000)).is_none());
    }

    #[test]
    fn accepts_https_with_domain() {
        assert!(validate_url("https://web.whatsapp.com").is_ok());
        assert!(validate_url("  https://chat.example.com/app  ").is_ok());
    }

    #[test]
    fn rejects_insecure_or_hostless_urls() {
        assert!(validate_url("http://example.com").is_err());
        assert!(validate_url("https://localhost").is_err());
        assert!(validate_url("not a url").is_err());
        assert!(validate_url("file:///C:/secret.txt").is_err());
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let json = r#"{"id":"x","name":"X","url":"https://x.com"}"#;
        let cfg: ServiceConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.options, ServiceOptions::default());
        assert_eq!(cfg.icon, IconKind::Initials);
    }
}
