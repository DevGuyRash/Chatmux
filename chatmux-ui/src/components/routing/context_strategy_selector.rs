//! Context strategy selector (§3.11).
//!
//! Dropdown showing the workspace-level default strategy.
//! Conditional inputs for window size.

use leptos::prelude::*;

use crate::components::primitives::dropdown::{Dropdown, DropdownOption};
use crate::components::primitives::number_input::NumberInput;
use crate::models::ContextStrategy;

/// Context strategy selector.
#[component]
pub fn ContextStrategySelector(
    /// Current strategy.
    strategy: ReadSignal<ContextStrategy>,
    /// On change callback.
    on_change: impl Fn(ContextStrategy) + 'static + Copy + Send,
    /// Number of edges with per-edge overrides.
    override_count: ReadSignal<u32>,
) -> impl IntoView {
    let (selected, set_selected) = signal(strategy_to_value(strategy.get_untracked()));
    let (window_size, set_window_size) = signal(5.0);

    view! {
        <div class="context-strategy-selector flex flex-col gap-3">
            <Dropdown
                options=vec![
                    DropdownOption { value: "workspace_default".into(), label: "Workspace Default".into() },
                    DropdownOption { value: "full".into(), label: "Full History".into() },
                    DropdownOption { value: "last_n".into(), label: "Last N Messages".into() },
                    DropdownOption { value: "pinned_summary".into(), label: "Pinned Summary".into() },
                    DropdownOption { value: "none".into(), label: "None".into() },
                ]
                selected=selected
                on_change=move |v| {
                    set_selected.set(v.clone());
                    let strat = value_to_strategy(&v, window_size.get_untracked() as u32);
                    on_change(strat);
                }
                aria_label="Context strategy"
            />

            // Window size (conditional for last_n)
            {move || (selected.get() == "last_n").then(|| view! {
                <div class="flex items-center gap-2">
                    <span class="type-caption text-secondary">"Message count:"</span>
                    <NumberInput
                        value=window_size
                        on_change=move |v| {
                            set_window_size.set(v);
                            on_change(ContextStrategy::LastN { count: v as usize });
                        }
                        min=1.0
                        max=100.0
                    />
                </div>
            })}

            // Override indicator
            {move || {
                let count = override_count.get();
                (count > 0).then(|| view! {
                    <span class="type-caption text-link cursor-pointer">
                        {format!("{count} edges with overrides")}
                    </span>
                })
            }}

            <p class="type-caption text-secondary">
                "Per-edge overrides can be set in the routing editor."
            </p>
        </div>
    }
}

fn strategy_to_value(s: ContextStrategy) -> String {
    match s {
        ContextStrategy::WorkspaceDefault => "workspace_default",
        ContextStrategy::FullHistory => "full",
        ContextStrategy::LastN { .. } => "last_n",
        ContextStrategy::SpecificRange { .. } => "full", // display as full, no specific UI yet
        ContextStrategy::PinnedSummary { .. } => "pinned_summary",
        ContextStrategy::None => "none",
    }
    .to_string()
}

fn value_to_strategy(v: &str, count: u32) -> ContextStrategy {
    match v {
        "full" => ContextStrategy::FullHistory,
        "last_n" => ContextStrategy::LastN {
            count: count as usize,
        },
        "pinned_summary" => ContextStrategy::PinnedSummary {
            summary: String::new(),
        },
        "none" => ContextStrategy::None,
        _ => ContextStrategy::WorkspaceDefault,
    }
}
