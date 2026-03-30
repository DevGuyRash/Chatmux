//! Diagnostic events state.

use leptos::prelude::*;

use crate::models::{DiagnosticEvent, WorkspaceDiagnosticsSummary};

/// Diagnostics state.
#[derive(Clone, Copy)]
pub struct DiagnosticsState {
    pub events: ReadSignal<Vec<DiagnosticEvent>>,
    pub set_events: WriteSignal<Vec<DiagnosticEvent>>,
    pub summary: ReadSignal<WorkspaceDiagnosticsSummary>,
    pub set_summary: WriteSignal<WorkspaceDiagnosticsSummary>,
    /// Count of unread critical+warning events (for nav badge).
    pub unread_count: ReadSignal<u32>,
    pub set_unread_count: WriteSignal<u32>,
    // Display-level filters (shared so badge + panel stay in sync).
    pub filter_critical: ReadSignal<bool>,
    pub set_filter_critical: WriteSignal<bool>,
    pub filter_warning: ReadSignal<bool>,
    pub set_filter_warning: WriteSignal<bool>,
    pub filter_info: ReadSignal<bool>,
    pub set_filter_info: WriteSignal<bool>,
    pub filter_debug: ReadSignal<bool>,
    pub set_filter_debug: WriteSignal<bool>,
}

pub fn provide_diagnostics_state() -> DiagnosticsState {
    let (events, set_events) = signal(Vec::<DiagnosticEvent>::new());
    let (summary, set_summary) = signal(WorkspaceDiagnosticsSummary::default());
    let (unread_count, set_unread_count) = signal(0u32);
    let (filter_critical, set_filter_critical) = signal(true);
    let (filter_warning, set_filter_warning) = signal(true);
    let (filter_info, set_filter_info) = signal(true);
    let (filter_debug, set_filter_debug) = signal(false);

    let state = DiagnosticsState {
        events,
        set_events,
        summary,
        set_summary,
        unread_count,
        set_unread_count,
        filter_critical,
        set_filter_critical,
        filter_warning,
        set_filter_warning,
        filter_info,
        set_filter_info,
        filter_debug,
        set_filter_debug,
    };

    provide_context(state);
    state
}
