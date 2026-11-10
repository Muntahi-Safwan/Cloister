//! Recent notifications, kept in memory only.
//!
//! The list is never written to disk and is cleared when Cloister quits.

use std::collections::VecDeque;

use serde::Serialize;

use crate::services::ServiceId;

/// How many notifications are kept.
pub const HISTORY_LIMIT: usize = 50;

/// What happened to a notification when it arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Delivery {
    /// Shown as a toast.
    Shown,
    /// Held for the focus summary.
    Held,
    /// Silenced by mute, quiet hours or the service's notification switch.
    Silenced,
}

/// One remembered notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub service_id: ServiceId,
    pub title: String,
    pub body: String,
    pub at_ms: u64,
    pub delivery: Delivery,
}

/// A bounded list of recent notifications, newest first.
#[derive(Debug, Default)]
pub struct History {
    items: VecDeque<HistoryItem>,
}

impl History {
    /// Adds a notification, dropping the oldest one past [`HISTORY_LIMIT`].
    pub fn push(&mut self, item: HistoryItem) {
        self.items.push_front(item);
        self.items.truncate(HISTORY_LIMIT);
    }

    /// All items, newest first.
    pub fn items(&self) -> Vec<HistoryItem> {
        self.items.iter().cloned().collect()
    }

    /// When the newest notification arrived, if any.
    pub fn latest_ms(&self) -> Option<u64> {
        self.items.front().map(|item| item.at_ms)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Forgets everything from one service (used when it is removed or erased).
    pub fn forget_service(&mut self, service_id: &str) {
        self.items.retain(|item| item.service_id != service_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(i: u64) -> HistoryItem {
        HistoryItem {
            service_id: format!("s{}", i % 2),
            title: format!("t{i}"),
            body: String::new(),
            at_ms: i,
            delivery: Delivery::Shown,
        }
    }

    #[test]
    fn keeps_newest_first_and_caps_length() {
        let mut history = History::default();
        for i in 0..(HISTORY_LIMIT as u64 + 5) {
            history.push(item(i));
        }
        assert_eq!(history.len(), HISTORY_LIMIT);
        assert_eq!(history.items()[0].at_ms, HISTORY_LIMIT as u64 + 4);
    }

    #[test]
    fn forgets_one_service() {
        let mut history = History::default();
        (0..4).for_each(|i| history.push(item(i)));
        history.forget_service("s0");
        assert!(history.items().iter().all(|i| i.service_id == "s1"));
        history.clear();
        assert!(history.is_empty());
    }
}
