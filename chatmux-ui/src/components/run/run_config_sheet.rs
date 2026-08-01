//! Run configuration sheet (§3.24).

use leptos::prelude::*;
use std::collections::BTreeSet;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::modal::Modal;
use crate::models::{
    BarrierPolicy, OrchestrationMode, ProviderId, RunConfiguration, StopPolicy, TimingPolicy,
};

#[component]
pub fn RunConfigSheet(
    open: ReadSignal<bool>,
    initial_mode: OrchestrationMode,
    available_participants: BTreeSet<ProviderId>,
    on_cancel: impl Fn() + 'static + Copy + Send,
    on_start: impl Fn(RunConfiguration) + 'static + Copy + Send,
) -> impl IntoView {
    let (selected_mode, set_selected_mode) = signal(initial_mode);
    let (participants, set_participants) = signal(available_participants.clone());
    let (moderator, set_moderator) = signal(None::<ProviderId>);
    let (barrier, set_barrier) = signal("all".to_owned());
    let (moderated, set_moderated) = signal(false);
    let (max_rounds, set_max_rounds) = signal(3u32);
    let (generation_timeout, set_generation_timeout) = signal(120u64);
    let (inter_round_delay, set_inter_round_delay) = signal(5u64);
    let (max_concurrent, set_max_concurrent) = signal(4usize);
    let (sentinel, set_sentinel) = signal(String::new());

    let can_start = Signal::derive(move || {
        !participants.get().is_empty()
            && max_rounds.get() > 0
            && generation_timeout.get() > 0
            && max_concurrent.get() > 0
            && (!matches!(selected_mode.get(), OrchestrationMode::ModeratorJury)
                || moderator.get().is_some())
    });

    view! {
        <Modal open=open on_close=on_cancel max_width=760>
            <div class="flex flex-col gap-7" style="max-height: 82vh; overflow-y: auto;">
                <div>
                    <h2 class="type-title text-primary">"Configure run"</h2>
                    <p class="type-caption text-secondary mt-1">
                        "The topology and limits below are persisted with the run ledger."
                    </p>
                </div>

                <section class="flex flex-col gap-3">
                    <h3 class="type-subtitle text-primary">"Orchestration mode"</h3>
                    <div class="grid grid-cols-2 gap-3">
                        {[
                            OrchestrationMode::Broadcast,
                            OrchestrationMode::Directed,
                            OrchestrationMode::RelayToMany,
                            OrchestrationMode::Roundtable,
                            OrchestrationMode::ModeratorJury,
                            OrchestrationMode::RelayChain,
                            OrchestrationMode::ModeratedAutonomous,
                        ].into_iter().map(|mode| view! {
                            <ModeCard mode=mode selected=selected_mode on_select=move |value| {
                                set_selected_mode.set(value);
                                if value == OrchestrationMode::ModeratedAutonomous {
                                    set_moderated.set(true);
                                }
                            } />
                        }).collect_view()}
                    </div>
                </section>

                <section class="flex flex-col gap-3">
                    <h3 class="type-subtitle text-primary">"Participants"</h3>
                    <div class="grid grid-cols-2 gap-3">
                        {available_participants.iter().copied().map(|provider| view! {
                            <label class="flex items-center gap-3 type-body text-primary surface-sunken border rounded-md p-3">
                                <input
                                    type="checkbox"
                                    checked=move || participants.get().contains(&provider)
                                    on:change=move |event| set_participants.update(|selected| {
                                        if event_target_checked(&event) {
                                            selected.insert(provider);
                                        } else {
                                            selected.remove(&provider);
                                            if moderator.get_untracked() == Some(provider) {
                                                set_moderator.set(None);
                                            }
                                        }
                                    })
                                />
                                <span>{provider.display_name()}</span>
                            </label>
                        }).collect_view()}
                    </div>
                </section>

                <Show when=move || matches!(selected_mode.get(), OrchestrationMode::Directed | OrchestrationMode::RelayToOne | OrchestrationMode::ModeratorJury)>
                    <section class="flex flex-col gap-3">
                        <label class="type-label text-secondary" for="run-moderator">
                            {move || if selected_mode.get() == OrchestrationMode::ModeratorJury { "Moderator" } else { "Target provider" }}
                        </label>
                        <select id="run-moderator"
                            class="type-body text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-3) var(--space-8) var(--space-3) var(--space-4);"
                            on:change=move |event| set_moderator.set(provider_from_key(&event_target_value(&event)))>
                            <option value="">"Select a provider"</option>
                            {available_participants.iter().copied().map(|provider| view! {
                                <option value=provider_key(provider)>{provider.display_name()}</option>
                            }).collect_view()}
                        </select>
                    </section>
                </Show>

                <section class="grid grid-cols-2 gap-5">
                    <div class="flex flex-col gap-3">
                        <label class="type-label text-secondary" for="run-barrier">"Round barrier"</label>
                        <select id="run-barrier"
                            class="type-body text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-3) var(--space-8) var(--space-3) var(--space-4);"
                            prop:value=move || barrier.get()
                            on:change=move |event| set_barrier.set(event_target_value(&event))>
                            // Names match `barrier_policy_label`, which is what
                            // the run controls bar shows once the run starts.
                            // An option the operator picks here must still be
                            // called the same thing while it is in effect.
                            <option value="all">"Wait for all"</option>
                            <option value="quorum">"Quorum"</option>
                            <option value="first">"First finisher"</option>
                            <option value="manual">"Manual advance"</option>
                        </select>
                    </div>
                    <label class="flex items-center gap-3 type-body text-primary self-end surface-sunken border rounded-md p-3">
                        <input type="checkbox" checked=move || moderated.get()
                            on:change=move |event| set_moderated.set(event_target_checked(&event)) />
                        "Pause for review between rounds"
                    </label>
                </section>

                <section class="flex flex-col gap-3">
                    <h3 class="type-subtitle text-primary">"Timing and stop limits"</h3>
                    <div class="grid grid-cols-2 gap-4">
                        <NumberField id="run-max-rounds" label="Maximum rounds" value=max_rounds set_value=set_max_rounds min=1 />
                        <NumberField id="run-timeout" label="Generation timeout (seconds)" value=generation_timeout set_value=set_generation_timeout min=1 />
                        <NumberField id="run-delay" label="Delay between rounds (seconds)" value=inter_round_delay set_value=set_inter_round_delay min=0 />
                        <NumberField id="run-concurrency" label="Maximum concurrent sends" value=max_concurrent set_value=set_max_concurrent min=1 />
                    </div>
                    <label class="type-label text-secondary" for="run-sentinel">"Optional stop phrase"</label>
                    <input id="run-sentinel"
                        class="type-body text-primary surface-sunken border rounded-md"
                        style="padding: var(--space-3) var(--space-4);"
                        placeholder="Stop when a response contains…"
                        prop:value=move || sentinel.get()
                        on:input=move |event| set_sentinel.set(event_target_value(&event)) />
                </section>

                <section class="surface-sunken border rounded-md p-4">
                    <h3 class="type-label text-secondary mb-2">"Routing summary"</h3>
                    <p class="type-body text-primary">
                        {move || routing_summary(selected_mode.get(), participants.get().len(), moderated.get())}
                    </p>
                </section>

                <div class="flex justify-end gap-3 border-t pt-4">
                    <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| on_cancel())>
                        "Cancel"
                    </Button>
                    <Button variant=ButtonVariant::Primary disabled=Signal::derive(move || !can_start.get())
                        on_click=Box::new(move |_| {
                            let participant_set = participants.get_untracked();
                            let barrier_policy = match barrier.get_untracked().as_str() {
                                "quorum" => BarrierPolicy::Quorum { providers: participant_set.clone() },
                                "first" => BarrierPolicy::FirstFinisher,
                                "manual" => BarrierPolicy::ManualAdvance,
                                _ => BarrierPolicy::WaitForAll,
                            };
                            let stop_on_sentinel_phrase = {
                                let value = sentinel.get_untracked().trim().to_owned();
                                (!value.is_empty()).then_some(value)
                            };
                            let stop_policy = StopPolicy {
                                stop_on_sentinel_phrase,
                                require_approval_between_rounds: moderated.get_untracked(),
                                ..StopPolicy::default()
                            };
                            on_start(RunConfiguration {
                                mode: selected_mode.get_untracked(),
                                participants: participant_set.clone(),
                                moderator: moderator.get_untracked(),
                                relay_order: participant_set.iter().copied().collect(),
                                barrier_policy,
                                timing_policy: TimingPolicy {
                                    per_provider_generation_timeout_secs: generation_timeout.get_untracked(),
                                    inter_round_delay_secs: inter_round_delay.get_untracked(),
                                    max_concurrent_sends: max_concurrent.get_untracked(),
                                    max_rounds: Some(max_rounds.get_untracked()),
                                    ..TimingPolicy::default()
                                },
                                stop_policy,
                                require_review_between_rounds: moderated.get_untracked(),
                            });
                        })>
                        "Start run"
                    </Button>
                </div>
            </div>
        </Modal>
    }
}

