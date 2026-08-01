//! Between-round package review (§3.23).

use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::modal::Modal;
use crate::models::{NextRoundPackage, ProviderId};

#[component]
pub fn BetweenRoundsReview(
    open: ReadSignal<bool>,
    round_number: ReadSignal<u32>,
    packages: ReadSignal<Vec<NextRoundPackage>>,
    on_close: impl Fn() + 'static + Copy + Send,
    on_resume: impl Fn(BTreeMap<ProviderId, String>, BTreeSet<ProviderId>, Option<String>)
    + 'static
    + Copy
    + Send,
) -> impl IntoView {
    let (payloads, set_payloads) = signal(BTreeMap::<ProviderId, String>::new());
    let (originals, set_originals) = signal(BTreeMap::<ProviderId, String>::new());
    let (skipped, set_skipped) = signal(BTreeSet::<ProviderId>::new());
    let (user_message, set_user_message) = signal(String::new());

    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        let values = packages
            .get()
            .into_iter()
            .map(|package| (package.target_participant_id, package.rendered_payload))
            .collect::<BTreeMap<_, _>>();
        set_originals.set(values.clone());
        set_payloads.set(values);
        set_skipped.set(BTreeSet::new());
        set_user_message.set(String::new());
    });

    view! {
        <Modal open=open on_close=on_close max_width=780 aria_label="Review next round".to_owned()>
            <div class="flex flex-col gap-6" style="max-height: 82vh; overflow-y: auto;">
                <div>
                    <h2 class="type-title text-primary">"Review next round"</h2>
                    <p class="type-caption text-secondary mt-1">
                        {move || format!("Round {} preview · edits apply once and are preserved in the dispatch ledger.", round_number.get())}
                    </p>
                </div>

                <Show
                    when=move || !packages.get().is_empty()
                    fallback=move || view! {
                        <div class="surface-sunken border rounded-md p-5">
                            <p class="type-body text-secondary">
                                "No eligible messages are available for the next round. Add a user message below or stay paused."
                            </p>
                        </div>
                    }
                >
                    <section class="flex flex-col gap-4">
                        <h3 class="type-subtitle text-primary">"Per-target packages"</h3>
                        {move || packages.get().into_iter().map(|package| {
                            let provider = package.target_participant_id;
                            let provider_name = provider.display_name();
                            let source_count = package.source_message_ids.len();
                            view! {
                                <article class="surface-card border rounded-md p-4 flex flex-col gap-3">
                                    <div class="flex items-center justify-between gap-4">
                                        <div>
                                            <h4 class="type-body-strong text-primary">{provider_name}</h4>
                                            <p class="type-caption text-secondary">
                                                {format!("{source_count} source message{}", if source_count == 1 { "" } else { "s" })}
                                            </p>
                                        </div>
                                        <label class="flex items-center gap-2 type-caption text-secondary">
                                            <input
                                                type="checkbox"
                                                checked=move || skipped.get().contains(&provider)
                                                on:change=move |event| set_skipped.update(|values| {
                                                    if event_target_checked(&event) {
                                                        values.insert(provider);
                                                    } else {
                                                        values.remove(&provider);
                                                    }
                                                })
                                            />
                                            "Skip this round"
                                        </label>
                                    </div>
                                    <textarea
                                        aria-label=format!("Package for {provider_name}")
                                        class="type-code text-primary surface-sunken border rounded-md"
                                        style="padding: var(--space-4); min-height: 150px; resize: vertical;"
                                        disabled=move || skipped.get().contains(&provider)
                                        prop:value=move || payloads.get().get(&provider).cloned().unwrap_or_default()
                                        on:input=move |event| {
                                            let value = event_target_value(&event);
                                            set_payloads.update(|values| {
                                                values.insert(provider, value);
                                            });
                                        }
                                    />
                                    <p class="type-caption text-tertiary">
                                        {move || {
                                            let value = payloads.get().get(&provider).cloned().unwrap_or_default();
                                            let edited = originals.get().get(&provider).is_some_and(|original| original != &value);
                                            format!("{} characters{}", value.chars().count(), if edited { " · Edited" } else { "" })
                                        }}
                                    </p>
                                </article>
                            }
                        }).collect_view()}
                    </section>
                </Show>

                <section class="flex flex-col gap-2">
                    <label class="type-label text-secondary" for="review-user-message">
                        "Add a user message before the next round"
                    </label>
                    <textarea
                        id="review-user-message"
                        class="type-body text-primary surface-sunken border rounded-md"
                        style="padding: var(--space-3) var(--space-4); min-height: 76px; resize: vertical;"
                        placeholder="Optional instruction for all active targets…"
                        prop:value=move || user_message.get()
                        on:input=move |event| set_user_message.set(event_target_value(&event))
                    />
                </section>

                <div class="flex justify-end gap-3 border-t pt-4">
                    <Button variant=ButtonVariant::Ghost on_click=Box::new(move |_| on_close())>
                        "Stay paused"
                    </Button>
                    <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                        let original = originals.get_untracked();
                        let overrides = payloads
                            .get_untracked()
                            .into_iter()
                            .filter(|(provider, value)| original.get(provider) != Some(value))
                            .collect::<BTreeMap<_, _>>();
                        let message = user_message.get_untracked().trim().to_owned();
                        on_resume(
                            overrides,
                            skipped.get_untracked(),
                            (!message.is_empty()).then_some(message),
                        );
                    })>
                        "Resume with changes"
                    </Button>
                </div>
            </div>
        </Modal>
    }
}
