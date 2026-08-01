//! Edge list for sidebar mode (§3.9).
//!
//! Compact list of edges. Each row: "Source → Target" with toggle
//! and expand chevron. Expanding shows edge detail form inline.

use std::collections::BTreeSet;

use leptos::prelude::*;

use crate::components::primitives::icon::{Icon, IconKind};
use crate::components::primitives::toggle::Toggle;
use crate::components::provider::{Provider, provider_icon::ProviderIcon};
use crate::models::{EdgePolicy, EdgePolicyId};

/// Edge list component (sidebar variant of the routing editor).
#[component]
pub fn EdgeList(
    /// Edge policies to display.
    edges: Signal<Vec<EdgePolicy>>,
    /// Called when an edge is selected for editing.
    on_select: impl Fn(EdgePolicyId) + 'static + Copy + Send,
    /// Called when an edge is toggled enabled/disabled.
    on_toggle: impl Fn(EdgePolicyId, bool) + 'static + Copy + Send,
) -> impl IntoView {
    // Held at component level, keyed by edge id. Created inside the row closure
    // instead, it is rebuilt every time `edges` emits — so toggling one route
    // silently collapsed every other expanded row.
    let (expanded_ids, set_expanded_ids) = signal(BTreeSet::<EdgePolicyId>::new());

    view! {
        <div class="edge-list flex flex-col">
            {move || {
                let mut policies = edges.get();
                policies.sort_by_key(|edge| (
                    edge.priority,
                    edge.source_participant_id,
                    edge.target_participant_id,
                ));
                policies.into_iter().map(|edge| {
                let edge_id = edge.id;
                let (enabled, set_enabled) = signal(edge.enabled);
                let expanded = Signal::derive(move || expanded_ids.get().contains(&edge_id));

                view! {
                    <div
                        class="edge-row border-b"
                    >
                        // Main row
                        <div
                            class="flex items-center gap-3 cursor-pointer"
                            style="padding: var(--space-4) var(--space-5);"
                        >
                            <Toggle
                                checked=enabled
                                on_change=move |v| {
                                    set_enabled.set(v);
                                    on_toggle(edge_id, v);
                                }
                            />
                            <ProviderIcon provider=Provider::from_provider_id(edge.source_participant_id) size=14 />
                            <span class="type-body text-primary">"→"</span>
                            <ProviderIcon provider=Provider::from_provider_id(edge.target_participant_id) size=14 />
                            <span class="type-body text-primary flex-1">
                                {format!("{} → {}",
                                    Provider::from_provider_id(edge.source_participant_id).label(),
                                    Provider::from_provider_id(edge.target_participant_id).label())}
                            </span>

                            // Priority badge
                            <span
                                class="type-caption surface-sunken"
                                style="width: 20px; height: 20px; border-radius: var(--radius-full); \
                                       display: flex; align-items: center; justify-content: center;"
                            >
                                {edge.priority.to_string()}
                            </span>

                            // Expand chevron
                            <button
                                class="cursor-pointer transition-transform"
                                style=move || format!(
                                    "background: none; border: none; color: var(--text-secondary); \
                                     transform: rotate({}deg);",
                                    if expanded.get() { 90 } else { 0 },
                                )
                                aria-label=format!(
                                    "Edit {} to {} edge",
                                    Provider::from_provider_id(edge.source_participant_id).label(),
                                    Provider::from_provider_id(edge.target_participant_id).label(),
                                )
                                on:click=move |_| {
                                    set_expanded_ids.update(|ids| {
                                        if !ids.remove(&edge_id) {
                                            ids.insert(edge_id);
                                        }
                                    });
                                    on_select(edge_id);
                                }
                            >
                                <Icon kind=IconKind::ChevronRight size=14 />
                            </button>
                        </div>

                        // Expanded detail (inline)
                        {move || expanded.get().then(|| view! {
                            <div
                                class="px-5 py-4 surface-sunken border-t"
                            >
                                <p class="type-caption text-secondary">
                                    {format!(
                                        "Catch-up: {} · Incremental: {} · Approval: {}",
                                        crate::models::view_models::catch_up_policy_label(&edge.catch_up_policy),
                                        crate::models::view_models::incremental_policy_label(&edge.incremental_policy),
                                        if edge.approval_mode != crate::models::ApprovalMode::AutoSend { "Required" } else { "Not required" },
                                    )}
                                </p>
                            </div>
                        })}
                    </div>
                }
                }).collect_view()
            }}
        </div>
    }
}
