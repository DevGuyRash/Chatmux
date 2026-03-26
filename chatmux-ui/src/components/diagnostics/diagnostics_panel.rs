//! Diagnostics panel (§3.14).
//!
//! Filter bar (severity, provider, time range) + scrolling event list.

use leptos::prelude::*;

use crate::components::primitives::chip::Chip;
use crate::components::primitives::empty_state::EmptyState;
use crate::components::primitives::icon::IconKind;
use crate::models::{DiagnosticEvent, DiagnosticLevel};
use super::event_row::EventRow;

/// Diagnostics panel component.
#[component]
pub fn DiagnosticsPanel(
    /// Diagnostic events.
    events: ReadSignal<Vec<DiagnosticEvent>>,
) -> impl IntoView {
    // Filter state
    let (show_critical, set_show_critical) = signal(true);
    let (show_warning, set_show_warning) = signal(true);
    let (show_info, set_show_info) = signal(true);
    let (show_debug, set_show_debug) = signal(false);

    let filtered = move || {
        events.get().into_iter().filter(|e| {
            match e.level {
                DiagnosticLevel::Critical => show_critical.get(),
                DiagnosticLevel::Warning => show_warning.get(),
                DiagnosticLevel::Info => show_info.get(),
                DiagnosticLevel::Debug => show_debug.get(),
            }
        }).collect::<Vec<_>>()
    };

    view! {
        <div class="diagnostics-panel flex flex-col h-full">
            // Filter bar
            <div class="flex items-center gap-2 p-4 flex-wrap"
                 style="border-bottom: 1px solid var(--border-subtle);">
                <Chip label="Critical".into() selected=show_critical
                      on_click=move || set_show_critical.update(|v| *v = !*v)
                      selected_bg="var(--status-error-muted)".to_string()
                      selected_border="var(--status-error-border)".to_string() />
                <Chip label="Warning".into() selected=show_warning
                      on_click=move || set_show_warning.update(|v| *v = !*v)
                      selected_bg="var(--status-warning-muted)".to_string()
                      selected_border="var(--status-warning-border)".to_string() />
                <Chip label="Info".into() selected=show_info
                      on_click=move || set_show_info.update(|v| *v = !*v)
                      selected_bg="var(--status-info-muted)".to_string()
                      selected_border="var(--status-info-border)".to_string() />
                <Chip label="Debug".into() selected=show_debug
                      on_click=move || set_show_debug.update(|v| *v = !*v) />
            </div>

            // Event list
            <div class="flex-1 overflow-y-auto">
                {move || {
                    let items = filtered();
                    if items.is_empty() {
                        view! {
                            <EmptyState
                                icon=IconKind::ShieldCheck
                                heading="All clear"
                                description="No diagnostic events have been recorded."
                            />
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                {items.into_iter().map(|event| {
                                    view! { <EventRow event=event /> }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
