//! Workspace state — list of workspaces and active workspace details.

use leptos::prelude::*;

use crate::models::{Workspace, WorkspaceSnapshot};

/// Workspace list state.
#[derive(Clone, Copy)]
pub struct WorkspaceListState {
    pub workspaces: ReadSignal<Vec<Workspace>>,
    pub set_workspaces: WriteSignal<Vec<Workspace>>,
    pub snapshot: ReadSignal<Option<WorkspaceSnapshot>>,
    pub set_snapshot: WriteSignal<Option<WorkspaceSnapshot>>,
    pub show_archived: ReadSignal<bool>,
    pub set_show_archived: WriteSignal<bool>,
}

pub fn provide_workspace_state() -> WorkspaceListState {
    let (workspaces, set_workspaces) = signal(Vec::<Workspace>::new());
    let (snapshot, set_snapshot) = signal(None::<WorkspaceSnapshot>);
    let (show_archived, set_show_archived) = signal(false);

    let state = WorkspaceListState {
        workspaces,
        set_workspaces,
        snapshot,
        set_snapshot,
        show_archived,
        set_show_archived,
    };

    provide_context(state);
    state
}
