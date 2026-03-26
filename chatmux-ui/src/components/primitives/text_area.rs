//! Multi-line text area component.
//!
//! Used for the composer input, template editor, notes, and summary bodies.
//! Min height 2 lines (~52px), max height 8 lines (~208px) before scrolling.

use leptos::prelude::*;

/// Multi-line text area.
#[component]
pub fn TextArea(
    /// Current value.
    value: ReadSignal<String>,
    /// On input callback.
    on_input: impl Fn(String) + 'static,
    /// Placeholder text.
    #[prop(default = "")]
    placeholder: &'static str,
    /// Whether the area is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Minimum rows.
    #[prop(default = 2)]
    min_rows: u32,
    /// Maximum rows before scrolling.
    #[prop(default = 8)]
    max_rows: u32,
    /// Optional accessible label.
    #[prop(optional, into)]
    aria_label: Option<String>,
    /// Whether to use monospace font.
    #[prop(default = false)]
    monospace: bool,
) -> impl IntoView {
    let line_height = 20; // ~13px * 1.5 line-height
    let min_height = min_rows * line_height as u32;
    let max_height = max_rows * line_height as u32;

    let font_family = if monospace {
        "font-family: var(--font-mono); font-size: var(--type-code-size);"
    } else {
        ""
    };

    view! {
        <textarea
            class="text-area type-body"
            placeholder=placeholder
            disabled=disabled
            aria-label=aria_label
            style=format!(
                "width: 100%; \
                 min-height: {min_height}px; max-height: {max_height}px; \
                 padding: var(--space-4); \
                 background: var(--surface-sunken); \
                 border: 1px solid var(--border-default); \
                 border-radius: var(--radius-md); \
                 color: var(--text-primary); \
                 overflow-y: auto; resize: none; \
                 {font_family} \
                 transition: border-color var(--duration-fast) var(--easing-standard), \
                             box-shadow var(--duration-fast) var(--easing-standard);"
            )
            on:input=move |ev| {
                let val = event_target_value(&ev);
                on_input(val);
            }
        >
            {move || value.get()}
        </textarea>
    }
}
