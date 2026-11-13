//! Limits how often a service page may open links in the default browser.
//!
//! The bridge only asks right after a real click, but a page runs its own code
//! and could call the bridge directly. This keeps a misbehaving page from
//! flooding the browser with tabs.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// At most this many links...
pub const MAX_LINKS: usize = 5;
/// ...within this window.
pub const WINDOW: Duration = Duration::from_secs(10);

/// Records an attempt at `now` and returns whether it is allowed.
pub fn allow(recent: &mut VecDeque<Instant>, now: Instant) -> bool {
    while recent.front().is_some_and(|&t| now.duration_since(t) >= WINDOW) {
        recent.pop_front();
    }
    if recent.len() >= MAX_LINKS {
        return false;
    }
    recent.push_back(now);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_a_few_then_refuses_until_the_window_passes() {
        let start = Instant::now();
        let mut recent = VecDeque::new();
        for i in 0..MAX_LINKS {
            assert!(allow(&mut recent, start + Duration::from_millis(i as u64)), "link {i}");
        }
        assert!(!allow(&mut recent, start + Duration::from_secs(1)));
        assert!(allow(&mut recent, start + WINDOW + Duration::from_millis(10)));
    }
}
