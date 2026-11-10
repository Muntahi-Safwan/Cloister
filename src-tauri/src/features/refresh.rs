//! Which hidden pages have grown large enough to be worth reloading.
//!
//! Long-running web apps leak memory: a chat left open for days keeps growing.
//! Reloading a page that nobody is looking at gives that memory back without
//! the user noticing. Only pages that have been out of sight for a while are
//! chosen, so a page someone just switched away from is never reloaded.

use std::collections::HashMap;
use std::time::Duration;

/// A page must have been hidden at least this long before it is refreshed.
pub const MIN_HIDDEN: Duration = Duration::from_secs(10 * 60);

/// A loaded service as the refresher sees it.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: String,
    /// How long since it was last on screen (`None` = never shown).
    pub hidden_for: Option<Duration>,
    /// On screen right now, or suspended by pause mode (already small).
    pub exempt: bool,
}

/// Ids of the services to reload: hidden long enough and above `limit_bytes`.
pub fn pick(candidates: &[Candidate], usage: &HashMap<String, u64>, limit_bytes: u64) -> Vec<String> {
    if limit_bytes == 0 {
        return Vec::new();
    }
    candidates
        .iter()
        .filter(|c| !c.exempt)
        .filter(|c| c.hidden_for.is_none_or(|hidden| hidden >= MIN_HIDDEN))
        .filter(|c| usage.get(&c.id).is_some_and(|&bytes| bytes > limit_bytes))
        .map(|c| c.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MB: u64 = 1024 * 1024;

    fn candidate(id: &str, hidden_minutes: Option<u64>, exempt: bool) -> Candidate {
        Candidate { id: id.into(), hidden_for: hidden_minutes.map(|m| Duration::from_secs(m * 60)), exempt }
    }

    #[test]
    fn picks_large_pages_hidden_long_enough() {
        let usage = HashMap::from([("big".to_string(), 900 * MB), ("small".to_string(), 200 * MB)]);
        let candidates = [candidate("big", Some(30), false), candidate("small", Some(30), false)];
        assert_eq!(pick(&candidates, &usage, 800 * MB), vec!["big".to_string()]);
    }

    #[test]
    fn leaves_recent_on_screen_and_paused_pages_alone() {
        let usage = HashMap::from([("a".to_string(), 900 * MB), ("b".to_string(), 900 * MB)]);
        let candidates = [candidate("a", Some(2), false), candidate("b", Some(60), true)];
        assert!(pick(&candidates, &usage, 800 * MB).is_empty());
    }

    #[test]
    fn a_limit_of_zero_turns_refreshing_off() {
        let usage = HashMap::from([("a".to_string(), 9_000 * MB)]);
        assert!(pick(&[candidate("a", None, false)], &usage, 0).is_empty());
    }
}
