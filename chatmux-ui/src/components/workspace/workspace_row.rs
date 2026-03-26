//! Individual workspace list row (§3.1).
//!
//! Workspace name (left) + status summary badges (right).
//! States: Default, Hover, Active (selected), Archived.

use leptos::prelude::*;

use crate::components::primitives::badge::{Badge, BadgeVariant};
use crate::models::Workspace;

/// A single workspace row in the workspace list.
#[component]
pub fn WorkspaceRow(
    /// Workspace data.
    workspace: Workspace,
    /// Click handler.
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let is_archived = workspace.archived;

    view! {
        <button
            class="workspace-row flex items-center justify-between w-full cursor-pointer select-none transition-colors"
            style=format!(
                "padding: var(--space-5) var(--space-6); \
                 border-bottom: 1px solid var(--border-subtle); \
                 background: var(--surface-raised); \
                 text-align: left; border: none; border-bottom: 1px solid var(--border-subtle); \
                 color: {};",
                if is_archived { "var(--text-tertiary)" } else { "var(--text-primary)" },
            )
            on:click=move |_| on_click()
        >
            // Left: workspace name
            <span class="type-body-strong truncate" style=format!(
                "max-width: 200px;{}",
                if is_archived { " font-style: italic;" } else { "" },
            )>
                {workspace.name}
            </span>

            // Right: status badges
            <div class="flex items-center gap-2">
                {is_archived.then(|| view! {
                    <Badge variant=BadgeVariant::Neutral>"Archived"</Badge>
                })}
            </div>
        </button>
    }
}
