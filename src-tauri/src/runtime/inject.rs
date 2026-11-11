//! Builds the initialization script injected into each service page.

use serde::Serialize;

use crate::services::catalog::Recipe;
use crate::services::ServiceConfig;

/// The bridge source. Its last line is [`BRIDGE_CALL`].
const BRIDGE_JS: &str = include_str!("../../scripts/bridge.js");

/// The call on the last line of `bridge.js`; its argument is replaced with the JSON configuration.
const BRIDGE_CALL: &str = "})(__CLOISTER_CONFIG__);";

/// Configuration passed to the bridge when the page starts.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgeConfig<'a> {
    veil: bool,
    veil_selectors: &'a [&'a str],
    reveal_on_hover: bool,
    /// Shortcuts use Cmd instead of Ctrl on macOS.
    mac: bool,
}

/// Returns the full script for `service`: the bridge with its configuration filled in.
pub fn bridge_script(service: &ServiceConfig, recipe: Option<&Recipe>, veil: bool) -> String {
    let selectors: &[&str] = match recipe {
        Some(r) if !service.options.veil_whole_page => r.veil_selectors,
        _ => &[],
    };
    let config = BridgeConfig {
        veil,
        veil_selectors: selectors,
        reveal_on_hover: true,
        mac: cfg!(target_os = "macos"),
    };
    // Serialising a struct of strings and bools cannot fail.
    let json = serde_json::to_string(&config).unwrap_or_else(|_| "{}".into());
    BRIDGE_JS.replacen(BRIDGE_CALL, &format!("}})({json});"), 1)
}

/// Script that toggles Veil in an already-loaded page.
pub fn set_veil_script(on: bool) -> String {
    format!("window.__CLOISTER__ && window.__CLOISTER__.setVeil({on});")
}

/// Script that opens the find bar inside a service page.
pub fn find_script() -> &'static str {
    "window.__CLOISTER__ && window.__CLOISTER__.find();"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::model::{IconKind, ServiceOptions};

    fn service(whole_page: bool) -> ServiceConfig {
        ServiceConfig {
            id: "wa-1".into(),
            name: "WhatsApp".into(),
            url: "https://web.whatsapp.com/".into(),
            recipe_id: Some("whatsapp".into()),
            icon: IconKind::Initials,
            color: 0,
            icon_url: None,
            muted_until: None,
            zoom: 1.0,
            options: ServiceOptions { veil_whole_page: whole_page, ..ServiceOptions::default() },
        }
    }

    /// Guards against the bridge silently never running (for example after a
    /// formatter changes how the file ends): the call must carry the config.
    #[test]
    fn bridge_calls_itself_with_the_config() {
        assert!(BRIDGE_JS.trim_end().ends_with(BRIDGE_CALL));
        let recipe = crate::services::catalog::find("whatsapp");
        let script = bridge_script(&service(false), recipe, true);
        assert!(script.contains("})({\"veil\":true"));
        assert!(!script.contains("__CLOISTER_CONFIG__);"));
        assert!(script.contains("\"veil\":true"));
        assert!(script.contains("#pane-side"));
    }

    #[test]
    fn whole_page_mode_sends_no_selectors() {
        let recipe = crate::services::catalog::find("whatsapp");
        let script = bridge_script(&service(true), recipe, false);
        assert!(script.contains("\"veilSelectors\":[]"));
    }
}
