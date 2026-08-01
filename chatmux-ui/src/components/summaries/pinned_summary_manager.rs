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
    pub id: crate::models::MessageId,
    pub name: String,
    pub body: String,
    pub created_at: String,
    pub in_use: bool,
}

/// Pinned summary manager.
#[component]
pub fn PinnedSummaryManager(
    /// Summaries to display.
    summaries: Signal<Vec<PinnedSummary>>,
    /// Called to save a summary.
    on_save: impl Fn(PinnedSummary) + 'static + Copy + Send,
    /// Called to delete a summary.
    on_delete: impl Fn(crate::models::MessageId) + 'static + Copy + Send,
) -> impl IntoView {
    let (editing_id, set_editing_id) = signal(None::<crate::models::MessageId>);
    let (edit_name, set_edit_name) = signal(String::new());
    let (edit_body, set_edit_body) = signal(String::new());

    view! {
        <div class="pinned-summary-manager flex flex-col h-full">
            // Header
            <div class="flex items-center justify-between p-5 border-b">
                <span class="type-title text-primary">"Pinned Summaries"</span>
                <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                    set_editing_id.set(Some(crate::models::MessageId::new()));
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
                                    set_editing_id.set(Some(crate::models::MessageId::new()));
                                    set_edit_name.set(String::new());
                                    set_edit_body.set(String::new());
                                })>
                                    "Create summary"
                                </Button>
                            </EmptyState>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex flex-col">
                                {items.into_iter().map(|summary| {
                                    let summary_for_edit = summary.clone();
                                    let summary_for_duplicate = summary.clone();
                                    let summary_id = summary.id;
                                    view! {
                                        <div class="p-5 border-b">
                                            <div class="flex items-center justify-between mb-2">
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
                                            <p class="type-caption text-secondary truncate mb-1">
                                                {summary.body.chars().take(100).collect::<String>()}
                                            </p>
                                            <span class="type-caption text-tertiary">{summary.created_at}</span>
                                            <div class="flex items-center gap-2 mt-3">
                                                <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| {
                                                    set_editing_id.set(Some(summary_for_edit.id));
                                                    set_edit_name.set(summary_for_edit.name.clone());
                                                    set_edit_body.set(summary_for_edit.body.clone());
                                                })>
                                                    "Edit"
                                                </Button>
                                                <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| {
                                                    set_editing_id.set(Some(crate::models::MessageId::new()));
                                                    set_edit_name.set(format!("{} Copy", summary_for_duplicate.name));
                                                    set_edit_body.set(summary_for_duplicate.body.clone());
                                                })>
                                                    "Duplicate"
                                                </Button>
                                                <Button variant=ButtonVariant::Ghost on_click=Box::new(move |_| on_delete(summary_id))>
                                                    "Delete"
                                                </Button>
                                            </div>
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
                            <span class="type-caption text-tertiary">
                                {move || format!("{} characters", edit_body.get().chars().count())}
                            </span>
                            <p class="type-caption text-secondary">
                                "This will be sent as the initial context when catch-up rule is set to 'Pinned Summary'."
                            </p>
                            <div class="flex gap-2 justify-end">
                                <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| set_editing_id.set(None))>
                                    "Cancel"
                                </Button>
                                <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                                    let summary = PinnedSummary {
                                        id: editing_id.get_untracked().unwrap_or_else(crate::models::MessageId::new),
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
