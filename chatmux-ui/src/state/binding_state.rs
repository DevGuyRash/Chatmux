//! Provider binding state.

use leptos::prelude::*;

use crate::models::ParticipantBinding;

/// Provider binding state for the current workspace.
#[derive(Clone, Copy)]
pub struct BindingState {
    pub bindings: ReadSignal<Vec<ParticipantBinding>>,
    pub set_bindings: WriteSignal<Vec<ParticipantBinding>>,
}

pub fn provide_binding_state() {
    let (bindings, set_bindings) = signal(Vec::<ParticipantBinding>::new());
    provide_context(BindingState {
        bindings,
        set_bindings,
    });
}
