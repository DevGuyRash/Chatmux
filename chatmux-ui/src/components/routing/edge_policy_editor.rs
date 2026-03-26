//! Edge policy editor (§3.9).
//!
//! Dual-zone layout: graph visualization (full-tab) or edge list (sidebar)
//! on one side, edge detail panel on the other.

use leptos::prelude::*;

use crate::layout::responsive::LayoutMode;
use crate::models::{EdgePolicy, EdgePolicyId};
use super::edge_list::EdgeList;
use super::edge_detail_panel::EdgeDetailPanel;

/// Edge policy editor — the routing configuration view.
#[component]
pub fn EdgePolicyEditor(
    /// Edge policies.
    edges: ReadSignal<Vec<EdgePolicy>>,
    /// Called when an edge is updated.
    on_update: impl Fn(EdgePolicy) + 'static + Copy + Send,
) -> impl IntoView {
    let layout_mode = expect_context::<ReadSignal<LayoutMode>>();
    let (selected_edge_id, set_selected_edge_id) = signal(None::<EdgePolicyId>);

    let selected_edge = Signal::derive(move || {
        let id = selected_edge_id.get()?;
        edges.get().into_iter().find(|e| e.id == id)
    });

    view! {
        <div class="edge-policy-editor flex h-full" style=move || match layout_mode.get() {
            LayoutMode::Sidebar => "flex-direction: column;",
            LayoutMode::FullTab => "flex-direction: row;",
        }>
            // Zone 1: Graph/list
            <div style=move || match layout_mode.get() {
                LayoutMode::Sidebar => "width: 100%;",
                LayoutMode::FullTab => "width: 40%; border-right: 1px solid var(--border-subtle); overflow-y: auto;",
            }>
                <EdgeList
                    edges=edges
                    on_select=move |id| set_selected_edge_id.set(Some(id))
                    on_toggle=move |id, enabled| {
                        // TODO(backend): Update edge enabled state
                        log::info!("Toggle edge {:?} to {}", id, enabled);
                    }
                />
            </div>

            // Zone 2: Detail panel
            <div class="flex-1 overflow-y-auto p-5" style=move || match layout_mode.get() {
                LayoutMode::Sidebar => "",
                LayoutMode::FullTab => "min-width: 300px;",
            }>
                {move || match selected_edge.get() {
                    Some(edge) => view! {
                        <EdgeDetailPanel edge=edge on_save=on_update />
                    }.into_any(),
                    None => view! {
                        <div class="flex items-center justify-center h-full">
                            <p class="type-body text-secondary">
                                "Select an edge to configure its routing policy."
                            </p>
                        </div>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}
