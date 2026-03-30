//! Segmented control component.
//!
//! Used for theme picker (Dark/Light/System), format picker (Markdown/JSON/TOML),
//! surface preference (Sidebar/Full Tab), and mode selectors.

use leptos::prelude::*;
use std::sync::Arc;

/// A single segment option.
#[derive(Clone, Debug)]
pub struct Segment {
    pub value: String,
    pub label: String,
}

/// Segmented control (radio group rendered as connected buttons).
#[component]
pub fn SegmentedControl(
    /// Available segments.
    segments: Vec<Segment>,
    /// Currently selected value.
    selected: ReadSignal<String>,
    /// On change callback.
    on_change: impl Fn(String) + 'static + Send + Sync,
    /// Optional aria-label for the group.
    #[prop(optional, into)]
    aria_label: Option<String>,
    /// Optional tooltips for each segment (parallel to segments vec).
    #[prop(optional)]
    tooltips: Vec<String>,
) -> impl IntoView {
    let on_change = Arc::new(on_change);

    view! {
        <div
            class="segmented-control flex items-center select-none"
            role="radiogroup"
            aria-label=aria_label
            style="\
                background: var(--surface-sunken); \
                border-radius: var(--radius-md); \
                padding: var(--space-1); \
                gap: var(--space-1);"
        >
            {segments.into_iter().enumerate().map(|(idx, seg)| {
                let tooltip = tooltips.get(idx).cloned();
                let value_check = seg.value.clone();
                let value_style = seg.value.clone();
                let value_click = seg.value.clone();
                let label = seg.label;
                let on_change = Arc::clone(&on_change);

                view! {
                    <button
                        class="type-caption-strong cursor-pointer"
                        role="radio"
                        title=tooltip
                        aria-checked=move || if selected.get() == value_check { "true" } else { "false" }
                        style=move || format!(
                            "padding: var(--space-2) var(--space-4); \
                             border-radius: var(--radius-sm); \
                             border: none; \
                             background: {}; \
                             color: {}; \
                             transition: all var(--duration-fast) var(--easing-standard);",
                            if selected.get() == value_style { "var(--surface-raised)" } else { "transparent" },
                            if selected.get() == value_style { "var(--text-primary)" } else { "var(--text-secondary)" },
                        )
                        on:click=move |_| on_change(value_click.clone())
                    >
                        {label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}
