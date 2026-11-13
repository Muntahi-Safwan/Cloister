//! Grants service pages access to a handful of narrow backend commands.
//!
//! Third-party pages run untrusted code. Each service webview gets a runtime
//! capability scoped to its own label and its own domains that allows only
//! `bridge_notify`, `bridge_icon`, `bridge_shortcut` and `bridge_open_link`.
//! Nothing else in the backend is reachable from a service page.

use tauri::ipc::CapabilityBuilder;
use tauri::{AppHandle, Manager, Runtime};

use crate::error::AppResult;
use crate::state::AppState;

/// Permissions granted to every service page.
const BRIDGE_PERMISSIONS: &[&str] =
    &["allow-bridge-notify", "allow-bridge-icon", "allow-bridge-shortcut", "allow-bridge-open-link"];

/// Registers the bridge capability for the webview `label` on `domains`.
///
/// Capabilities cannot be removed at runtime, so each label is granted once;
/// reloading an unloaded service reuses the existing grant.
pub fn grant_bridge<R: Runtime>(
    app: &AppHandle<R>,
    label: &str,
    domains: &[String],
    port: Option<u16>,
) -> AppResult<()> {
    if !app.state::<AppState>().session().bridge_granted.insert(label.to_string()) {
        return Ok(());
    }
    let mut capability = CapabilityBuilder::new(format!("bridge-{label}")).local(false).webview(label);
    for pattern in remote_patterns(domains, port) {
        capability = capability.remote(pattern);
    }
    for permission in BRIDGE_PERMISSIONS {
        capability = capability.permission(*permission);
    }
    app.add_capability(capability)?;
    Ok(())
}

/// URL patterns that match each domain and all of its subdomains, on the
/// default port or on `port` when the service uses a custom one.
fn remote_patterns(domains: &[String], port: Option<u16>) -> Vec<String> {
    let port = port.map(|p| format!(":{p}")).unwrap_or_default();
    domains.iter().flat_map(|d| [format!("https://{d}{port}/*"), format!("https://*.{d}{port}/*")]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns_cover_apex_and_subdomains() {
        let patterns = remote_patterns(&["whatsapp.com".to_string()], None);
        assert_eq!(patterns, vec!["https://whatsapp.com/*", "https://*.whatsapp.com/*"]);
    }

    /// Checks the patterns with the same matcher Tauri's ACL uses at runtime.
    #[test]
    fn patterns_match_service_pages_and_nothing_else() {
        use tauri::utils::acl::RemoteUrlPattern;

        let patterns: Vec<RemoteUrlPattern> =
            remote_patterns(&["whatsapp.com".to_string()], None).iter().map(|p| p.parse().unwrap()).collect();
        let allowed = |url: &str| {
            let url = url::Url::parse(url).unwrap();
            patterns.iter().any(|p| p.test(&url))
        };

        assert!(allowed("https://web.whatsapp.com/"));
        assert!(allowed("https://whatsapp.com/download?x=1"));
        assert!(!allowed("https://evil.com/"));
        assert!(!allowed("https://whatsapp.com.evil.com/"));
        assert!(!allowed("http://web.whatsapp.com/"));
    }

    #[test]
    fn custom_ports_are_matched() {
        use tauri::utils::acl::RemoteUrlPattern;

        let patterns: Vec<RemoteUrlPattern> = remote_patterns(&["chat.example.com".to_string()], Some(8443))
            .iter()
            .map(|p| p.parse().unwrap())
            .collect();
        let url = url::Url::parse("https://chat.example.com:8443/room").unwrap();
        assert!(patterns.iter().any(|p| p.test(&url)));
    }
}
