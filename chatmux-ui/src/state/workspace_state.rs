//! Workspace state — list of workspaces and active workspace details.

use leptos::prelude::*;

use crate::models::Workspace;

/// Workspace list state.
#[derive(Clone, Copy)]
pub struct WorkspaceListState {
    pub workspaces: ReadSignal<Vec<Workspace>>,
    pub set_workspaces: WriteSignal<Vec<Workspace>>,
    pub show_archived: ReadSignal<bool>,
    pub set_show_archived: WriteSignal<bool>,
}

pub fn provide_workspace_state() {
    let (workspaces, set_workspaces) = signal(Vec::<Workspace>::new());
    let (show_archived, set_show_archived) = signal(false);

    provide_context(WorkspaceListState {
        workspaces,
        set_workspaces,
        show_archived,
        set_show_archived,
    });
}
