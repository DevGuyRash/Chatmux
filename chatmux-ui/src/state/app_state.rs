//! Global app-level state.

use leptos::prelude::*;
use uuid::Uuid;

/// Global application state provided at the app root.
#[derive(Clone, Copy)]
pub struct AppState {
    /// The ID of the currently active workspace, if any.
    pub active_workspace_id: ReadSignal<Option<Uuid>>,
    pub set_active_workspace_id: WriteSignal<Option<Uuid>>,
}

/// Create and provide the global app state.
pub fn provide_app_state() {
    let (active_workspace_id, set_active_workspace_id) = signal(None::<Uuid>);

    provide_context(AppState {
        active_workspace_id,
        set_active_workspace_id,
    });
}
