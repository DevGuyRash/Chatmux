//! Delivery cursor inspector (§3.10).
//!
//! Table view: one row per active edge (source→target pair).
//! Columns: Edge, Cursor Position, Status, Actions.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::provider::{Provider, provider_icon::ProviderIcon};
use crate::models::{DeliveryCursor, DeliveryCursorId};

/// Delivery cursor inspector.
#[component]
pub fn CursorInspector(
    /// Delivery cursors.
    cursors: ReadSignal<Vec<DeliveryCursor>>,
    /// Called to reset a cursor.
    on_reset: impl Fn(DeliveryCursorId) + 'static + Copy + Send,
    /// Called to freeze/unfreeze a cursor.
    on_toggle_freeze: impl Fn(DeliveryCursorId) + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <div class="cursor-inspector">
            <h3 class="type-title text-primary mb-5">
                "Delivery Cursors"
            </h3>

            {move || {
                let items = cursors.get();
                if items.is_empty() {
                    view! {
                        <p class="type-body text-secondary">"No active delivery cursors."</p>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex flex-col gap-3">
                            {items.into_iter().map(|cursor| {
                                let cursor_id = cursor.id;
                                let frozen = cursor.frozen;
                                let row_bg = if frozen { "background: var(--status-info-muted);" } else { "" };
                                let source = Provider::from_provider_id(cursor.source_participant_id);
                                let target = Provider::from_provider_id(cursor.target_participant_id);

                                view! {
                                    <div
                                        class="flex items-center gap-3 p-4 surface-raised rounded-md"
                                        style=format!("border: 1px solid var(--border-subtle); {row_bg}")
                                    >
                                        // Edge: source → target
                                        <div class="flex items-center gap-2 flex-1 min-w-0">
                                            <ProviderIcon provider=source size=14 />
                                            <span class="type-caption text-secondary">"→"</span>
                                            <ProviderIcon provider=target size=14 />
                                            <span class="type-body text-primary truncate">
                                                {format!("{} → {}", source.label(), target.label())}
                                            </span>
                                        </div>

                                        // Status badge
                                        <span class="type-caption-strong" style=format!(
                                            "color: var(--{}-text);",
                                            if frozen { "status-info" } else { "status-success" },
                                        )>
                                            {if frozen { "Frozen".to_string() } else { "Current".to_string() }}
                                        </span>

                                        // Actions
                                        <div class="flex gap-1">
                                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                                    on_click=Box::new(move |_| on_reset(cursor_id))>
                                                "Reset"
                                            </Button>
                                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                                    on_click=Box::new(move |_| on_toggle_freeze(cursor_id))>
                                                {if frozen { "Unfreeze" } else { "Freeze" }}
                                            </Button>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
