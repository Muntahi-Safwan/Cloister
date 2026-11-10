//! Everything about *what* a service is: its configuration, the built-in
//! catalog, persistence and the pure rules (unread parsing, host matching)
//! that the runtime applies to it.

pub mod catalog;
pub mod hosts;
pub mod model;
pub mod profile;
pub mod registry;
pub mod unread;

pub use model::{BackgroundMode, NewService, ServiceConfig, ServiceId, UserAgentPreset};
pub use registry::ServiceRegistry;

use catalog::{Recipe, UnreadRule};

/// Returns the recipe behind a service, if it was created from the catalog.
pub fn recipe_of(service: &ServiceConfig) -> Option<&'static Recipe> {
    service.recipe_id.as_deref().and_then(catalog::find)
}

/// The unread rule that applies to a service.
///
/// Recipes decide for catalog services; custom services use the title rule
/// when the user enabled it.
pub fn unread_rule_of(service: &ServiceConfig) -> UnreadRule {
    if !service.options.unread_from_title {
        return UnreadRule::None;
    }
    recipe_of(service).map_or(UnreadRule::TitleParentheses, |r| r.unread)
}
