//! Pinned summary manager (§3.25).
//!
//! List of pinned summaries + editor for creating/editing.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::empty_state::EmptyState;
use crate::components::primitives::icon::IconKind;
use crate::components::primitives::text_area::TextArea;
use crate::components::primitives::text_input::TextInput;

/// A pinned summary.
#[derive(Clone, Debug)]
pub struct PinnedSummary {
    pub id: uuid::Uuid,
    pub name: String,
    pub body: String,
    pub created_at: String,
    pub in_use: bool,
}

/// Pinned summary manager.
#[component]
pub fn PinnedSummaryManager(
    /// Summaries to display.
    summaries: ReadSignal<Vec<PinnedSummary>>,
    /// Called to save a summary.
    on_save: impl Fn(PinnedSummary) + 'static + Copy + Send,
) -> impl IntoView {
    let (editing_id, set_editing_id) = signal(None::<uuid::Uuid>);
    let (edit_name, set_edit_name) = signal(String::new());
    let (edit_body, set_edit_body) = signal(String::new());

    view! {
        <div class="pinned-summary-manager flex flex-col h-full">
            // Header
            <div class="flex items-center justify-between p-5"
                 style="border-bottom: 1px solid var(--border-subtle);">
                <span class="type-title text-primary">"Pinned Summaries"</span>
                <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                    set_editing_id.set(Some(uuid::Uuid::new_v4()));
                    set_edit_name.set(String::new());
                    set_edit_body.set(String::new());
                })>
                    "+ Create"
                </Button>
            </div>

            // List or empty state
            <div class="flex-1 overflow-y-auto">
                {move || {
                    let items = summaries.get();
                    if items.is_empty() && editing_id.get().is_none() {
                        view! {
                            <EmptyState
                                icon=IconKind::Pin
                                heading="No pinned summaries"
                                description="Create a pinned summary to use as compact context for catch-up rules."
                            >
                                <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                                    set_editing_id.set(Some(uuid::Uuid::new_v4()));
                                })>
                                    "Create Summary"
                                </Button>
                            </EmptyState>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex flex-col">
                                {items.into_iter().map(|summary| {
                                    view! {
                                        <div class="p-5" style="border-bottom: 1px solid var(--border-subtle);">
                                            <div class="flex items-center justify-between" style="margin-bottom: var(--space-2);">
                                                <span class="type-body-strong text-primary">{summary.name.clone()}</span>
                                                {summary.in_use.then(|| view! {
                                                    <span class="type-caption-strong"
                                                          style="color: var(--status-info-text); \
                                                                 background: var(--status-info-muted); \
                                                                 padding: var(--space-1) var(--space-3); \
                                                                 border-radius: var(--radius-sm);">
                                                        "In use"
                                                    </span>
                                                })}
                                            </div>
                                            <p class="type-caption text-secondary truncate" style="margin-bottom: var(--space-1);">
                                                {summary.body.chars().take(100).collect::<String>()}
                                            </p>
                                            <span class="type-caption text-tertiary">{summary.created_at}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}

                // Inline editor (when editing)
                {move || editing_id.get().map(|_| view! {
                    <div class="p-5" style="border-top: 1px solid var(--border-subtle);">
                        <div class="flex flex-col gap-4">
                            <TextInput
                                value=edit_name
                                on_input=move |v| set_edit_name.set(v)
                                placeholder="Summary name"
                            />
                            <TextArea
                                value=edit_body
                                on_input=move |v| set_edit_body.set(v)
                                placeholder="Write a concise summary of the conversation context…"
                                min_rows=4
                            />
                            <p class="type-caption text-secondary">
                                "This will be sent as the initial context when catch-up rule is set to 'Pinned Summary'."
                            </p>
                            <div class="flex gap-2 justify-end">
                                <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| set_editing_id.set(None))>
                                    "Cancel"
                                </Button>
                                <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                                    let summary = PinnedSummary {
                                        id: editing_id.get_untracked().unwrap_or_else(uuid::Uuid::new_v4),
                                        name: edit_name.get_untracked(),
                                        body: edit_body.get_untracked(),
                                        created_at: "Just now".to_string(),
                                        in_use: false,
                                    };
                                    on_save(summary);
                                    set_editing_id.set(None);
                                })>
                                    "Save"
                                </Button>
                            </div>
                        </div>
                    </div>
                })}
            </div>
        </div>
    }
}
