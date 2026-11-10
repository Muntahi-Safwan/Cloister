//! Tauri commands, grouped by caller.
//!
//! * [`services`], [`service_actions`], [`session`] and [`privacy`] are called by Cloister's own UI.
//! * [`bridge`] is called by third-party service pages and is kept minimal.
//!
//! Every command name must also be listed in `build.rs` so Tauri can generate
//! its permission, and granted in `capabilities/main.json` (UI commands) or
//! `runtime/capabilities.rs` (bridge commands).

pub mod bridge;
pub mod privacy;
pub mod service_actions;
pub mod services;
pub mod session;
