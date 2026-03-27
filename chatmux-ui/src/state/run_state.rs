//! Run lifecycle state.

use leptos::prelude::*;

use crate::models::{Round, Run, RunStatus};

/// Active run state for the current workspace.
#[derive(Clone, Copy)]
pub struct ActiveRunState {
    pub run: ReadSignal<Option<Run>>,
    pub set_run: WriteSignal<Option<Run>>,
    pub rounds: ReadSignal<Vec<Round>>,
    pub set_rounds: WriteSignal<Vec<Round>>,
}

impl ActiveRunState {
    pub fn state(&self) -> RunStatus {
        self.run
            .get_untracked()
            .map(|r| r.status)
            .unwrap_or(RunStatus::Created)
    }
}

pub fn provide_run_state() -> ActiveRunState {
    let (run, set_run) = signal(None::<Run>);
    let (rounds, set_rounds) = signal(Vec::<Round>::new());

    let state = ActiveRunState {
        run,
        set_run,
        rounds,
        set_rounds,
    };
    provide_context(state);
    state
}
