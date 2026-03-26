//! Toggle switch component.
//!
//! A sliding toggle switch for boolean settings. Uses accent-primary
//! when active, surface-sunken when inactive.

use leptos::prelude::*;

/// Toggle switch component.
#[component]
pub fn Toggle(
    /// Whether the toggle is on.
    checked: ReadSignal<bool>,
    /// Callback when toggled.
    on_change: impl Fn(bool) + 'static,
    /// Whether the toggle is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Accessible label.
    #[prop(optional, into)]
    aria_label: Option<String>,
) -> impl IntoView {
    view! {
        <button
            class="toggle select-none"
            role="switch"
            aria-checked=move || if checked.get() { "true" } else { "false" }
            aria-label=aria_label
            disabled=disabled
            style=move || format!(
                "position: relative; width: 36px; height: 20px; \
                 border-radius: var(--radius-full); border: none; \
                 cursor: {}; \
                 background: {}; \
                 transition: background var(--duration-fast) var(--easing-standard); \
                 opacity: {};",
                if disabled { "not-allowed" } else { "pointer" },
                if checked.get() { "var(--accent-primary)" } else { "var(--border-default)" },
                if disabled { "0.5" } else { "1" },
            )
            on:click=move |_| {
                if !disabled {
                    on_change(!checked.get_untracked());
                }
            }
        >
            <span
                style=move || format!(
                    "position: absolute; top: 2px; \
                     left: {}px; \
                     width: 16px; height: 16px; \
                     border-radius: var(--radius-full); \
                     background: var(--text-inverse); \
                     transition: left var(--duration-fast) var(--easing-spring); \
                     box-shadow: var(--shadow-sm);",
                    if checked.get() { 18 } else { 2 },
                )
            />
        </button>
    }
}
