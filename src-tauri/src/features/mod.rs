//! Product features expressed as pure, testable logic.
//!
//! Nothing in this module touches Tauri; the `runtime` module applies these
//! decisions to real windows, webviews and notifications.

pub mod away;
pub mod focus;
pub mod history;
pub mod notify;
pub mod schedule;
