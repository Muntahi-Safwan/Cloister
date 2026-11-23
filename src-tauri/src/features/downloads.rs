//! Files saved from service pages: where they go, and the in-memory list of recent ones.
//!
//! The list is never written to disk and is cleared when Cloister quits. The
//! files themselves stay in the Downloads folder.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::services::ServiceId;

/// How many downloads the list keeps.
pub const DOWNLOADS_LIMIT: usize = 30;

/// Longest file name kept, in characters (before the extension is re-added).
const MAX_NAME_CHARS: usize = 120;

/// File types that run code when opened. Cloister only shows these in their folder,
/// so one click on a download can never start a program.
const RISKY_EXTENSIONS: &[&str] = &[
    "app", "appimage", "apk", "bat", "cmd", "com", "command", "cpl", "deb", "dmg", "exe", "hta", "jar", "js",
    "jse", "lnk", "msc", "msi", "msix", "pkg", "ps1", "py", "reg", "rpm", "run", "scr", "sh", "vb", "vbe",
    "vbs", "ws", "wsf",
];

/// Where a download stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DownloadState {
    Saving,
    Done,
    Failed,
}

/// One file saved from a service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    pub id: u64,
    pub service_id: ServiceId,
    pub file_name: String,
    #[serde(skip)]
    pub path: PathBuf,
    /// The address it came from, used to match the finish event to its start.
    #[serde(skip)]
    pub url: String,
    pub state: DownloadState,
    pub started_ms: u64,
    /// Opening it would run code, so the UI only offers "Show in folder".
    pub risky: bool,
}

/// A bounded list of recent downloads, newest first.
#[derive(Debug, Default)]
pub struct Downloads {
    items: VecDeque<DownloadItem>,
    next_id: u64,
}

impl Downloads {
    /// Records a download that has just started and returns its id.
    pub fn start(&mut self, service_id: &str, url: &str, path: &Path, started_ms: u64) -> u64 {
        self.next_id += 1;
        let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        self.items.push_front(DownloadItem {
            id: self.next_id,
            service_id: service_id.to_owned(),
            risky: is_risky(&file_name),
            file_name,
            path: path.to_path_buf(),
            url: url.to_owned(),
            state: DownloadState::Saving,
            started_ms,
        });
        self.items.truncate(DOWNLOADS_LIMIT);
        self.next_id
    }

    /// Marks the newest unfinished download of `url` from `service_id` as finished.
    /// `path` is where the file really ended up, when the system reports it
    /// (macOS never does). Returns whether a download was updated.
    pub fn finish(&mut self, service_id: &str, url: &str, path: Option<&Path>, success: bool) -> bool {
        let Some(item) = self
            .items
            .iter_mut()
            .find(|i| i.state == DownloadState::Saving && i.service_id == service_id && i.url == url)
        else {
            return false;
        };
        item.state = if success { DownloadState::Done } else { DownloadState::Failed };
        if let Some(path) = path.filter(|p| !p.as_os_str().is_empty()) {
            item.path = path.to_path_buf();
            if let Some(name) = path.file_name() {
                item.file_name = name.to_string_lossy().into_owned();
                item.risky = is_risky(&item.file_name);
            }
        }
        true
    }

    /// All items, newest first.
    pub fn items(&self) -> Vec<DownloadItem> {
        self.items.iter().cloned().collect()
    }

    pub fn get(&self, id: u64) -> Option<&DownloadItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn clear(&mut self) {
        self.items.retain(|i| i.state == DownloadState::Saving);
    }

    /// Forgets everything from one service (used when it is removed or erased).
    pub fn forget_service(&mut self, service_id: &str) {
        self.items.retain(|i| i.service_id != service_id);
    }
}

/// Decides where a download is saved.
///
/// `suggested` is the path the browser engine proposed. Its folder is kept when
/// it is a real absolute folder (normally the user's Downloads folder),
/// otherwise `downloads_dir` is used. The file name is cleaned so a page cannot
/// write outside that folder or pick a reserved name, and a number is added
/// when a file of that name already exists.
pub fn destination(suggested: &Path, downloads_dir: &Path, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let dir =
        suggested.parent().filter(|p| p.is_absolute() && !p.as_os_str().is_empty()).unwrap_or(downloads_dir);
    let name = clean_file_name(&suggested.file_name().map(|n| n.to_string_lossy()).unwrap_or_default());

    let (stem, ext) = split_name(&name);
    let mut candidate = dir.join(&name);
    let mut counter = 1;
    while exists(&candidate) {
        candidate = dir.join(format!("{stem} ({counter}){ext}"));
        counter += 1;
    }
    candidate
}

