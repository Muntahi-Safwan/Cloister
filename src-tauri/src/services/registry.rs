//! The ordered list of services the user has added, persisted to `services.json`.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::catalog;
use super::model::{validate_url, IconKind, NewService, ServiceConfig, ServiceId};
use crate::error::{AppError, AppResult};
use crate::persistence::{load_json, save_json};

/// On-disk format of `services.json`. Versioned so the format can evolve.
#[derive(Debug, Default, Serialize, Deserialize)]
struct ServicesFile {
    #[serde(default = "current_version")]
    version: u32,
    #[serde(default)]
    services: Vec<ServiceConfig>,
}

fn current_version() -> u32 {
    1
}

/// In-memory service list with persistence.
#[derive(Debug)]
pub struct ServiceRegistry {
    path: PathBuf,
    services: Vec<ServiceConfig>,
}

impl ServiceRegistry {
    /// Loads the registry from `path`, starting empty if the file does not exist.
    pub fn load(path: PathBuf) -> AppResult<Self> {
        let file: ServicesFile = load_json(&path)?;
        Ok(Self { path, services: file.services })
    }

    /// All services in sidebar order.
    pub fn all(&self) -> &[ServiceConfig] {
        &self.services
    }

    /// Looks up a service by id.
    pub fn get(&self, id: &str) -> AppResult<&ServiceConfig> {
        self.services.iter().find(|s| s.id == id).ok_or_else(|| AppError::ServiceNotFound(id.to_string()))
    }

    /// Validates and appends a new service, then saves. Returns the stored config.
    pub fn add(&mut self, input: NewService) -> AppResult<ServiceConfig> {
        let url = validate_url(&input.url)?;
        if let Some(recipe_id) = &input.recipe_id {
            catalog::find(recipe_id).ok_or_else(|| AppError::RecipeNotFound(recipe_id.clone()))?;
        }
        let name = input.name.trim();
        let name = if name.is_empty() { url.host_str().unwrap_or("Service") } else { name };

        let config = ServiceConfig {
            id: generate_id(name),
            name: name.to_string(),
            url: url.to_string(),
            recipe_id: input.recipe_id,
            icon: input.icon,
            color: input.color,
            icon_url: None,
            muted_until: None,
            zoom: 1.0,
            options: input.options,
        };
        self.services.push(config.clone());
        self.save()?;
        Ok(config)
    }

    /// Removes a service and saves. Returns the removed config.
    pub fn remove(&mut self, id: &str) -> AppResult<ServiceConfig> {
        let index = self
            .services
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| AppError::ServiceNotFound(id.to_string()))?;
        let removed = self.services.remove(index);
        self.save()?;
        Ok(removed)
    }

    /// Changes how a service's icon is drawn and saves.
    pub fn set_icon(&mut self, id: &str, icon: IconKind) -> AppResult<()> {
        self.modify(id, |s| s.icon = icon).map(drop)
    }

    /// Applies `change` to one service, saves, and returns the updated config.
    pub fn modify(&mut self, id: &str, change: impl FnOnce(&mut ServiceConfig)) -> AppResult<ServiceConfig> {
        let service = self.get_mut(id)?;
        change(service);
        let updated = service.clone();
        self.save()?;
        Ok(updated)
    }

    /// Stores the site icon a page reported. Returns `true` if it changed.
    pub fn set_icon_url(&mut self, id: &str, icon_url: String) -> AppResult<bool> {
        let service = self.get_mut(id)?;
        if service.icon_url.as_deref() == Some(icon_url.as_str()) {
            return Ok(false);
        }
        service.icon_url = Some(icon_url);
        self.save()?;
        Ok(true)
    }

    fn get_mut(&mut self, id: &str) -> AppResult<&mut ServiceConfig> {
        self.services.iter_mut().find(|s| s.id == id).ok_or_else(|| AppError::ServiceNotFound(id.to_string()))
    }

    /// Reorders services to match `ids`. Unknown ids are ignored and services
    /// missing from `ids` keep their relative order at the end.
    pub fn reorder(&mut self, ids: &[ServiceId]) -> AppResult<()> {
        let mut ordered = Vec::with_capacity(self.services.len());
        for id in ids {
            if let Some(pos) = self.services.iter().position(|s| &s.id == id) {
                ordered.push(self.services.remove(pos));
            }
        }
        ordered.append(&mut self.services);
        self.services = ordered;
        self.save()
    }

    fn save(&self) -> AppResult<()> {
        save_json(&self.path, &ServicesFile { version: current_version(), services: self.services.clone() })
    }
}

