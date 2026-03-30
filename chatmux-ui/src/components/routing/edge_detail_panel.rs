//! Edge detail panel (§3.9).
//!
//! Form for editing a single edge policy. Shows all fields:
//! Enabled, Catch-Up Rule, Incremental Rule, Self-Exclusion, etc.

use leptos::prelude::*;

use crate::components::primitives::toggle::Toggle;
use crate::components::primitives::checkbox::Checkbox;
use crate::components::provider::{Provider, provider_icon::ProviderIcon};
use crate::models::{EdgePolicy, ApprovalMode};

/// Edge detail editing panel.
#[component]
pub fn EdgeDetailPanel(
    /// The edge policy being edited.
    edge: EdgePolicy,
    /// Called when changes are saved.
    #[allow(unused)]
    on_save: impl Fn(EdgePolicy) + 'static + Copy + Send,
) -> impl IntoView {
    let (enabled, set_enabled) = signal(edge.enabled);
    let (self_exclusion, set_self_exclusion) = signal(edge.self_exclusion);
    let (include_user, set_include_user) = signal(edge.include_user_turns);
    let (include_system, set_include_system) = signal(edge.include_system_notes);
    let (include_summaries, set_include_summaries) = signal(edge.include_pinned_summaries);
    let (approval, set_approval) = signal(edge.approval_mode != ApprovalMode::AutoSend);

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

            // Catch-up and incremental policy dropdowns will use the Dropdown primitive
            // once fully wired with the EdgePolicy model's enum variants.
            <div class="type-caption text-tertiary mt-4">
                {format!("Catch-up: {:?}", edge.catch_up_policy)}
            </div>
            <div class="type-caption text-tertiary">
                {format!("Incremental: {:?}", edge.incremental_policy)}
            </div>
        </div>
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
