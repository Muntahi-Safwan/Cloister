//! Decides what to do after the user has been away from Cloister.

use std::time::Duration;

/// Automatic privacy actions after the window has been out of focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AwayPolicy {
    /// Lock after this long away (`None` = never).
    pub lock_after: Option<Duration>,
    /// Turn on Veil after this long away (`None` = never).
    pub veil_after: Option<Duration>,
}

/// Actions due now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AwayActions {
    pub lock: bool,
    pub veil: bool,
}

/// Returns which actions are due after being away for `away`.
/// Actions that are already in effect are not repeated.
pub fn due(policy: AwayPolicy, away: Duration, locked: bool, veiled: bool) -> AwayActions {
    let reached = |limit: Option<Duration>| limit.is_some_and(|l| away >= l);
    AwayActions { lock: !locked && reached(policy.lock_after), veil: !veiled && reached(policy.veil_after) }
}

/// Converts a minutes setting where 0 means "off".
pub fn minutes(value: u32) -> Option<Duration> {
    (value > 0).then(|| Duration::from_secs(u64::from(value) * 60))
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: AwayPolicy =
        AwayPolicy { lock_after: Some(Duration::from_secs(300)), veil_after: Some(Duration::from_secs(60)) };

    #[test]
    fn actions_fire_once_their_time_is_reached() {
        assert_eq!(due(POLICY, Duration::from_secs(30), false, false), AwayActions::default());
        assert_eq!(
            due(POLICY, Duration::from_secs(90), false, false),
            AwayActions { lock: false, veil: true }
        );
        assert_eq!(
            due(POLICY, Duration::from_secs(600), false, false),
            AwayActions { lock: true, veil: true }
        );
    }

    #[test]
    fn nothing_repeats_and_zero_means_off() {
        assert_eq!(due(POLICY, Duration::from_secs(600), true, true), AwayActions::default());
        assert_eq!(minutes(0), None);
        assert_eq!(minutes(5), Some(Duration::from_secs(300)));
    }
}
