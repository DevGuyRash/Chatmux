//! Surface component — card-like container with design system backgrounds.
//!
//! Replaces inline `background: var(--surface-sunken); border: 1px solid ...` patterns.

use leptos::prelude::*;

/// Surface visual variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SurfaceVariant {
    /// Sunken background — form groups, toolbars, embedded panels.
    #[default]
    Sunken,
    /// Raised background — detail panels, overlays.
    Raised,
}

/// Card-like container with consistent surface styling.
#[component]
pub fn Surface(
    /// Surface variant.
    #[prop(default = SurfaceVariant::Sunken)]
    variant: SurfaceVariant,
    /// Additional CSS classes to compose.
    #[prop(optional, into)]
    class: Option<String>,
    /// Container content.
    children: Children,
) -> impl IntoView {
    let base_class = match variant {
        SurfaceVariant::Sunken => "surface-card",
        SurfaceVariant::Raised => "surface-card-raised",
    };
    let full_class = match &class {
        Some(extra) => format!("{base_class} {extra}"),
        None => base_class.to_owned(),
    };
    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}
