//! Round header divider (§3.3).
//!
//! Thin horizontal line with "Round N" label centered.
//! type-caption, text-tertiary.

use leptos::prelude::*;

/// Round divider shown before the first message of each round.
#[component]
pub fn RoundDivider(
    /// Round number.
    round: u32,
) -> impl IntoView {
    view! {
        <div
            class="round-divider flex items-center gap-3 select-none py-3"
        >
            <div style="flex: 1; height: 1px; background: var(--border-subtle);" />
            <span class="type-caption text-tertiary">
                {format!("Round {round}")}
            </span>
            <div style="flex: 1; height: 1px; background: var(--border-subtle);" />
        </div>
    }
}
