//! Edge detail panel (§3.9).
//!
//! Form for editing a single edge policy. Shows all fields:
//! Enabled, Catch-Up Rule, Incremental Rule, Self-Exclusion, etc.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::checkbox::Checkbox;
use crate::components::primitives::toggle::Toggle;
use crate::components::provider::{Provider, provider_icon::ProviderIcon};
use crate::models::{ApprovalMode, CatchUpPolicy, EdgePolicy, IncrementalPolicy, MessageId};

/// Edge detail editing panel.
#[component]
pub fn EdgeDetailPanel(
    /// The edge policy being edited.
    edge: EdgePolicy,
    /// Pinned summaries that may be selected for initial catch-up.
    summaries: Signal<Vec<(MessageId, String)>>,
    /// Called when changes are saved.
    on_save: impl Fn(EdgePolicy) + 'static + Copy + Send,
) -> impl IntoView {
    let (enabled, set_enabled) = signal(edge.enabled);
    let (self_exclusion, set_self_exclusion) = signal(edge.self_exclusion);
    let (include_user, set_include_user) = signal(edge.include_user_turns);
    let (include_system, set_include_system) = signal(edge.include_system_notes);
    let (include_summaries, set_include_summaries) = signal(edge.include_pinned_summaries);
    let (approval, set_approval) = signal(edge.approval_mode != ApprovalMode::AutoSend);
    let (catch_up, set_catch_up) = signal(catch_up_key(&edge.catch_up_policy).to_owned());
    let (last_n, set_last_n) = signal(match edge.catch_up_policy {
        CatchUpPolicy::LastN { count } => count.max(1),
        _ => 10,
    });
    let (summary_id, set_summary_id) = signal(match edge.catch_up_policy {
        CatchUpPolicy::PinnedSummary { summary_message_id } => summary_message_id,
        _ => None,
    });
    let (incremental, set_incremental) =
        signal(incremental_key(&edge.incremental_policy).to_owned());
    let edge_for_save = edge.clone();

    view! {
        <div class="edge-detail flex flex-col gap-4">
            // Header: Source → Target
            {
                let source = Provider::from_provider_id(edge.source_participant_id);
                let target = Provider::from_provider_id(edge.target_participant_id);
                view! {
                    <div class="flex items-center gap-3">
                        <ProviderIcon provider=source size=20 />
                        <span class="type-subtitle" style=format!("color: {};", source.text_color())>
                            {source.label()}
                        </span>
                        <span class="type-body text-secondary">"→"</span>
                        <ProviderIcon provider=target size=20 />
                        <span class="type-subtitle" style=format!("color: {};", target.text_color())>
                            {target.label()}
                        </span>
                    </div>
                }
            }

            // Toggle fields
            <FieldToggle label="Enabled" checked=enabled on_change=move |v| set_enabled.set(v) />
            <FieldToggle label="Approval required" checked=approval on_change=move |v| set_approval.set(v) />

            // Checkbox fields
            <Checkbox checked=self_exclusion on_change=move |v| set_self_exclusion.set(v)
                      label="Exclude target's own prior turns" />
            <Checkbox checked=include_user on_change=move |v| set_include_user.set(v)
                      label="Include user turns" />
            <Checkbox checked=include_system on_change=move |v| set_include_system.set(v)
                      label="Include system notes" />
            <Checkbox checked=include_summaries on_change=move |v| set_include_summaries.set(v)
                      label="Include pinned summaries" />

            <label class="flex flex-col gap-2 mt-2">
                <span class="type-caption-strong text-secondary">"Catch-up rule"</span>
                <select
                    class="type-body text-primary surface-sunken border rounded-md p-3"
                    aria-label="Catch-up rule"
                    prop:value=move || catch_up.get()
                    on:change=move |event| set_catch_up.set(event_target_value(&event))
                >
                    <option value="full_history">"Full history once"</option>
                    <option value="selected_range" disabled=true>"Selected range (set outside this form)"</option>
                    <option value="last_n">"Last N messages"</option>
                    <option value="pinned_summary">"Pinned summary"</option>
                    <option value="none">"No initial catch-up"</option>
                </select>
            </label>
            {move || (catch_up.get() == "last_n").then(|| view! {
                <label class="flex flex-col gap-2">
                    <span class="type-caption text-secondary">"Messages to include"</span>
                    <input
                        type="number"
                        min="1"
                        class="type-body text-primary surface-sunken border rounded-md p-3"
                        prop:value=move || last_n.get().to_string()
                        on:change=move |event| {
                            set_last_n.set(event_target_value(&event).parse().unwrap_or(10));
                        }
                    />
                </label>
            })}
            {move || (catch_up.get() == "pinned_summary").then(|| {
                let options = summaries.get();
                if options.is_empty() {
                    view! {
                        <p class="type-caption text-secondary">
                            "No pinned summaries yet. Create one from Pinned Summaries, then return here."
                        </p>
                    }.into_any()
                } else {
                    view! {
                        <label class="flex flex-col gap-2">
                            <span class="type-caption text-secondary">"Select summary"</span>
                            <select
                                class="type-body text-primary surface-sunken border rounded-md p-3"
                                aria-label="Pinned summary"
                                prop:value=move || summary_id.get().map(|id| id.0.to_string()).unwrap_or_default()
                                on:change=move |event| {
                                    let selected = event_target_value(&event);
                                    set_summary_id.set(
                                        summaries.get_untracked().into_iter()
                                            .find(|(id, _)| id.0.to_string() == selected)
                                            .map(|(id, _)| id)
                                    );
                                }
                            >
                                <option value="">"Choose a summary"</option>
                                {options.into_iter().map(|(id, name)| view! {
                                    <option value=id.0.to_string()>{name}</option>
                                }).collect_view()}
                            </select>
                        </label>
                    }.into_any()
                }
            })}
            <label class="flex flex-col gap-2">
                <span class="type-caption-strong text-secondary">"Incremental rule"</span>
                <select
                    class="type-body text-primary surface-sunken border rounded-md p-3"
                    aria-label="Incremental rule"
                    prop:value=move || incremental.get()
                    on:change=move |event| set_incremental.set(event_target_value(&event))
                >
                    <option value="unseen_delta">"Unseen delta only"</option>
                    <option value="sliding_window" disabled=true>"Sliding window (set outside this form)"</option>
                    <option value="last_response">"Last response only"</option>
                    <option value="full_history">"Full history every time"</option>
                    <option value="manual">"Manual only"</option>
                </select>
            </label>
            <div class="flex justify-end pt-2">
                <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                    let mut updated = edge_for_save.clone();
                    updated.enabled = enabled.get_untracked();
                    updated.self_exclusion = self_exclusion.get_untracked();
                    updated.include_user_turns = include_user.get_untracked();
                    updated.include_system_notes = include_system.get_untracked();
                    updated.include_pinned_summaries = include_summaries.get_untracked();
                    updated.approval_mode = if approval.get_untracked() {
                        ApprovalMode::RequireUserConfirmation
                    } else {
                        ApprovalMode::AutoSend
                    };
                    updated.catch_up_policy = match catch_up.get_untracked().as_str() {
                        "last_n" => CatchUpPolicy::LastN { count: last_n.get_untracked().max(1) },
                        "pinned_summary" => CatchUpPolicy::PinnedSummary {
                            summary_message_id: summary_id.get_untracked(),
                        },
                        "none" => CatchUpPolicy::None,
                        "full_history" => CatchUpPolicy::FullHistory,
                        // Anything this form cannot construct is carried through
                        // untouched. Falling back to a default here silently
                        // rewrote the operator's routing policy on any save.
                        _ => updated.catch_up_policy.clone(),
                    };
                    updated.incremental_policy = match incremental.get_untracked().as_str() {
                        "last_response" => IncrementalPolicy::LastResponseOnly,
                        "full_history" => IncrementalPolicy::FullHistoryEveryTime,
                        "manual" => IncrementalPolicy::ManualOnly,
                        "unseen_delta" => IncrementalPolicy::UnseenDeltaOnly,
                        _ => updated.incremental_policy.clone(),
                    };
                    on_save(updated);
                })>
                    "Save edge policy"
                </Button>
            </div>
        </div>
    }
}

fn catch_up_key(policy: &CatchUpPolicy) -> &'static str {
    match policy {
        CatchUpPolicy::FullHistory => "full_history",
        CatchUpPolicy::SelectedRange { .. } => "selected_range",
        CatchUpPolicy::LastN { .. } => "last_n",
        CatchUpPolicy::PinnedSummary { .. } => "pinned_summary",
        CatchUpPolicy::None => "none",
    }
}

fn incremental_key(policy: &IncrementalPolicy) -> &'static str {
    match policy {
        IncrementalPolicy::UnseenDeltaOnly => "unseen_delta",
        IncrementalPolicy::SlidingWindow { .. } => "sliding_window",
        IncrementalPolicy::LastResponseOnly => "last_response",
        IncrementalPolicy::FullHistoryEveryTime => "full_history",
        IncrementalPolicy::ManualOnly => "manual",
    }
}

/// Field row with toggle switch.
#[component]
fn FieldToggle(
    label: &'static str,
    checked: ReadSignal<bool>,
    on_change: impl Fn(bool) + 'static,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between">
            <span class="type-body text-primary">{label}</span>
            <Toggle checked=checked on_change=on_change />
        </div>
    }
}
