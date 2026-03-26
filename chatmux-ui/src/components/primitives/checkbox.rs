//! Checkbox component.
//!
//! Used in export selection, filter toggles, metadata toggles, etc.

use leptos::prelude::*;

/// Checkbox component.
#[component]
pub fn Checkbox(
    /// Whether the checkbox is checked.
    checked: ReadSignal<bool>,
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
            <span
                class="checkbox-box flex items-center justify-center"
                style=move || format!(
                    "width: 16px; height: 16px; \
                     border-radius: var(--radius-sm); \
                     border: 1.5px solid {}; \
                     background: {}; \
                     transition: all var(--duration-instant) var(--easing-standard); \
                     flex-shrink: 0;",
                    if checked.get() { "var(--accent-primary)" } else { "var(--border-default)" },
                    if checked.get() { "var(--accent-primary)" } else { "transparent" },
                )
                on:click=move |_| {
                    if !disabled {
                        on_change(!checked.get_untracked());
                    }
                }
            >
                {move || checked.get().then(|| view! {
                    <span style="color: var(--text-inverse); font-size: 10px; line-height: 1;">
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
