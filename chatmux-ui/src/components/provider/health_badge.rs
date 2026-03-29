//! Health state badge component (§3.8).
//!
//! Pill-shaped badge with icon + label, colored by health state.
//! Used on provider binding cards.

use leptos::prelude::*;

use super::HealthState;

/// Health state badge — pill with icon + label.
#[component]
pub fn HealthBadge(
    /// Health state to display.
    health: Signal<HealthState>,
) -> impl IntoView {
    view! {
        <span
            class="health-badge type-caption-strong select-none"
            style=move || {
                let h = health.get();
                format!(
                    "display: inline-flex; align-items: center; gap: var(--space-1); \
                     padding: var(--space-1) var(--space-3); \
                     border-radius: var(--radius-xl); \
                     background: var(--{}-muted); \
                     color: var(--{}-text); \
                     transition: all var(--duration-fast) var(--easing-spring);",
                    h.status_color(),
                    h.status_color(),
                )
            }
        >
            <span style="font-size: 10px;">{move || health.get().icon()}</span>
            {move || health.get().label()}
        </span>
    }
}
