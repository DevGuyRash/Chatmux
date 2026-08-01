//! Workspace list component (§3.1).
//!
//! First view when the extension opens. Vertical scrolling list.
//! Archive filter (Active/Archived segmented control).
//! "New workspace" button at top.

use leptos::prelude::*;

use super::workspace_row::WorkspaceRow;
use crate::components::primitives::button::{Button, ButtonVariant};
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
    /// Called when "New workspace" is clicked.
    on_create: impl Fn() + 'static + Copy + Send,
    /// Called when a workspace should be deleted.
    /// Deleting a workspace always removes everything stored under it, so there
    /// is nothing to opt into here.
    on_delete: impl Fn(WorkspaceId) + 'static + Copy + Send,
    /// Called after the user confirms a new workspace name.
    on_rename: impl Fn(WorkspaceId, String) + 'static + Copy + Send,
    /// Called when a workspace should be duplicated.
    on_duplicate: impl Fn(WorkspaceId) + 'static + Copy + Send,
    /// Called when a workspace should be archived or restored.
    on_archive: impl Fn(WorkspaceId, bool) + 'static + Copy + Send,
) -> impl IntoView {
    let (filter, set_filter) = signal("active".to_string());

    // Confirmation dialog state: which workspace is pending deletion.
    let (pending_delete, set_pending_delete) = signal(None::<(WorkspaceId, String)>);
    let (confirm_open, set_confirm_open) = signal(false);
    let (pending_rename, set_pending_rename) = signal(None::<WorkspaceId>);
    let (rename_value, set_rename_value) = signal(String::new());
    let rename_open = Signal::derive(move || pending_rename.get().is_some());

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
                        // The Archived tab needs its own empty state. Offering
                        // "Create workspace" here does not fail harmlessly — it
                        // succeeds at the wrong thing, adding an unwanted active
                        // workspace the user cannot see from the tab they are on.
                        if filter.get() == "archived" {
                            view! {
                                <EmptyState
                                    icon=IconKind::StackedRectangles
                                    heading="No archived workspaces"
                                    description="Archive a workspace from its row menu to park it here without deleting anything."
                                />
                            }.into_any()
                        } else {
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
                                        "Create workspace"
                                    </Button>
                                </EmptyState>
                            }.into_any()
                        }
                    } else {
                        view! {
                            <div>
                                {items.into_iter().map(|ws| {
                                    let id = ws.id;
                                    let name = ws.name.clone();
                                    let delete_name = name.clone();
                                    let archived = ws.archived;
                                    view! {
                                        <WorkspaceRow
                                            workspace=ws
                                            on_click=move || on_select(id)
                                            on_delete=move || {
                                                set_pending_delete.set(Some((id, delete_name.clone())));
                                            }
                                            on_rename=move || {
                                                set_rename_value.set(name.clone());
                                                set_pending_rename.set(Some(id));
                                            }
                                            on_duplicate=move || on_duplicate(id)
                                            on_archive=move || on_archive(id, !archived)
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
                    <h2 class="type-title text-primary">"Delete workspace"</h2>
                    <p class="type-body text-secondary">
                        {move || {
                            let name = pending_delete.get()
                                .map(|(_, n)| n)
                                .unwrap_or_default();
                            format!("Delete \u{201c}{name}\u{201d} and everything stored under it. This cannot be undone.")
                        }}
                    </p>

                    // Enumerate what goes. Deletion is not partial and there is
                    // no undo, so the dialog names the cost rather than leaving
                    // the operator to infer it.
                    <div class="surface-card p-5">
                        <span class="type-label micro-label">"This removes"</span>
                        <ul class="flex flex-col gap-2 mt-3 type-caption text-secondary">
                            <li>"Every message, run and dispatch record"</li>
                            <li>"Provider bindings and delivery cursors"</li>
                            <li>"Routing policies, templates and export profiles"</li>
                            <li>"All diagnostics for this workspace"</li>
                        </ul>
                        <p class="type-caption text-tertiary mt-3">
                            "To empty the transcript but keep this setup, use Clear conversation history in Settings."
                        </p>
                    </div>

                    // Action buttons
                    <div class="flex justify-end gap-3" style="margin-top: var(--space-3);">
                        <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| set_pending_delete.set(None))>
                            "Cancel"
                        </Button>
                        <Button variant=ButtonVariant::Danger on_click=Box::new(move |_| {
                                if let Some((id, _)) = pending_delete.get_untracked() {
                                    set_pending_delete.set(None);
                                    on_delete(id);
                                }
                            })>
                            "Delete workspace"
                        </Button>
                    </div>
                </div>
            </Modal>

            <Modal
                open=rename_open
                on_close=move || set_pending_rename.set(None)
                max_width=420
                aria_label="Rename workspace".to_string()
            >
                <div class="flex flex-col gap-5">
                    <h2 class="type-title text-primary">"Rename workspace"</h2>
                    <label class="type-label text-secondary" for="workspace-rename">"Workspace name"</label>
                    <input
                        id="workspace-rename"
                        class="type-body text-primary surface-sunken border rounded-md"
                        style="padding: var(--space-3) var(--space-4);"
                        prop:value=move || rename_value.get()
                        on:input=move |event| set_rename_value.set(event_target_value(&event))
                    />
                    <div class="flex justify-end gap-3">
                        <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| set_pending_rename.set(None))>
                            "Cancel"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            disabled=Signal::derive(move || rename_value.get().trim().is_empty())
                            on_click=Box::new(move |_| {
                                if let Some(id) = pending_rename.get_untracked() {
                                    let name = rename_value.get_untracked().trim().to_owned();
                                    set_pending_rename.set(None);
                                    on_rename(id, name);
                                }
                            })
                        >
                            "Save name"
                        </Button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}
