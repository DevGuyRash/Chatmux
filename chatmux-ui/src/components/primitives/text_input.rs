//! Single-line text input component.
//!
//! surface-sunken background, border-default border, radius-md.
//! Focus state: border-accent with subtle glow.

use leptos::prelude::*;

/// Single-line text input.
#[component]
pub fn TextInput(
    /// Current value.
    value: ReadSignal<String>,
    /// On change callback.
    on_input: impl Fn(String) + 'static,
    /// Placeholder text.
    #[prop(default = "")]
    placeholder: &'static str,
    /// Whether the input is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Optional accessible label.
    #[prop(optional, into)]
    aria_label: Option<String>,
    /// Optional type attribute (e.g., "number").
    #[prop(default = "text")]
    input_type: &'static str,
) -> impl IntoView {
    view! {
        <input
            class="text-input type-body"
            type=input_type
            value=move || value.get()
            placeholder=placeholder
            disabled=disabled
            aria-label=aria_label
            style="\
                width: 100%; \
                padding: var(--space-4) var(--space-4); \
                background: var(--surface-sunken); \
                border: 1px solid var(--border-default); \
                border-radius: var(--radius-md); \
                color: var(--text-primary); \
                transition: border-color var(--duration-fast) var(--easing-standard), \
                            box-shadow var(--duration-fast) var(--easing-standard);"
            on:input=move |ev| {
                let val = event_target_value(&ev);
                on_input(val);
            }
        />
    }
}
