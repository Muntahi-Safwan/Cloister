//! User preferences, persisted to `settings.json`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::features::schedule::QuietHours;
use crate::persistence::{load_json, save_json};

/// Colour theme. `System` follows the Windows light or dark setting.
/// Ids must match `ThemeChoice` in `src/lib/theme.ts` and `src/styles/themes.css`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    System,
    #[default]
    QuietRoom,
    Midnight,
    Forest,
    Mono,
    Paper,
    Daylight,
}

/// App-wide preferences. Every field has a sensible default so older or
/// partially written files keep working.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    /// Start every session with Veil already on.
    pub veil_on_launch: bool,
    /// While Veil is on, toasts name the service but never the sender or text.
    pub redact_toasts_in_veil: bool,
    /// Names whose notifications break through a focus session.
    pub vip_names: Vec<String>,
    /// Default focus session length in minutes.
    pub focus_minutes: u32,
    /// Minutes of inactivity before an "unload when idle" service is closed.
    pub unload_after_minutes: u32,
    /// Closing the window hides Cloister to the tray instead of quitting.
    pub close_to_tray: bool,
    /// Ask the browser engine to use less memory for pages that are not on screen.
    pub low_memory_background: bool,
    /// Colour theme of Cloister's own interface.
    pub theme: Theme,
    /// Daily window when notifications are silenced.
    pub quiet_hours: QuietHours,
    /// Start Cloister minimised to the tray when Windows starts.
    pub launch_at_login: bool,
    /// System-wide shortcuts to show/hide Cloister and to hide everything instantly.
    pub global_shortcuts: bool,
    /// Require Windows Hello to open Cloister.
    pub app_lock: bool,
    /// Lock after the window has been out of focus this many minutes (0 = only on launch).
    pub lock_after_minutes: u32,
    /// Turn on Veil as soon as the window loses focus.
    pub veil_on_blur: bool,
    /// Turn on Veil after the window has been out of focus this many minutes (0 = off).
    pub veil_after_minutes: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            veil_on_launch: false,
            redact_toasts_in_veil: true,
            vip_names: Vec::new(),
            focus_minutes: 25,
            unload_after_minutes: 15,
            close_to_tray: true,
            low_memory_background: true,
            theme: Theme::default(),
            quiet_hours: QuietHours::default(),
            launch_at_login: false,
            global_shortcuts: true,
            app_lock: false,
            lock_after_minutes: 5,
            veil_on_blur: false,
            veil_after_minutes: 0,
        }
    }
}

impl AppSettings {
    /// Clamps values into supported ranges and tidies the VIP list.
    pub fn normalized(mut self) -> Self {
        self.focus_minutes = self.focus_minutes.clamp(5, 180);
        self.unload_after_minutes = self.unload_after_minutes.clamp(1, 240);
        self.quiet_hours = self.quiet_hours.normalized();
        self.lock_after_minutes = self.lock_after_minutes.min(240);
        self.veil_after_minutes = self.veil_after_minutes.min(240);
        self.vip_names =
            self.vip_names.into_iter().map(|n| n.trim().to_string()).filter(|n| !n.is_empty()).collect();
        self
    }
}

/// Settings plus the file they live in.
#[derive(Debug)]
pub struct SettingsStore {
    path: PathBuf,
    current: AppSettings,
}

impl SettingsStore {
    /// Loads settings from `path`, falling back to defaults.
    pub fn load(path: PathBuf) -> AppResult<Self> {
        let current = load_json::<AppSettings>(&path)?.normalized();
        Ok(Self { path, current })
    }

    /// The current settings.
    pub fn get(&self) -> &AppSettings {
        &self.current
    }

    /// Replaces and saves the settings. Returns the normalised value.
    pub fn replace(&mut self, next: AppSettings) -> AppResult<AppSettings> {
        self.current = next.normalized();
        save_json(&self.path, &self.current)?;
        Ok(self.current.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_ids_match_the_frontend() {
        let json = serde_json::to_string(&[Theme::System, Theme::QuietRoom, Theme::Daylight]).unwrap();
        assert_eq!(json, r#"["system","quiet-room","daylight"]"#);
        let old: AppSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(old.theme, Theme::QuietRoom);
    }

    #[test]
    fn normalization_clamps_and_trims() {
        let settings = AppSettings {
            focus_minutes: 1,
            unload_after_minutes: 10_000,
            vip_names: vec!["  Mom ".into(), "".into()],
            ..AppSettings::default()
        }
        .normalized();
        assert_eq!(settings.focus_minutes, 5);
        assert_eq!(settings.unload_after_minutes, 240);
        assert_eq!(settings.vip_names, vec!["Mom".to_string()]);
    }
}
