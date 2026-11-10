//! Extracts unread counts from page titles.

use super::catalog::UnreadRule;

/// Returns the unread count encoded in `title` according to `rule`.
///
/// Titles without a count yield `0`. The first parenthesised number wins, so
/// `Inbox (12) - Gmail` gives `12` and `(3) WhatsApp` gives `3`.
pub fn parse_title(title: &str, rule: UnreadRule) -> u32 {
    match rule {
        UnreadRule::None => 0,
        UnreadRule::TitleParentheses => first_parenthesised_number(title).unwrap_or(0),
    }
}

/// Finds the first `(N)` group in `text` where `N` is a non-negative integer.
/// A trailing `+` inside the parentheses (`(99+)`) is accepted.
fn first_parenthesised_number(text: &str) -> Option<u32> {
    let mut rest = text;
    while let Some(open) = rest.find('(') {
        let after = &rest[open + 1..];
        let Some(close) = after.find(')') else { break };
        let inner = after[..close].trim().trim_end_matches('+');
        if !inner.is_empty() && inner.chars().all(|c| c.is_ascii_digit()) {
            return inner.parse().ok();
        }
        rest = &after[close + 1..];
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_common_title_formats() {
        let rule = UnreadRule::TitleParentheses;
        assert_eq!(parse_title("(3) WhatsApp", rule), 3);
        assert_eq!(parse_title("Inbox (12) - me@example.com - Gmail", rule), 12);
        assert_eq!(parse_title("(99+) Discord", rule), 99);
    }

    #[test]
    fn ignores_non_numeric_parentheses() {
        let rule = UnreadRule::TitleParentheses;
        assert_eq!(parse_title("Discord (beta) (4)", rule), 4);
        assert_eq!(parse_title("Messenger", rule), 0);
        assert_eq!(parse_title("Broken (title", rule), 0);
    }

    #[test]
    fn none_rule_always_returns_zero() {
        assert_eq!(parse_title("(5) Slack", UnreadRule::None), 0);
    }
}
