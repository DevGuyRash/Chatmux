//! Number input component.
//!
//! Used for timing settings, round counts, quorum thresholds, etc.
//! Includes optional suffix label (e.g., "s", "m", "%").

use leptos::prelude::*;

/// Number input with optional suffix.
#[component]
pub fn NumberInput(
    /// Current value.
    value: ReadSignal<f64>,
    /// On change callback.
    on_change: impl Fn(f64) + 'static,
    /// Minimum value.
    #[prop(optional)]
    min: Option<f64>,
    /// Maximum value.
    #[prop(optional)]
    max: Option<f64>,
    /// Step increment.
    #[prop(default = 1.0)]
    step: f64,
    /// Suffix label (e.g., "s", "m", "%").
    #[prop(optional)]
    suffix: Option<&'static str>,
    /// Whether the input is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Optional aria-label.
    #[prop(optional, into)]
    aria_label: Option<String>,
) -> impl IntoView {
    view! {
        <div class="number-input flex items-center gap-2">
            <input
                type="number"
                class="type-body"
                style="\
                    width: 72px; \
                    padding: var(--space-3) var(--space-3); \
                    background: var(--surface-sunken); \
                    border: 1px solid var(--border-default); \
                    border-radius: var(--radius-md); \
                    color: var(--text-primary); \
                    text-align: right;"
                prop:value=move || value.get()
                min=min.map(|v| v.to_string())
                max=max.map(|v| v.to_string())
                step=step.to_string()
                disabled=disabled
                aria-label=aria_label
                on:input=move |ev| {
                    if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                        on_change(val);
                    }
                }
            />
            {suffix.map(|s| view! {
                <span class="type-caption text-secondary">{s}</span>
            })}
        </div>
    }
}
