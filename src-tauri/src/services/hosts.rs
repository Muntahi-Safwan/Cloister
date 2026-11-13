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

/// Link wrappers that services put around outgoing links: (host, path prefix,
/// query parameter that holds the real address). A host starting with `.`
/// matches any subdomain of it.
const REDIRECTORS: &[(&str, &str, &str)] = &[
    ("google.com", "/url", "q"),
    ("www.google.com", "/url", "q"),
    ("l.facebook.com", "/l.php", "u"),
    ("lm.facebook.com", "/l.php", "u"),
    ("l.messenger.com", "/l.php", "u"),
    ("l.instagram.com", "/", "u"),
    (".safelinks.protection.outlook.com", "/", "url"),
    ("statics.teams.cdn.office.net", "/evergreen-assets/safelinks/", "url"),
    ("www.linkedin.com", "/redir/redirect", "url"),
    ("slack-redir.net", "/link", "url"),
];

/// The real destination of a wrapped link such as `google.com/url?q=…`, if
/// `url` is one and the destination is a web address.
pub fn unwrap_redirect(url: &Url) -> Option<Url> {
    let host = url.host_str()?;
    let (_, _, param) = REDIRECTORS.iter().find(|(h, path, _)| {
        let host_ok = match h.strip_prefix('.') {
            Some(suffix) => host.ends_with(&format!(".{suffix}")),
            None => host == *h,
        };
        host_ok && url.path().starts_with(path)
    })?;
    let target = url.query_pairs().find(|(k, _)| k == param).map(|(_, v)| v.into_owned())?;
    let target = Url::parse(&target).ok()?;
    matches!(target.scheme(), "https" | "http").then_some(target)
}

/// Where a link should open outside Cloister, or `None` when it belongs to the
/// service. Wrapped links are judged by their real destination.
pub fn external_target(url: &Url, domains: &[String]) -> Option<Url> {
    let target = unwrap_redirect(url).unwrap_or_else(|| url.clone());
    match target.scheme() {
        "https" | "http" if !is_internal(&target, domains) => Some(target),
        "mailto" | "tel" => Some(target),
        _ => None,
    }
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
    fn wrapped_links_are_unwrapped() {
        let cases = [
            (
                "https://www.google.com/url?q=https%3A%2F%2Fexample.com%2Fa%3Fb%3D1&sa=D",
                "https://example.com/a?b=1",
            ),
            ("https://l.facebook.com/l.php?u=https%3A%2F%2Fnews.example%2F&h=x", "https://news.example/"),
            ("https://l.messenger.com/l.php?u=https%3A%2F%2Fexample.org", "https://example.org/"),
            (
                "https://nam12.safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com%2F&data=x",
                "https://example.com/",
            ),
            ("https://www.linkedin.com/redir/redirect?url=https%3A%2F%2Fexample.com", "https://example.com/"),
            ("https://slack-redir.net/link?url=http%3A%2F%2Fexample.com", "http://example.com/"),
        ];
        for (wrapped, real) in cases {
            assert_eq!(
                unwrap_redirect(&url(wrapped)).map(|u| u.to_string()),
                Some(real.to_string()),
                "{wrapped}"
            );
        }
    }

    #[test]
    fn only_known_wrappers_with_web_targets_unwrap() {
        assert_eq!(unwrap_redirect(&url("https://www.google.com/search?q=https://x.com")), None);
        assert_eq!(unwrap_redirect(&url("https://evil.example/url?q=https://x.com")), None);
        assert_eq!(unwrap_redirect(&url("https://www.google.com/url?q=javascript:alert(1)")), None);
        assert_eq!(unwrap_redirect(&url("https://www.google.com/url?q=file:///etc/passwd")), None);
        assert_eq!(
            unwrap_redirect(&url("https://safelinks.protection.outlook.com.evil.example/?url=https://x.com")),
            None
        );
    }

    #[test]
    fn external_targets_skip_the_service_itself() {
        let gmail = vec!["mail.google.com".to_string(), "google.com".to_string()];
        let to = |s: &str| external_target(&url(s), &gmail).map(|u| u.to_string());
        assert_eq!(
            to("https://www.google.com/url?q=https://example.com/"),
            Some("https://example.com/".into())
        );
        assert_eq!(to("https://example.com/x"), Some("https://example.com/x".into()));
        assert_eq!(to("https://mail.google.com/mail/u/0/#inbox"), None);
        assert_eq!(to("https://www.google.com/url?q=https://docs.google.com/doc"), None);
        assert_eq!(to("mailto:a@b.com"), Some("mailto:a@b.com".into()));
        assert_eq!(to("about:blank"), None);
        assert_eq!(to("file:///C:/Windows/system32/calc.exe"), None);
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
