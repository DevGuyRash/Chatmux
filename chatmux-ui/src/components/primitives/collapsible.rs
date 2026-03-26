//! Collapsible section component.
//!
//! Expand/collapse with height animation (duration-normal, easing-standard).
//! Content fades in on expand, fades out before collapse.

use leptos::prelude::*;

/// Collapsible section with a clickable header.
#[component]
pub fn Collapsible(
    /// Header label.
    label: String,
    /// Whether initially expanded.
    #[prop(default = false)]
    default_open: bool,
    /// Section content.
    children: Children,
) -> impl IntoView {
    let (open, set_open) = signal(default_open);

    // Render children once and store — Children is FnOnce
    let rendered_children = children();

    view! {
        <div class="collapsible">
            // Header (clickable)
            <button
                class="collapsible-header flex items-center gap-2 w-full cursor-pointer select-none"
                style="\
                    padding: var(--space-3) 0; \
                    background: none; border: none; \
                    color: var(--text-primary); text-align: left;"
                aria-expanded=move || if open.get() { "true" } else { "false" }
                on:click=move |_| set_open.update(|v| *v = !*v)
            >
                <span
                    class="transition-transform"
                    style=move || format!(
                        "display: inline-block; font-size: 10px; \
                         transform: rotate({}deg);",
                        if open.get() { 90 } else { 0 },
                    )
                >
                    "▸"
                </span>
                <span class="type-body-strong">{label}</span>
            </button>

            // Content — hidden via CSS display when collapsed
            <div
                class="collapsible-content"
                style=move || format!(
                    "padding-left: var(--space-6); padding-bottom: var(--space-4); \
                     display: {};",
                    if open.get() { "block" } else { "none" },
                )
            >
                {rendered_children}
            </div>
        </div>
    }
}
