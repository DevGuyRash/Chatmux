//! Divider component — horizontal or vertical separator line.
//!
//! Replaces inline `width: 1px; background: var(--border-subtle)` patterns.

use leptos::prelude::*;

/// Semantic separator line.
#[component]
pub fn Divider(
    /// Render as vertical (inline) divider instead of horizontal.
    #[prop(default = false)]
    vertical: bool,
) -> impl IntoView {
    let class = if vertical { "divider-v" } else { "divider-h" };
    view! {
        <div class=class role="separator" aria-orientation=if vertical { "vertical" } else { "horizontal" } />
    }
}
