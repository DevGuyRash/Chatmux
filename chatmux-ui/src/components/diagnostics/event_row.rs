//! Diagnostic event row (§3.14).
//!
//! Expandable row showing severity, timestamp, event code, detail text.
//! Expanded: full detail, key-value metadata, snapshot link.

use leptos::prelude::*;

use crate::models::{DiagnosticEvent, DiagnosticLevel};

/// Diagnostic event row.
#[component]
pub fn EventRow(
    /// The diagnostic event.
    event: DiagnosticEvent,
) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);
    let severity_color = match event.level {
        DiagnosticLevel::Critical => "status-error",
        DiagnosticLevel::Warning => "status-warning",
        DiagnosticLevel::Info => "status-info",
        DiagnosticLevel::Debug => "status-neutral",
    };
    let timestamp = event.timestamp.format("%H:%M:%S").to_string();

    view! {
        <div
            class="event-row cursor-pointer transition-colors"
            style="border-bottom: 1px solid var(--border-subtle);"
            on:click=move |_| set_expanded.update(|v| *v = !*v)
        >
            // Main row
            <div class="flex items-center gap-3"
                 style="padding: var(--space-3) var(--space-5);">
                // Severity icon
                <span style=format!("color: var(--{}-text);", severity_color)>
                    {match event.level {
                        DiagnosticLevel::Critical => "✖",
                        DiagnosticLevel::Warning => "⚠",
                        DiagnosticLevel::Info => "ℹ",
                        DiagnosticLevel::Debug => "●",
                    }}
                </span>

                // Timestamp
                <span class="type-caption text-secondary">{timestamp}</span>

                // Event code
                <span class="type-code-small" style="font-family: var(--font-mono);">
                    {event.code.clone()}
                </span>

                // Detail (truncated)
                <span class="type-body text-primary flex-1 truncate">
                    {event.detail.clone()}
                </span>

                // Provider icon placeholder — binding_id shown in expanded view
            </div>

            // Expanded detail
            {
                let detail = event.detail.clone();
                let code = event.code.clone();
                let binding_id = event.binding_id;
                let snapshot_ref = event.snapshot_ref.clone();
                move || expanded.get().then(|| view! {
                    <div class="surface-sunken p-4 rounded-sm" style="margin: 0 var(--space-5) var(--space-3);">
                        <p class="type-body text-primary" style="margin-bottom: var(--space-3); white-space: pre-wrap;">
                            {detail.clone()}
                        </p>
                        <div class="flex flex-col gap-1">
                            <span class="type-caption text-secondary">
                                {format!("Event Code: {}", code)}
                            </span>
                            {binding_id.map(|id| view! {
                                <span class="type-caption text-secondary">
                                    {format!("Binding ID: {:?}", id)}
                                </span>
                            })}
                            {snapshot_ref.clone().map(|id| view! {
                                <span class="type-caption text-link cursor-pointer">
                                    {format!("View Snapshot ({})", id)}
                                </span>
                            })}
                        </div>
                    </div>
                })
            }
        </div>
    }
}
