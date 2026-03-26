//! Skeleton shimmer loader component.
//!
//! Used for loading states across message cards, binding cards,
//! template previews, and inspection panels.

use leptos::prelude::*;

/// Skeleton shimmer placeholder.
#[component]
pub fn Skeleton(
    /// Width (CSS value, e.g., "100%", "200px").
    #[prop(default = "100%")]
    width: &'static str,
    /// Height (CSS value, e.g., "16px", "1em").
    #[prop(default = "16px")]
    height: &'static str,
    /// Border radius.
    #[prop(default = "var(--radius-sm)")]
    radius: &'static str,
) -> impl IntoView {
    view! {
        <div
            class="skeleton"
            aria-hidden="true"
            style=format!(
                "width: {width}; height: {height}; \
                 border-radius: {radius}; \
                 background: linear-gradient(90deg, \
                     var(--surface-sunken) 25%, \
                     var(--border-subtle) 50%, \
                     var(--surface-sunken) 75%); \
                 background-size: 200% 100%; \
                 animation: skeleton-shimmer 1.5s infinite;"
            )
        />
    }
}