#[component]
fn ModeCard(
    mode: OrchestrationMode,
    selected: ReadSignal<OrchestrationMode>,
    on_select: impl Fn(OrchestrationMode) + 'static + Copy,
) -> impl IntoView {
    view! {
        <button type="button" aria-pressed=move || selected.get() == mode
            class="flex flex-col gap-1 w-full cursor-pointer select-none text-left transition-colors"
            style=move || format!(
                "padding: var(--space-4) var(--space-5); border-radius: var(--radius-md); border: 1px solid {}; background: {};",
                if selected.get() == mode { "var(--border-accent)" } else { "var(--border-default)" },
                if selected.get() == mode { "var(--surface-selected)" } else { "var(--surface-raised)" },
            )
            on:click=move |_| on_select(mode)>
            <span class="type-subtitle text-primary">{mode_label(mode)}</span>
            <span class="type-caption text-secondary">{mode_description(mode)}</span>
        </button>
    }
}

#[component]
fn NumberField<T>(
    id: &'static str,
    label: &'static str,
    value: ReadSignal<T>,
    set_value: WriteSignal<T>,
    min: T,
) -> impl IntoView
where
    T: Copy + ToString + std::str::FromStr + PartialOrd + Send + Sync + 'static,
{
    view! {
        <div class="flex flex-col gap-2">
            <label class="type-label text-secondary" for=id>{label}</label>
            <input id=id type="number" min=min.to_string()
                class="type-body text-primary surface-sunken border rounded-md"
                style="padding: var(--space-3) var(--space-4);"
                prop:value=move || value.get().to_string()
                on:input=move |event| {
                    if let Ok(parsed) = event_target_value(&event).parse::<T>()
                        && parsed >= min
                    {
                        set_value.set(parsed);
                    }
                } />
        </div>
    }
}

