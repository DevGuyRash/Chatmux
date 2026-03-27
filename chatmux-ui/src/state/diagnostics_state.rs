//! Diagnostic events state.

use leptos::prelude::*;

use crate::models::DiagnosticEvent;

/// Diagnostics state.
#[derive(Clone, Copy)]
pub struct DiagnosticsState {
    pub events: ReadSignal<Vec<DiagnosticEvent>>,
    pub set_events: WriteSignal<Vec<DiagnosticEvent>>,
    /// Count of unread critical+warning events (for nav badge).
    pub unread_count: ReadSignal<u32>,
    pub set_unread_count: WriteSignal<u32>,
}

pub fn provide_diagnostics_state() -> DiagnosticsState {
    let (events, set_events) = signal(Vec::<DiagnosticEvent>::new());
    let (unread_count, set_unread_count) = signal(0u32);

    let state = DiagnosticsState {
        events,
        set_events,
        unread_count,
        set_unread_count,
    };

    provide_context(state);
    state
}
