//! Checkbox component.
//!
//! Used in export selection, filter toggles, metadata toggles, etc.

use leptos::prelude::*;

/// Checkbox component.
#[component]
pub fn Checkbox(
    /// Whether the checkbox is checked.
    #[prop(into)]
    checked: Signal<bool>,
    /// On change callback.
    on_change: impl Fn(bool) + 'static,
    /// Optional label text.
    #[prop(optional, into)]
    label: Option<String>,
    /// Whether the checkbox is disabled.
    #[prop(default = false)]
    disabled: bool,
) -> impl IntoView {
    view! {
        <label
            class="checkbox flex items-center gap-3 cursor-pointer select-none"
            style=format!(
                "opacity: {};",
                if disabled { "0.5" } else { "1" },
            )
        >
            <span class="relative flex items-center justify-center" style="width: 16px; height: 16px; flex-shrink: 0;">
                <input
                    class="checkbox-box cursor-pointer"
                    type="checkbox"
                    prop:checked=move || checked.get()
                    disabled=disabled
                    aria-label=label.clone().unwrap_or_else(|| "Select item".to_owned())
                    style=move || format!(
                        "appearance: none; width: 16px; height: 16px; margin: 0; border-radius: var(--radius-sm); border: 1.5px solid {}; background: {}; transition: all var(--duration-instant) var(--easing-standard);",
                        if checked.get() { "var(--accent-primary)" } else { "var(--border-default)" },
                        if checked.get() { "var(--accent-primary)" } else { "transparent" },
                    )
                    on:change=move |event| {
                        if !disabled {
                            on_change(event_target_checked(&event));
                        }
                    }
                />
                {move || checked.get().then(|| view! {
                    <span aria-hidden="true" style="position: absolute; pointer-events: none; color: var(--text-inverse); font-size: var(--type-label-size); line-height: 1;">
                        "✓"
                    </span>
                })}
            </span>
            {label.map(|l| view! {
                <span class="type-body text-primary">{l}</span>
            })}
        </label>
    }
}