// One definition lives in models::view_models so every screen spells these the
// same way; this stays as a local alias for the call sites in this file.
use crate::models::view_models::orchestration_mode_label as mode_label;

fn mode_description(mode: OrchestrationMode) -> &'static str {
    match mode {
        OrchestrationMode::Broadcast => "Send one user seed to every participant.",
        OrchestrationMode::Directed => "Package selected context for one target.",
        OrchestrationMode::RelayToOne => "Forward eligible output to one target.",
        OrchestrationMode::RelayToMany => "Duplicate one rendered package across targets.",
        OrchestrationMode::DraftOnly => "Prepare packages without browser I/O.",
        OrchestrationMode::CopyOnly => "Prepare one package for clipboard use.",
        OrchestrationMode::Roundtable => "Full mesh with one barrier per round.",
        OrchestrationMode::ModeratorJury => {
            "Peers report to a moderator and receive its synthesis."
        }
        OrchestrationMode::RelayChain => "Move output through the participant order.",
        OrchestrationMode::ModeratedAutonomous => "Roundtable with mandatory review checkpoints.",
    }
}

fn routing_summary(mode: OrchestrationMode, participants: usize, moderated: bool) -> String {
    format!(
        "{} participant{} · {} · {}",
        participants,
        if participants == 1 { "" } else { "s" },
        mode_label(mode),
        if moderated {
            "review between rounds"
        } else {
            "automatic continuation"
        }
    )
}

fn provider_key(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::Gpt => "gpt",
        ProviderId::Gemini => "gemini",
        ProviderId::Grok => "grok",
        ProviderId::Claude => "claude",
        ProviderId::User => "user",
        ProviderId::System => "system",
    }
}

fn provider_from_key(value: &str) -> Option<ProviderId> {
    match value {
        "gpt" => Some(ProviderId::Gpt),
        "gemini" => Some(ProviderId::Gemini),
        "grok" => Some(ProviderId::Grok),
        "claude" => Some(ProviderId::Claude),
        _ => None,
    }
}
