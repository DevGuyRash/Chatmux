//! Tooltip component.
//!
//! surface-overlay background, shadow-md, radius-md.
//! z-index: z-tooltip (500). Appears on hover with duration-fast delay.

use leptos::prelude::*;

/// Tooltip wrapper — wraps a child element and shows a tooltip on hover.
#[component]
pub fn Tooltip(
    /// Tooltip text.
    text: &'static str,
    /// The element to attach the tooltip to.
    children: Children,
) -> impl IntoView {
    // Simple title-attribute tooltip for now.
    // A proper floating tooltip with positioning will be implemented
    // in Phase 9 polish using a portal and pointer tracking.
    view! {
        <span
            class="tooltip-wrapper relative"
            title=text
            style="display: inline-flex;"
        >
            {children()}
        </span>
    }
}