/// Makes a file name safe on every platform. Never returns an empty name.
pub fn clean_file_name(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    // Windows drops trailing dots and spaces; a leading dot would hide the file elsewhere.
    let cleaned = cleaned.trim().trim_end_matches(['.', ' ']).trim_start_matches('.');

    let (stem, ext) = split_name(cleaned);
    let stem: String = stem.chars().take(MAX_NAME_CHARS).collect();
    let ext: String = if ext.chars().count() > 16 { String::new() } else { ext.to_owned() };
    let stem = stem.trim();
    if stem.is_empty() {
        format!("download{ext}")
    } else if is_reserved_on_windows(stem) {
        format!("download_{stem}{ext}")
    } else {
        format!("{stem}{ext}")
    }
}

/// Whether opening a file of this name would run code.
pub fn is_risky(file_name: &str) -> bool {
    let (_, ext) = split_name(file_name);
    let ext = ext.trim_start_matches('.').to_ascii_lowercase();
    RISKY_EXTENSIONS.contains(&ext.as_str())
}

/// Splits `report.final.pdf` into `("report.final", ".pdf")`.
fn split_name(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i..]),
        _ => (name, ""),
    }
}

fn is_reserved_on_windows(stem: &str) -> bool {
    let upper = stem.trim().to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (upper.len() == 4
            && (upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.as_bytes()[3].is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn downloads() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"C:\Users\me\Downloads")
        } else {
            PathBuf::from("/home/me/Downloads")
        }
    }

    #[test]
    fn cleans_names() {
        assert_eq!(clean_file_name("invoice.pdf"), "invoice.pdf");
        assert_eq!(clean_file_name("../../etc/passwd"), "_.._etc_passwd");
        assert_eq!(clean_file_name("a:b*c?.txt"), "a_b_c_.txt");
        assert_eq!(clean_file_name("notes. . "), "notes");
        assert_eq!(clean_file_name(".bashrc"), "bashrc");
        assert_eq!(clean_file_name(""), "download");
        assert_eq!(clean_file_name("   "), "download");
        assert_eq!(clean_file_name("CON.txt"), "download_CON.txt");
        assert_eq!(clean_file_name("com1"), "download_com1");
        assert_eq!(clean_file_name("comet.txt"), "comet.txt");
        let long = format!("{}.pdf", "x".repeat(300));
        assert_eq!(clean_file_name(&long).chars().count(), MAX_NAME_CHARS + 4);
    }

    #[test]
    fn keeps_the_suggested_folder_and_numbers_duplicates() {
        let dir = downloads();
        let taken = [dir.join("photo.jpg"), dir.join("photo (1).jpg")];
        let dest = destination(&dir.join("photo.jpg"), &dir, |p| taken.iter().any(|t| t == p));
        assert_eq!(dest, dir.join("photo (2).jpg"));
    }

    #[test]
    fn falls_back_to_downloads_for_bare_names() {
        let dest = destination(Path::new("report.pdf"), &downloads(), |_| false);
        assert_eq!(dest, downloads().join("report.pdf"));
    }

    #[test]
    fn flags_files_that_run_code() {
        assert!(is_risky("setup.EXE"));
        assert!(is_risky("install.sh"));
        assert!(!is_risky("invoice.pdf"));
        assert!(!is_risky("exe"));
    }

    #[test]
    fn matches_finish_to_start() {
        let mut list = Downloads::default();
        let path = downloads().join("a.pdf");
        let id = list.start("wa", "https://x/a", &path, 1);
        assert!(!list.finish("other", "https://x/a", None, true));
        let real = downloads().join("a (1).pdf");
        assert!(list.finish("wa", "https://x/a", Some(&real), true));
        let item = list.get(id).unwrap();
        assert_eq!(item.state, DownloadState::Done);
        assert_eq!(item.file_name, "a (1).pdf");
        // A second finish for the same address finds nothing left to update.
        assert!(!list.finish("wa", "https://x/a", None, false));
    }

    #[test]
    fn keeps_unfinished_downloads_when_cleared_and_caps_length() {
        let mut list = Downloads::default();
        for i in 0..(DOWNLOADS_LIMIT + 3) {
            list.start("wa", &format!("u{i}"), &downloads().join(format!("{i}.txt")), i as u64);
        }
        assert_eq!(list.items().len(), DOWNLOADS_LIMIT);
        list.finish("wa", &format!("u{}", DOWNLOADS_LIMIT + 2), None, true);
        list.clear();
        assert_eq!(list.items().len(), DOWNLOADS_LIMIT - 1);
        list.forget_service("wa");
        assert!(list.items().is_empty());
    }
}
