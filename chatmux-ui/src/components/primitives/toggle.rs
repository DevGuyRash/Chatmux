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
            class:toggle--on=move || checked.get()
            role="switch"
            aria-checked=move || if checked.get() { "true" } else { "false" }
            aria-label=aria_label
            disabled=disabled
            on:click=move |_| {
                if !disabled {
                    on_change(!checked.get_untracked());
                }
            }
        >
            <span class="toggle__knob" aria-hidden="true" />
        </button>
    }
}
