//! Measures how much memory each service's browser processes use.
//!
//! Every service runs in its own browser profile, and each of its processes
//! carries that profile's folder on its command line. Summing the memory of
//! processes that mention a service's folder gives its footprint.

use std::collections::HashMap;
use std::path::Path;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::{AppHandle, Manager, Runtime};

use crate::persistence;
use crate::services::ServiceId;
use crate::state::AppState;

/// Resident memory in bytes per loaded service. Unloaded services are absent.
pub fn usage_by_service<R: Runtime>(app: &AppHandle<R>) -> HashMap<ServiceId, u64> {
    let Ok(profiles) = persistence::profiles_dir(app) else { return HashMap::new() };
    let loaded: Vec<ServiceId> = app.state::<AppState>().session().loaded.iter().cloned().collect();
    if loaded.is_empty() {
        return HashMap::new();
    }

    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory().with_cmd(UpdateKind::OnlyIfNotSet),
    );
    let processes: Vec<(String, u64)> = system
        .processes()
        .values()
        .map(|p| (p.cmd().iter().map(|a| a.to_string_lossy()).collect::<Vec<_>>().join(" "), p.memory()))
        .collect();

    loaded
        .into_iter()
        .map(|id| {
            let marker = profile_marker(&profiles.join(&id));
            let bytes = processes
                .iter()
                .filter(|(cmd, _)| cmd.to_lowercase().contains(&marker))
                .map(|(_, memory)| memory)
                .sum();
            (id, bytes)
        })
        .collect()
}

/// The lowercase folder path as it appears on a command line, ending in a
/// separator so `whatsapp` does not also match `whatsapp-2`.
fn profile_marker(dir: &Path) -> String {
    format!("{}{}", dir.to_string_lossy(), std::path::MAIN_SEPARATOR).to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_are_case_insensitive_paths() {
        let sep = std::path::MAIN_SEPARATOR;
        let marker = profile_marker(Path::new("/Users/Me/Profiles/WhatsApp"));
        assert_eq!(marker, format!("/users/me/profiles/whatsapp{sep}"));
        assert!(format!("--user-data-dir=/users/me/profiles/whatsapp{sep}ebwebview").contains(&marker));
        assert!(!format!("--user-data-dir=/users/me/profiles/whatsapp-2{sep}ebwebview").contains(&marker));
    }
}
