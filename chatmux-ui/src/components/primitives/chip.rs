//! Selectable chip component.
//!
//! Used for provider target chips in the composer, filter chips in
//! the search bar, and participant selection.

use leptos::prelude::*;

/// Chip component — small selectable pill.
#[component]
pub fn Chip(
    /// Label text.
    label: String,
    /// Whether the chip is selected.
    #[prop(into)]
    selected: Signal<bool>,
    /// On click callback.
    on_click: impl Fn() + 'static,
    /// Whether the chip is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Optional custom background color when selected.
    #[prop(optional, into)]
    selected_bg: Option<String>,
    /// Optional custom border color when selected.
    #[prop(optional, into)]
    selected_border: Option<String>,
    /// Optional prefix content (e.g., provider icon).
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let sel_bg = selected_bg.unwrap_or_else(|| "var(--surface-selected)".to_string());
    let sel_border = selected_border.unwrap_or_else(|| "var(--border-accent)".to_string());

    view! {
        <button
            class="chip type-caption-strong select-none"
            style=move || format!(
                "display: inline-flex; align-items: center; gap: var(--space-2); \
                 padding: var(--space-2) var(--space-4); \
                 border-radius: var(--radius-xl); \
                 border: 1px solid {}; \
                 background: {}; \
                 color: {}; \
                 cursor: {}; \
                 opacity: {}; \
                 transition: all var(--duration-fast) var(--easing-standard);",
                if selected.get() { &sel_border } else { "var(--border-default)" },
                if selected.get() { &sel_bg } else { "var(--surface-raised)" },
                if disabled { "var(--text-tertiary)" } else { "var(--text-primary)" },
                if disabled { "not-allowed" } else { "pointer" },
                if disabled { "0.5" } else { "1" },
            )
            disabled=disabled
            on:click=move |_| {
                if !disabled {
                    on_click();
                }
            }
        >
            {children.map(|c| c())}
            {label}
        </button>
    }
}
