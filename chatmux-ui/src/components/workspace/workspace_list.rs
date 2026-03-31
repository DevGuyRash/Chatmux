//! Workspace list component (§3.1).
//!
//! First view when the extension opens. Vertical scrolling list.
//! Archive filter (Active/Archived segmented control).
//! "New Workspace" button at top.

use leptos::prelude::*;

use super::workspace_row::WorkspaceRow;
use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::checkbox::Checkbox;
use crate::components::primitives::empty_state::EmptyState;
use crate::components::primitives::icon::IconKind;
use crate::components::primitives::modal::Modal;
use crate::components::primitives::segmented_control::{Segment, SegmentedControl};
use crate::models::{Workspace, WorkspaceId};

/// Workspace list component.
#[component]
pub fn WorkspaceList(
    /// List of workspaces to display.
    workspaces: ReadSignal<Vec<Workspace>>,
    /// Called when a workspace is selected.
    on_select: impl Fn(crate::models::WorkspaceId) + 'static + Copy + Send,
    /// Called when "New Workspace" is clicked.
    on_create: impl Fn() + 'static + Copy + Send,
    /// Called when a workspace should be deleted.
    /// Arguments: (workspace_id, also_delete_chat_data).
    on_delete: impl Fn(WorkspaceId, bool) + 'static + Copy + Send,
) -> impl IntoView {
    let (filter, set_filter) = signal("active".to_string());

    // Confirmation dialog state: which workspace is pending deletion.
    let (pending_delete, set_pending_delete) = signal(None::<(WorkspaceId, String)>);
    let (delete_chat, set_delete_chat) = signal(false);
    let (confirm_open, set_confirm_open) = signal(false);

    // Keep confirm_open in sync with pending_delete.
    Effect::new(move |_| {
        set_confirm_open.set(pending_delete.get().is_some());
    });

    let filtered = move || {
        let ws = workspaces.get();
        let f = filter.get();
        ws.into_iter()
            .filter(|w| {
                if f == "archived" {
                    w.archived
                } else {
                    !w.archived
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="workspace-list flex flex-col h-full">
            // Header with create button
            <div class="flex items-center justify-between p-5 border-b">
                <span class="type-title text-primary">"Workspaces"</span>
                <Button
                    variant=ButtonVariant::Primary
                    on_click=Box::new(move |_| on_create())
                >
                    "+ New Workspace"
                </Button>
            </div>

            // Archive filter
            <div class="px-5 py-3">
                <SegmentedControl
                    segments=vec![
                        Segment { value: "active".to_string(), label: "Active".to_string() },
                        Segment { value: "archived".to_string(), label: "Archived".to_string() },
                    ]
                    selected=filter
                    on_change=move |v| set_filter.set(v)
                    aria_label="Filter workspaces"
                />
            </div>

            // Workspace rows
            <div class="flex-1 overflow-y-auto">
                {move || {
                    let items = filtered();
                    if items.is_empty() {
                        view! {
                            <EmptyState
                                icon=IconKind::StackedRectangles
                                heading="No workspaces yet"
                                description="Create a workspace to start orchestrating conversations across AI providers."
                            >
                                <Button
                                    variant=ButtonVariant::Primary
                                    on_click=Box::new(move |_| on_create())
                                >
                                    "Create Workspace"
                                </Button>
                            </EmptyState>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                {items.into_iter().map(|ws| {
                                    let id = ws.id;
                                    let name = ws.name.clone();
                                    view! {
                                        <WorkspaceRow
                                            workspace=ws
                                            on_click=move || on_select(id)
                                            on_delete=move || {
                                                set_delete_chat.set(false);
                                                set_pending_delete.set(Some((id, name.clone())));
                                            }
                                        />
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            // Delete confirmation dialog
            <Modal
                open=confirm_open
                on_close=move || set_pending_delete.set(None)
                max_width=400
                aria_label="Delete workspace confirmation".to_string()
            >
                <div class="flex flex-col gap-5">
                    <h2 class="type-title text-primary">"Delete Workspace"</h2>
                    <p class="type-body text-secondary">
                        {move || {
                            let name = pending_delete.get()
                                .map(|(_, n)| n)
                                .unwrap_or_default();
                            format!("Are you sure you want to delete \u{201c}{name}\u{201d}? This action cannot be undone.")
                        }}
                    </p>

                    // Checkbox: also delete chat data
                    <div
                        class="surface-card"
                        style="\
                            padding: var(--space-4) var(--space-5); \
                            margin-top: var(--space-2);"
                    >
                        <Checkbox
                            checked=delete_chat
                            on_change=move |v| set_delete_chat.set(v)
                            label="Also delete all conversation history".to_string()
                        />
                    </div>

                    // Action buttons
                    <div class="flex justify-end gap-3" style="margin-top: var(--space-3);">
                        <button
                            class="type-label select-none cursor-pointer"
                            style="\
                                padding: var(--space-3) var(--space-5); \
                                background: var(--surface-sunken); \
                                color: var(--text-primary); \
                                border: 1px solid var(--border-default); \
                                border-radius: var(--radius-md);"
                            on:click=move |_| set_pending_delete.set(None)
                        >
                            "Cancel"
                        </button>
                        <button
                            class="type-label select-none cursor-pointer"
                            style="\
                                padding: var(--space-3) var(--space-5); \
                                background: var(--status-error-solid); \
                                color: var(--text-inverse); \
                                border: none; \
                                border-radius: var(--radius-md);"
                            on:click=move |_| {
                                if let Some((id, _)) = pending_delete.get_untracked() {
                                    let also_clear = delete_chat.get_untracked();
                                    set_pending_delete.set(None);
                                    on_delete(id, also_clear);
                                }
                            }
                        >
                            "Delete"
                        </button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}
