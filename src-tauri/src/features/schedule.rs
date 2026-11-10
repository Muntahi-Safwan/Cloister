//! Time-based rules: quiet hours and mute durations.
//!
//! Functions take the current time as an argument so they can be tested
//! with fixed dates and time zones.

use chrono::{DateTime, TimeZone, Timelike};
use serde::{Deserialize, Serialize};

/// `muted_until` value that means "muted until unmuted".
pub const MUTED_FOREVER: u64 = u64::MAX;

/// Hour of the day "until tomorrow" mutes end at.
const TOMORROW_HOUR: u32 = 8;

/// A daily window during which notifications are silenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QuietHours {
    pub enabled: bool,
    /// Start, in minutes after local midnight.
    pub start_minute: u16,
    /// End, in minutes after local midnight. May be earlier than the start
    /// for windows that cross midnight (23:00 to 08:00).
    pub end_minute: u16,
}

impl Default for QuietHours {
    fn default() -> Self {
        Self { enabled: false, start_minute: 23 * 60, end_minute: 8 * 60 }
    }
}

impl QuietHours {
    /// Whether `minute` (minutes after local midnight) falls inside the window.
    pub fn contains(&self, minute: u16) -> bool {
        if !self.enabled || self.start_minute == self.end_minute {
            return false;
        }
        if self.start_minute < self.end_minute {
            minute >= self.start_minute && minute < self.end_minute
        } else {
            minute >= self.start_minute || minute < self.end_minute
        }
    }

    /// Whether quiet hours are active at `now`.
    pub fn active_at<Tz: TimeZone>(&self, now: &DateTime<Tz>) -> bool {
        self.contains((now.hour() * 60 + now.minute()) as u16)
    }

    /// Keeps minutes inside one day.
    pub fn normalized(self) -> Self {
        const DAY: u16 = 24 * 60;
        Self { start_minute: self.start_minute % DAY, end_minute: self.end_minute % DAY, ..self }
    }
}

/// How long to mute a service for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MuteDuration {
    OneHour,
    /// Until 08:00 tomorrow, local time.
    UntilTomorrow,
    Forever,
    /// Unmute.
    Off,
}

/// The `muted_until` timestamp (Unix ms) for a mute starting at `now`, or
/// `None` to unmute.
pub fn mute_until<Tz: TimeZone>(duration: MuteDuration, now: &DateTime<Tz>) -> Option<u64> {
    let now_ms = u64::try_from(now.timestamp_millis()).unwrap_or_default();
    match duration {
        MuteDuration::Off => None,
        MuteDuration::Forever => Some(MUTED_FOREVER),
        MuteDuration::OneHour => Some(now_ms + 3_600_000),
        MuteDuration::UntilTomorrow => {
            let tomorrow = now.date_naive().succ_opt()?.and_hms_opt(TOMORROW_HOUR, 0, 0)?;
            let local = now.timezone().from_local_datetime(&tomorrow).earliest()?;
            u64::try_from(local.timestamp_millis()).ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    fn dhaka(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(6 * 3600).unwrap().with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    #[test]
    fn overnight_window_wraps_midnight() {
        let quiet = QuietHours { enabled: true, start_minute: 23 * 60, end_minute: 8 * 60 };
        assert!(quiet.active_at(&dhaka(2026, 11, 9, 23, 30)));
        assert!(quiet.active_at(&dhaka(2026, 11, 10, 7, 59)));
        assert!(!quiet.active_at(&dhaka(2026, 11, 10, 8, 0)));
        assert!(!quiet.active_at(&dhaka(2026, 11, 10, 14, 0)));
    }

    #[test]
    fn daytime_window_and_disabled_state() {
        let quiet = QuietHours { enabled: true, start_minute: 9 * 60, end_minute: 17 * 60 };
        assert!(quiet.contains(12 * 60));
        assert!(!quiet.contains(18 * 60));
        assert!(!QuietHours { enabled: false, ..quiet }.contains(12 * 60));
    }

    #[test]
    fn mute_durations() {
        let now = dhaka(2026, 11, 9, 22, 15);
        let now_ms = now.timestamp_millis() as u64;
        assert_eq!(mute_until(MuteDuration::OneHour, &now), Some(now_ms + 3_600_000));
        assert_eq!(mute_until(MuteDuration::Forever, &now), Some(MUTED_FOREVER));
        assert_eq!(mute_until(MuteDuration::Off, &now), None);
        let tomorrow = dhaka(2026, 11, 10, 8, 0).timestamp_millis() as u64;
        assert_eq!(mute_until(MuteDuration::UntilTomorrow, &now), Some(tomorrow));
    }
}