/// Builds an id like `whatsapp-lq3k9` from a display name and the current time.
fn generate_id(name: &str) -> ServiceId {
    let slug: String = name
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let slug = if slug.is_empty() { "service".to_string() } else { slug.chars().take(24).collect() };
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or_default();
    format!("{slug}-{}", to_base36(nanos as u64))
}

fn to_base36(mut value: u64) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::new();
    loop {
        out.push(DIGITS[(value % 36) as usize]);
        value /= 36;
        if value == 0 {
            break;
        }
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::model::ServiceOptions;

    fn temp_registry(name: &str) -> ServiceRegistry {
        let path = std::env::temp_dir().join(format!("cloister-registry-{name}.json"));
        let _ = std::fs::remove_file(&path);
        ServiceRegistry::load(path).unwrap()
    }

    fn input(name: &str, url: &str) -> NewService {
        NewService {
            name: name.into(),
            url: url.into(),
            recipe_id: None,
            icon: IconKind::Initials,
            color: 0,
            options: ServiceOptions::default(),
        }
    }

    #[test]
    fn ids_are_slugged_and_safe_for_labels() {
        let id = generate_id("Company Chat!");
        assert!(id.starts_with("company-chat-"));
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    }

    #[test]
    fn add_remove_and_persist() {
        let mut reg = temp_registry("crud");
        let added = reg.add(input("Team", "https://chat.example.com")).unwrap();
        assert_eq!(reg.all().len(), 1);

        let reloaded = ServiceRegistry::load(reg.path.clone()).unwrap();
        assert_eq!(reloaded.all()[0].id, added.id);

        reg.remove(&added.id).unwrap();
        assert!(reg.all().is_empty());
    }

    #[test]
    fn rejects_bad_urls_and_unknown_recipes() {
        let mut reg = temp_registry("invalid");
        assert!(reg.add(input("X", "http://insecure.com")).is_err());
        let mut bad = input("X", "https://x.com");
        bad.recipe_id = Some("nope".into());
        assert!(reg.add(bad).is_err());
    }

    #[test]
    fn icon_updates_persist_and_report_changes() {
        let mut reg = temp_registry("icons");
        let added = reg.add(input("Team", "https://chat.example.com")).unwrap();
        assert!(reg.set_icon_url(&added.id, "https://chat.example.com/i.png".into()).unwrap());
        assert!(!reg.set_icon_url(&added.id, "https://chat.example.com/i.png".into()).unwrap());
        reg.set_icon(&added.id, IconKind::Favicon).unwrap();

        let reloaded = ServiceRegistry::load(reg.path.clone()).unwrap();
        assert_eq!(reloaded.all()[0].icon, IconKind::Favicon);
        assert_eq!(reloaded.all()[0].icon_url.as_deref(), Some("https://chat.example.com/i.png"));
    }

    #[test]
    fn modify_saves_changes() {
        let mut reg = temp_registry("modify");
        let added = reg.add(input("Team", "https://chat.example.com")).unwrap();
        let updated = reg.modify(&added.id, |s| s.name = "Work".into()).unwrap();
        assert_eq!(updated.name, "Work");
        assert_eq!(ServiceRegistry::load(reg.path.clone()).unwrap().all()[0].name, "Work");
        assert!(reg.modify("missing", |_| {}).is_err());
    }

    #[test]
    fn empty_name_falls_back_to_host() {
        let mut reg = temp_registry("noname");
        let added = reg.add(input("   ", "https://chat.example.com")).unwrap();
        assert_eq!(added.name, "chat.example.com");
    }
}
