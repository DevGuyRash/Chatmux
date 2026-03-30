//! Workspace list component (§3.1).
//!
//! First view when the extension opens. Vertical scrolling list.
//! Archive filter (Active/Archived segmented control).
//! "New Workspace" button at top.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::empty_state::EmptyState;
use crate::components::primitives::icon::IconKind;
use crate::components::primitives::segmented_control::{Segment, SegmentedControl};
use crate::models::Workspace;
use super::workspace_row::WorkspaceRow;

/// Workspace list component.
#[component]
pub fn WorkspaceList(
    /// List of workspaces to display.
    workspaces: ReadSignal<Vec<Workspace>>,
    /// Called when a workspace is selected.
    on_select: impl Fn(crate::models::WorkspaceId) + 'static + Copy + Send,
    /// Called when "New Workspace" is clicked.
    on_create: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    let (filter, set_filter) = signal("active".to_string());

    let filtered = move || {
        let ws = workspaces.get();
        let f = filter.get();
        ws.into_iter()
            .filter(|w| {
                if f == "archived" { w.archived } else { !w.archived }
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
                                    view! {
                                        <WorkspaceRow
                                            workspace=ws
                                            on_click=move || on_select(id)
                                        />
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
