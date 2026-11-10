//! Focus sessions: a timed period during which notifications are held.

use serde::Serialize;

use crate::services::ServiceId;

/// A notification held back during a focus session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeldNotification {
    pub service_id: ServiceId,
    pub title: String,
    pub body: String,
    /// Unix time in milliseconds when the notification arrived.
    pub at_ms: u64,
}

/// A running focus session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSession {
    started_at_ms: u64,
    ends_at_ms: u64,
    held: Vec<HeldNotification>,
}

impl FocusSession {
    /// Starts a session of `minutes` beginning at `now_ms`.
    pub fn start(now_ms: u64, minutes: u32) -> Self {
        Self { started_at_ms: now_ms, ends_at_ms: now_ms + u64::from(minutes) * 60_000, held: Vec::new() }
    }

    /// Whether the session is still running at `now_ms`.
    pub fn is_running(&self, now_ms: u64) -> bool {
        now_ms < self.ends_at_ms
    }

    /// Stores a notification to show in the end-of-session summary.
    pub fn hold(&mut self, notification: HeldNotification) {
        self.held.push(notification);
    }

    /// Number of notifications held so far.
    pub fn held_count(&self) -> usize {
        self.held.len()
    }

    /// Ends the session and returns everything that was held, oldest first.
    pub fn finish(self) -> Vec<HeldNotification> {
        self.held
    }

    /// Public, serialisable view of the session for the frontend at `now_ms`.
    pub fn view(&self, now_ms: u64) -> FocusView {
        FocusView {
            started_at_ms: self.started_at_ms,
            ends_at_ms: self.ends_at_ms,
            held: self.held.len(),
            expired: !self.is_running(now_ms),
        }
    }
}

/// What the frontend needs to render the focus timer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusView {
    pub started_at_ms: u64,
    pub ends_at_ms: u64,
    pub held: usize,
    /// The timer ran out but the UI has not shown the summary yet.
    pub expired: bool,
}

/// Returns `true` when `title` mentions one of the VIP names (case-insensitive).
///
/// Notification titles are usually the sender or chat name, so this lets
/// specific people break through a focus session.
pub fn is_vip(title: &str, vip_names: &[String]) -> bool {
    let title = title.to_lowercase();
    vip_names.iter().any(|name| !name.is_empty() && title.contains(&name.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_runs_for_requested_minutes() {
        let session = FocusSession::start(1_000, 25);
        assert!(session.is_running(1_000 + 24 * 60_000));
        assert!(!session.is_running(1_000 + 25 * 60_000));
    }

    #[test]
    fn held_notifications_are_returned_in_order() {
        let mut session = FocusSession::start(0, 25);
        for i in 0..3 {
            session.hold(HeldNotification {
                service_id: "s".into(),
                title: format!("t{i}"),
                body: String::new(),
                at_ms: i,
            });
        }
        assert_eq!(session.held_count(), 3);
        let titles: Vec<_> = session.finish().into_iter().map(|n| n.title).collect();
        assert_eq!(titles, ["t0", "t1", "t2"]);
    }

    #[test]
    fn vip_matching_is_case_insensitive_substring() {
        let vips = vec!["Ayesha".to_string()];
        assert!(is_vip("ayesha rahman", &vips));
        assert!(!is_vip("Design team", &vips));
        assert!(!is_vip("anything", &[String::new()]));
    }
}
