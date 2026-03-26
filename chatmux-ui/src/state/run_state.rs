//! Run lifecycle state.

use leptos::prelude::*;

use crate::models::{Run, RunStatus};

/// Active run state for the current workspace.
#[derive(Clone, Copy)]
pub struct ActiveRunState {
    pub run: ReadSignal<Option<Run>>,
    pub set_run: WriteSignal<Option<Run>>,
}

impl ActiveRunState {
    pub fn state(&self) -> RunStatus {
        self.run
            .get_untracked()
            .map(|r| r.status)
            .unwrap_or(RunStatus::Created)
    }
}

pub fn provide_run_state() {
    let (run, set_run) = signal(None::<Run>);
    provide_context(ActiveRunState { run, set_run });
}
