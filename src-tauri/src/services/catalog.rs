//! Built-in service recipes.
//!
//! A recipe describes how Cloister should treat a well-known website: where it
//! starts, which hosts belong to it (everything else opens in the default
//! browser), how to read its unread count and which regions Veil should blur.
//!
//! To add a service, append a [`Recipe`] to [`RECIPES`] and open a pull request.
//! See `docs/recipes.md` for guidance.

use serde::Serialize;

use super::model::UserAgentPreset;

/// How a service exposes its unread count in the page title.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UnreadRule {
    /// A number in parentheses anywhere in the title: `(3) WhatsApp`, `Inbox (12) - Gmail`.
    TitleParentheses,
    /// The service does not expose a count in its title.
    None,
}

/// Static description of a well-known service.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recipe {
    /// Stable recipe id.
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Short category shown in the catalog (Chat, Mail, ...).
    pub category: &'static str,
    /// Start URL.
    pub url: &'static str,
    /// Hosts that stay inside the service. Subdomains of each entry also match.
    pub hosts: &'static [&'static str],
    /// How unread counts are read.
    pub unread: UnreadRule,
    /// Regions Veil blurs. Regions containing list items (chats, messages) are
    /// blurred row by row so hovering reveals one row; others as a whole.
    /// An empty list blurs the whole page.
    pub veil_selectors: &'static [&'static str],
    /// User agent the service needs to work correctly.
    pub user_agent: UserAgentPreset,
}

/// Every recipe that ships with Cloister, in catalog order.
pub const RECIPES: &[Recipe] = &[
    Recipe {
        id: "whatsapp",
        name: "WhatsApp",
        category: "Chat",
        url: "https://web.whatsapp.com/",
        hosts: &["whatsapp.com", "whatsapp.net"],
        unread: UnreadRule::TitleParentheses,
        // Chat list and messages are revealed row by row; the open chat's header as a whole.
        veil_selectors: &["#pane-side", "#main header", "#main", "[data-icon='default-user']"],
        user_agent: UserAgentPreset::Default,
    },
    Recipe {
        id: "messenger",
        name: "Messenger",
        category: "Chat",
        url: "https://www.messenger.com/",
        hosts: &["messenger.com", "facebook.com", "fbcdn.net"],
        unread: UnreadRule::TitleParentheses,
        veil_selectors: &[],
        user_agent: UserAgentPreset::Default,
    },
    Recipe {
        id: "discord",
        name: "Discord",
        category: "Community",
        url: "https://discord.com/app",
        hosts: &["discord.com", "discordapp.com", "discord.gg"],
        unread: UnreadRule::TitleParentheses,
        veil_selectors: &["nav[aria-label]", "main", "section[aria-label]", "aside", "[class*='sidebar']"],
        user_agent: UserAgentPreset::Default,
    },
    Recipe {
        id: "gmail",
        name: "Gmail",
        category: "Mail",
        url: "https://mail.google.com/",
        hosts: &["mail.google.com", "accounts.google.com", "google.com", "gstatic.com"],
        unread: UnreadRule::TitleParentheses,
        veil_selectors: &["[role='main']", "[role='navigation']", "img"],
        user_agent: UserAgentPreset::Default,
    },
    Recipe {
        id: "outlook",
        name: "Outlook",
        category: "Mail",
        url: "https://outlook.live.com/mail/",
        hosts: &[
            "outlook.live.com",
            "outlook.office.com",
            "live.com",
            "microsoftonline.com",
            "microsoft.com",
        ],
        unread: UnreadRule::TitleParentheses,
        veil_selectors: &[],
        user_agent: UserAgentPreset::Default,
    },
    Recipe {
        id: "telegram",
        name: "Telegram",
        category: "Chat",
        url: "https://web.telegram.org/a/",
        hosts: &["telegram.org", "t.me"],
        unread: UnreadRule::TitleParentheses,
        veil_selectors: &[],
        user_agent: UserAgentPreset::Default,
    },
    Recipe {
        id: "slack",
        name: "Slack",
        category: "Work chat",
        url: "https://app.slack.com/client",
        hosts: &["slack.com", "slack-edge.com"],
        unread: UnreadRule::None,
        veil_selectors: &[],
        user_agent: UserAgentPreset::Default,
    },
    Recipe {
        id: "teams",
        name: "Microsoft Teams",
        category: "Work chat",
        url: "https://teams.microsoft.com/",
        hosts: &["teams.microsoft.com", "teams.live.com", "microsoftonline.com", "microsoft.com", "live.com"],
        unread: UnreadRule::TitleParentheses,
        veil_selectors: &[],
        user_agent: UserAgentPreset::Default,
    },
    Recipe {
        id: "facebook",
        name: "Facebook",
        category: "Social",
        url: "https://www.facebook.com/",
        hosts: &["facebook.com", "fbcdn.net"],
        unread: UnreadRule::TitleParentheses,
        veil_selectors: &[],
        user_agent: UserAgentPreset::Default,
    },
    Recipe {
        id: "linkedin",
        name: "LinkedIn",
        category: "Social",
        url: "https://www.linkedin.com/messaging/",
        hosts: &["linkedin.com", "licdn.com"],
        unread: UnreadRule::TitleParentheses,
        veil_selectors: &[],
        user_agent: UserAgentPreset::Default,
    },
    Recipe {
        id: "proton-mail",
        name: "Proton Mail",
        category: "Mail",
        url: "https://mail.proton.me/",
        hosts: &["proton.me"],
        unread: UnreadRule::TitleParentheses,
        veil_selectors: &[],
        user_agent: UserAgentPreset::Default,
    },
];

/// Looks up a recipe by id.
pub fn find(id: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn recipe_ids_are_unique() {
        let ids: HashSet<_> = RECIPES.iter().map(|r| r.id).collect();
        assert_eq!(ids.len(), RECIPES.len());
    }

    #[test]
    fn every_recipe_url_is_https_and_owned_by_its_hosts() {
        for recipe in RECIPES {
            let url = url::Url::parse(recipe.url).expect(recipe.id);
            assert_eq!(url.scheme(), "https", "{}", recipe.id);
            let host = url.host_str().unwrap();
            assert!(
                recipe.hosts.iter().any(|h| host == *h || host.ends_with(&format!(".{h}"))),
                "{} start URL host is not in its host list",
                recipe.id
            );
        }
    }

    #[test]
    fn find_returns_known_recipes() {
        assert_eq!(find("whatsapp").map(|r| r.name), Some("WhatsApp"));
        assert!(find("does-not-exist").is_none());
    }
}
