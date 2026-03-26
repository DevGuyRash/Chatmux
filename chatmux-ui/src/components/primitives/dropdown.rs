//! Dropdown select component.
//!
//! surface-sunken background, border-default, radius-md.
//! Dropdown menu appears as surface-overlay, shadow-md, z-dropdown.

use leptos::prelude::*;

/// A dropdown option.
#[derive(Clone, Debug)]
pub struct DropdownOption {
    pub value: String,
    pub label: String,
}

/// Dropdown select component.
#[component]
pub fn Dropdown(
    /// Available options.
    options: Vec<DropdownOption>,
    /// Currently selected value.
    selected: ReadSignal<String>,
    /// On change callback.
    on_change: impl Fn(String) + 'static,
    /// Placeholder when nothing is selected.
    #[prop(default = "Select…")]
    placeholder: &'static str,
    /// Whether the dropdown is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Optional aria-label.
    #[prop(optional, into)]
    aria_label: Option<String>,
) -> impl IntoView {
    view! {
        <div class="dropdown relative" style="display: inline-flex;">
            <select
                class="type-body cursor-pointer"
                style="\
                    padding: var(--space-3) var(--space-8) var(--space-3) var(--space-4); \
                    background: var(--surface-sunken); \
                    border: 1px solid var(--border-default); \
                    border-radius: var(--radius-md); \
                    color: var(--text-primary); \
                    appearance: none; \
                    -webkit-appearance: none; \
                    background-image: url(\"data:image/svg+xml,%3Csvg width='10' height='6' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%23888' fill='none' stroke-width='1.5'/%3E%3C/svg%3E\"); \
                    background-repeat: no-repeat; \
                    background-position: right var(--space-3) center; \
                    min-width: 120px;"
                disabled=disabled
                aria-label=aria_label
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    on_change(val);
                }
            >
                <option value="" disabled=true selected=move || selected.get().is_empty()>
                    {placeholder}
                </option>
                {options.into_iter().map(|opt| {
                    let value = opt.value.clone();
                    view! {
                        <option
                            value=opt.value.clone()
                            selected=move || selected.get() == value
                        >
                            {opt.label}
                        </option>
                    }
                }).collect_view()}
            </select>
        </div>
    }
}
