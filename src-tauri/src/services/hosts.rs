//! Decides which URLs belong to a service.
//!
//! Navigation to a host owned by the service stays inside Cloister; anything
//! else is handed to the user's default browser.

use url::Url;

/// Returns `true` when `host` equals `domain` or is a subdomain of it.
pub fn host_matches(host: &str, domain: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    let domain = domain.trim_end_matches('.').to_ascii_lowercase();
    host == domain || host.ends_with(&format!(".{domain}"))
}

/// Returns `true` when `url` should load inside the service rather than externally.
///
/// Non-web schemes such as `about:` and `blob:` are kept internal because
/// sites use them for in-page features; `mailto:` and similar are external.
pub fn is_internal(url: &Url, domains: &[String]) -> bool {
    match url.scheme() {
        "about" | "blob" | "data" => true,
        "https" | "http" => url.host_str().is_some_and(|host| domains.iter().any(|d| host_matches(host, d))),
        _ => false,
    }
}

/// Returns `true` when a top-level navigation should leave Cloister.
///
/// Web navigations always stay inside the service so sign-in redirects
/// (for example to a single sign-on provider) keep working. Only non-web
/// schemes like `mailto:`, `tel:` or app deep links are handed to the OS.
/// Links that open a *new window* are handled separately with [`is_internal`].
pub fn navigation_leaves_app(url: &Url) -> bool {
    !matches!(url.scheme(), "https" | "http" | "about" | "blob" | "data")
}

/// Returns `true` for URLs that belong to Cloister itself. Service pages must
/// never navigate there, because Cloister's own UI has more permissions.
pub fn is_app_origin(url: &Url) -> bool {
    matches!(url.scheme(), "tauri" | "ipc" | "asset")
        || url.host_str().is_some_and(|h| {
            matches!(h, "tauri.localhost" | "ipc.localhost" | "asset.localhost" | "localhost")
        })
}

/// Builds the list of domains for a service: the recipe hosts when there is a
/// recipe, otherwise exactly the start URL's host (and its subdomains).
///
/// Custom services deliberately do not widen to a guessed parent domain:
/// `chat.example.co.uk` must not grant anything to all of `co.uk`.
pub fn domains_for(start: &Url, recipe_hosts: Option<&[&str]>) -> Vec<String> {
    if let Some(hosts) = recipe_hosts {
        return hosts.iter().map(|h| (*h).to_string()).collect();
    }
    start.host_str().map(str::to_string).into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn subdomains_match_but_lookalikes_do_not() {
        assert!(host_matches("web.whatsapp.com", "whatsapp.com"));
        assert!(host_matches("WhatsApp.com", "whatsapp.com"));
        assert!(!host_matches("evilwhatsapp.com", "whatsapp.com"));
        assert!(!host_matches("whatsapp.com.evil.io", "whatsapp.com"));
    }

    #[test]
    fn classifies_navigation_targets() {
        let domains = vec!["discord.com".to_string()];
        assert!(is_internal(&url("https://discord.com/channels/1"), &domains));
        assert!(is_internal(&url("about:blank"), &domains));
        assert!(!is_internal(&url("https://github.com/"), &domains));
        assert!(!is_internal(&url("mailto:someone@example.com"), &domains));
    }

    #[test]
    fn only_non_web_navigation_leaves_the_app() {
        assert!(!navigation_leaves_app(&url("https://login.microsoftonline.com/")));
        assert!(navigation_leaves_app(&url("mailto:a@b.com")));
        assert!(navigation_leaves_app(&url("tel:+880123")));
    }

    #[test]
    fn app_origins_are_recognised() {
        assert!(is_app_origin(&url("http://tauri.localhost/")));
        assert!(is_app_origin(&url("tauri://localhost/")));
        assert!(!is_app_origin(&url("https://web.whatsapp.com/")));
    }

    #[test]
    fn custom_services_use_their_exact_host() {
        let start = url("https://chat.example.co.uk/app");
        assert_eq!(domains_for(&start, None), vec!["chat.example.co.uk".to_string()]);
        assert_eq!(domains_for(&start, Some(&["a.com", "b.com"])), vec!["a.com", "b.com"]);
    }
}
