//! Run boundary divider (§5.2).
//!
//! Prominent divider with "Run N — Mode" label and timestamp.

use leptos::prelude::*;

/// Run divider shown between messages of different runs.
#[component]
pub fn RunDivider(
    /// Run number.
    run_number: u32,
    /// Orchestration mode label.
    mode_label: String,
    /// Timestamp of run start.
    timestamp: String,
) -> impl IntoView {
    view! {
        <div
            class="run-divider flex flex-col items-center gap-1 select-none"
            style="padding: var(--space-6) 0;"
        >
            <div class="flex items-center gap-3 w-full">
                <div style="flex: 1; height: 2px; background: var(--border-default);" />
                <span class="type-subtitle text-secondary">
                    {format!("Run {run_number} — {mode_label}")}
                </span>
                <div style="flex: 1; height: 2px; background: var(--border-default);" />
            </div>
            <span class="type-caption text-tertiary">{timestamp}</span>
        </div>
    }
}
