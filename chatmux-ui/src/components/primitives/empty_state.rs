//! Empty state component (§3.18).
//!
//! Centered layout with: icon placeholder (64×64px), heading (type-subtitle),
//! description (type-body, text-secondary), and optional CTA button.

use leptos::prelude::*;

use super::icon::{Icon, IconKind};

/// Empty state component.
#[component]
pub fn EmptyState(
    /// Icon to display.
    icon: IconKind,
    /// Heading text.
    heading: &'static str,
    /// Description text.
    description: &'static str,
    /// Optional CTA button content.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <div
            class="empty-state flex flex-col items-center justify-center"
            style="padding: var(--space-10) var(--space-6); text-align: center; gap: var(--space-4);"
        >
            <div style="color: var(--text-tertiary);">
                <Icon kind=icon size=64 />
            </div>
            <h3 class="type-subtitle text-primary">{heading}</h3>
            <p class="type-body text-secondary" style="max-width: 280px;">
                {description}
            </p>
            {children.map(|c| view! { <div style="margin-top: var(--space-4);">{c()}</div> })}
        </div>
    }
}
