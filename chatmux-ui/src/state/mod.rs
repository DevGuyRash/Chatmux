//! Application state management.
//!
//! Signal-based state organized by domain. Each module provides
//! reactive state that is provided via Leptos context at the app root.

pub mod app_state;
pub mod workspace_state;
pub mod run_state;
pub mod binding_state;
pub mod message_state;
pub mod selection_state;
pub mod search_state;
pub mod toast_state;
pub mod diagnostics_state;
