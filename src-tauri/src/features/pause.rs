//! When a paused background service should sleep and when it should wake.
//!
//! A service in "pause in background" mode is suspended while hidden: its
//! scripts stop and the system can reclaim its memory. Every few minutes it is
//! woken briefly so it can reconnect, update its unread count and deliver any
//! new notifications, then it is suspended again. Opening it wakes it for good.
//!
//! This module only decides; `runtime::pause` performs the actions.

use std::time::{Duration, Instant};

/// How long a woken service stays awake to catch up before it is paused again.
pub const CHECK_DURATION: Duration = Duration::from_secs(25);

/// Where a hidden paused-mode service is in its cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseState {
    /// Suspended since this moment.
    Paused(Instant),
    /// Woken at this moment to check for new messages.
    Checking(Instant),
}

/// What to do with a hidden paused-mode service right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseAction {
    Suspend,
    Wake,
    Wait,
}

/// Decides the next step. `state` is `None` for a service that has just gone
/// into the background. `check_every` of `None` means "never wake on a timer".
pub fn next_action(state: Option<PauseState>, now: Instant, check_every: Option<Duration>) -> PauseAction {
    match state {
        None => PauseAction::Suspend,
        Some(PauseState::Checking(since)) if now.duration_since(since) >= CHECK_DURATION => {
            PauseAction::Suspend
        }
        Some(PauseState::Checking(_)) => PauseAction::Wait,
        Some(PauseState::Paused(since)) => match check_every {
            Some(interval) if now.duration_since(since) >= interval => PauseAction::Wake,
            _ => PauseAction::Wait,
        },
    }
}

/// The check interval for a setting in minutes (0 = never).
pub fn interval(minutes: u32) -> Option<Duration> {
    (minutes > 0).then(|| Duration::from_secs(u64::from(minutes) * 60))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIVE_MIN: Option<Duration> = Some(Duration::from_secs(300));

    #[test]
    fn a_service_that_just_went_to_the_background_is_suspended() {
        assert_eq!(next_action(None, Instant::now(), FIVE_MIN), PauseAction::Suspend);
    }

    #[test]
    fn a_paused_service_wakes_after_the_interval() {
        let start = Instant::now();
        let paused = Some(PauseState::Paused(start));
        assert_eq!(next_action(paused, start + Duration::from_secs(299), FIVE_MIN), PauseAction::Wait);
        assert_eq!(next_action(paused, start + Duration::from_secs(300), FIVE_MIN), PauseAction::Wake);
    }

    #[test]
    fn a_checking_service_is_paused_again_after_catching_up() {
        let start = Instant::now();
        let checking = Some(PauseState::Checking(start));
        assert_eq!(next_action(checking, start + Duration::from_secs(10), FIVE_MIN), PauseAction::Wait);
        assert_eq!(next_action(checking, start + CHECK_DURATION, FIVE_MIN), PauseAction::Suspend);
    }

    #[test]
    fn never_wakes_when_checking_is_off() {
        let start = Instant::now();
        let paused = Some(PauseState::Paused(start));
        assert_eq!(next_action(paused, start + Duration::from_secs(86_400), interval(0)), PauseAction::Wait);
    }
}
