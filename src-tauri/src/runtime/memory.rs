//! Measures how much memory each service uses.
//!
//! * Windows: services share one browser engine, which reports the frames each
//!   of its processes renders; renderers are attributed to their service and
//!   the rest (browser, GPU, network, Cloister's window) counts as shared.
//! * Linux: each service has its own browser profile folder, and each of its
//!   processes carries that folder on its command line.
//! * macOS: WebKit keeps no per-service processes, so only the total is known.

use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::{AppHandle, Manager, Runtime};

use super::webviews;
use crate::services::{profile, ServiceId};
use crate::state::AppState;
use crate::{persistence, platform};

/// Memory in bytes per loaded service, plus what no single service owns.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryReport {
    pub services: HashMap<ServiceId, u64>,
    pub shared: u64,
}

/// Measures every loaded service. Blocks briefly; never call on the main thread.
pub fn report<R: Runtime>(app: &AppHandle<R>) -> MemoryReport {
    let loaded: Vec<ServiceId> = app.state::<AppState>().session().loaded.iter().cloned().collect();
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory().with_cmd(UpdateKind::OnlyIfNotSet),
    );
    let own = system.process(Pid::from_u32(std::process::id())).map_or(0, |p| p.memory());

    let memory_of = |pid: u32| system.process(Pid::from_u32(pid)).map_or(0, |p| p.memory());
    if let Some((by_label, shared)) = platform::shared_engine_memory(app, memory_of) {
        let services = by_label
            .into_iter()
            .filter_map(|(label, bytes)| webviews::service_id_of(&label).map(|id| (id.to_string(), bytes)))
            .collect();
        return MemoryReport { services, shared: shared + own };
    }

    MemoryReport { services: by_profile_folder(app, &system, &loaded), shared: own }
}

/// Attributes processes to services by the profile folder on their command line.
fn by_profile_folder<R: Runtime>(
    app: &AppHandle<R>,
    system: &System,
    loaded: &[ServiceId],
) -> HashMap<ServiceId, u64> {
    let Ok(profiles) = persistence::profiles_dir(app) else { return HashMap::new() };
    let processes: Vec<(String, u64)> = system
        .processes()
        .values()
        .map(|p| {
            (
                p.cmd().iter().map(|a| a.to_string_lossy()).collect::<Vec<_>>().join(" ").to_lowercase(),
                p.memory(),
            )
        })
        .collect();
    loaded
        .iter()
        .filter_map(|id| {
            let marker = profile_marker(&profiles.join(profile::folder_name(id)));
            let bytes: u64 =
                processes.iter().filter(|(cmd, _)| cmd.contains(&marker)).map(|(_, memory)| memory).sum();
            (bytes > 0).then(|| (id.clone(), bytes))
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
