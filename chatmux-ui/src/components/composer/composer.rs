//! Main composer component (§3.5).
//!
//! Fixed at the bottom of the active workspace view.
//! Contains: target selector, input area, mode selector,
//! send button, package preview toggle, context pick button.

use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

use super::availability::{can_submit, reconcile_selected_targets};
use super::mode_selector::{ComposerMode, ModeSelector};
use super::package_preview::PackagePreview;
use super::target_selector::{Target, TargetSelector};
use crate::components::primitives::badge::{Badge, BadgeVariant};
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::checkbox::Checkbox;
use crate::components::primitives::icon::{Icon, IconKind};
use crate::components::primitives::text_area::TextArea;
use crate::components::primitives::text_input::TextInput;
use crate::components::provider::Provider;
use crate::models::{MessageId, NextRoundPackage, ProviderId, UiEvent, WorkspaceId};

#[derive(Clone, Debug)]
pub struct ComposerSubmission {
    pub text: String,
    pub mode: ComposerMode,
    pub targets: Vec<Provider>,
    pub selected_message_ids: BTreeSet<MessageId>,
    pub pinned_note: Option<String>,
    pub target_notes: BTreeMap<ProviderId, String>,
    pub include_target_prior_turns: bool,
    pub payload_overrides: BTreeMap<ProviderId, String>,
    pub parent_message_id: Option<MessageId>,
}

/// Main composer component.
#[component]
pub fn Composer(
    /// Active workspace receiving the manual package.
    workspace_id: WorkspaceId,
    /// Available targets.
    targets: Signal<Vec<Target>>,
    /// Persisted draft owned by the active workspace screen.
    draft: ReadSignal<String>,
    /// Update the persisted draft.
    set_draft: WriteSignal<String>,
    /// Whether a composer command is waiting for bridge confirmation.
    submitting: ReadSignal<bool>,
    /// Whether the global automation kill switch is active.
    kill_switch_active: ReadSignal<bool>,
    /// Whether the message log is selecting exact context.
    context_selection_mode: ReadSignal<bool>,
    /// Update context-selection mode.
    set_context_selection_mode: WriteSignal<bool>,
    /// Exact message identifiers picked for context.
    selected_context_ids: ReadSignal<BTreeSet<MessageId>>,
    /// Update exact context after package-block editing.
    set_selected_context_ids: WriteSignal<BTreeSet<MessageId>>,
    /// Message selected as the parent for the next send.
    #[prop(into)]
    branch_parent_id: Signal<Option<MessageId>>,
    /// Called when the branch parent should be cleared.
    on_clear_branch_parent: impl Fn() + 'static + Clone + Send,
    /// Called when the user sends a message.
    on_send: impl Fn(ComposerSubmission) + 'static + Clone + Send,
) -> impl IntoView {
    // Names the branch source the way the operator saw it in the log: who said
    // it and what it opened with. An eight-character id fragment identifies the
    // message to the storage layer and to nobody else.
    let branch_source_label = Signal::derive(move || {
        let Some(parent_id) = branch_parent_id.get() else {
            return String::new();
        };
        let messages = expect_context::<crate::state::message_state::MessageState>().messages;
        messages
            .get()
            .iter()
            .find(|message| message.id == parent_id)
            .map(|message| {
                let who = Provider::from_provider_id(message.participant_id).label();
                let snippet: String = message
                    .body_text
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                let snippet = if snippet.chars().count() > 60 {
                    format!("{}…", snippet.chars().take(60).collect::<String>())
                } else {
                    snippet
                };
                if snippet.is_empty() {
                    who.to_owned()
                } else {
                    format!("{who} · {snippet}")
                }
            })
            // The parent can fall outside the loaded window on a long
            // transcript; the id is the honest fallback, not the default.
            .unwrap_or_else(|| short_message_id(parent_id))
    });

    let (mode, set_mode) = signal(ComposerMode::Send);
    let (selected_targets, set_selected_targets) = signal(Vec::<Provider>::new());
    let (targets_initialized, set_targets_initialized) = signal(false);
    let (show_preview, set_show_preview) = signal(false);
    let (show_notes, set_show_notes) = signal(false);
    let (pinned_note, set_pinned_note) = signal(String::new());
    let (include_target_prior_turns, set_include_target_prior_turns) = signal(false);
    let (preview_packages, set_preview_packages) = signal(Vec::<NextRoundPackage>::new());
    let (preview_loading, set_preview_loading) = signal(false);
    let (preview_error, set_preview_error) = signal(None::<String>);
    let (preview_stale, set_preview_stale) = signal(false);
    let (edited_targets, set_edited_targets) = signal(BTreeSet::<ProviderId>::new());
    let (payload_overrides, set_payload_overrides) = signal(BTreeMap::<ProviderId, String>::new());
    let target_note_signals = StoredValue::new(BTreeMap::from([
        (ProviderId::Gpt, RwSignal::new(String::new())),
        (ProviderId::Gemini, RwSignal::new(String::new())),
        (ProviderId::Grok, RwSignal::new(String::new())),
        (ProviderId::Claude, RwSignal::new(String::new())),
    ]));
    let (context_dependency_snapshot, set_context_dependency_snapshot) =
        signal((BTreeSet::<MessageId>::new(), None::<MessageId>));

    Effect::new(move |_| {
        let current_targets = targets.get();
        let initialize = !targets_initialized.get_untracked();
        let reconciled = reconcile_selected_targets(
            &selected_targets.get_untracked(),
            &current_targets,
            initialize,
        );
        if reconciled != selected_targets.get_untracked() {
            set_selected_targets.set(reconciled);
            if show_preview.get_untracked() {
                set_preview_stale.set(true);
            }
        }
        if initialize {
            set_targets_initialized.set(true);
        }
    });

    Effect::new(move |_| {
        let current = (selected_context_ids.get(), branch_parent_id.get());
        if current != context_dependency_snapshot.get_untracked() {
            if show_preview.get_untracked() && !preview_loading.get_untracked() {
                set_preview_stale.set(true);
            }
            set_context_dependency_snapshot.set(current);
        }
    });

    let can_send = Signal::derive(move || {
        can_submit(
            &draft.get(),
            &selected_targets.get(),
            &targets.get(),
            kill_switch_active.get(),
            submitting.get(),
        ) && (mode.get() != ComposerMode::CopyOnly || selected_targets.get().len() == 1)
            && (!show_preview.get()
                || (!preview_loading.get()
                    && !preview_stale.get()
                    && !preview_packages.get().is_empty()))
    });
    let on_send_keydown = on_send.clone();
    let on_send_click = on_send.clone();
    let on_clear_branch_parent_button = on_clear_branch_parent.clone();

    let toggle_target = move |provider: Provider| {
        set_selected_targets.update(|targets| {
            if let Some(pos) = targets.iter().position(|&p| p == provider) {
                targets.remove(pos);
            } else {
                targets.push(provider);
            }
        });
        if show_preview.get_untracked() {
            set_preview_stale.set(true);
        }
    };

    let request_preview = StoredValue::new(move || {
        let target_ids = selected_targets
            .get_untracked()
            .into_iter()
            .map(Provider::to_provider_id)
            .collect::<Vec<_>>();
        let pinned = pinned_note.get_untracked();
        let pinned = (!pinned.trim().is_empty()).then_some(pinned);
        let target_notes = target_note_signals.with_value(|signals| {
            signals
                .iter()
                .filter_map(|(provider, note)| {
                    let value = note.get_untracked();
                    (!value.trim().is_empty()).then_some((*provider, value))
                })
                .collect::<BTreeMap<_, _>>()
        });
        set_preview_loading.set(true);
        set_preview_error.set(None);
        set_preview_stale.set(false);
        leptos::task::spawn_local_scoped_with_cancellation(async move {
            let result = crate::bridge::messaging::preview_manual_message(
                workspace_id,
                target_ids,
                draft.get_untracked(),
                selected_context_ids.get_untracked(),
                pinned,
                target_notes,
                include_target_prior_turns.get_untracked(),
                branch_parent_id.get_untracked(),
            )
            .await;
            match result {
                Ok(events) => {
                    let packages = events.into_iter().find_map(|event| match event {
                        UiEvent::ManualMessagePreview { packages } => Some(packages),
                        _ => None,
                    });
                    if let Some(packages) = packages {
                        set_payload_overrides.set(
                            packages
                                .iter()
                                .map(|package| {
                                    (
                                        package.target_participant_id,
                                        package.rendered_payload.clone(),
                                    )
                                })
                                .collect(),
                        );
                        set_edited_targets.set(BTreeSet::new());
                        set_preview_packages.set(packages);
                    } else {
                        set_preview_packages.set(Vec::new());
                        set_preview_error.set(Some(
                            "The background did not return a rendered package. Retry the preview."
                                .to_owned(),
                        ));
                    }
                }
                Err(detail) => {
                    set_preview_packages.set(Vec::new());
                    set_preview_error.set(Some(detail));
                }
            }
            set_preview_loading.set(false);
        });
    });

    view! {
        <div
            class="composer surface-raised"
            style="border-top: 1px solid var(--border-subtle); \
                   padding: var(--space-4) var(--space-5);"
        >
            // Target selector row
            <div class="mb-3">
                <TargetSelector
                    targets=targets
                    selected=selected_targets
                    on_toggle=toggle_target
                />
            </div>

            {move || show_notes.get().then(|| view! {
                <div class="surface-card flex flex-col gap-4 p-4 mb-3">
                    <div class="flex items-center justify-between gap-3">
                        <div class="flex flex-col gap-1">
                            <span class="type-subtitle text-primary">"Package notes"</span>
                            <span class="type-caption text-secondary">
                                "Pinned instructions go to every target; target notes stay private to that provider."
                            </span>
                        </div>
                        <Button
                            variant=ButtonVariant::Icon
                            size=ButtonSize::Small
                            aria_label="Close package notes".to_owned()
                            on_click=Box::new(move |_| set_show_notes.set(false))
                        >
                            <Icon kind=IconKind::Close size=14 />
                        </Button>
                    </div>
                    <label class="flex flex-col gap-2">
                        <span class="type-label text-secondary">"PINNED NOTE"</span>
                        <TextArea
                            value=pinned_note
                            min_rows=2
                            max_rows=4
                            placeholder="Included in every target package…"
                            aria_label="Pinned package note".to_owned()
                            on_input=move |value| {
                                set_pinned_note.set(value);
                                if show_preview.get_untracked() {
                                    set_preview_stale.set(true);
                                }
                            }
                        />
                    </label>
                    {move || target_note_signals.with_value(|signals| {
                        selected_targets
                            .get()
                            .into_iter()
                            .filter_map(|provider| {
                                let provider_id = provider.to_provider_id();
                                let note = signals.get(&provider_id).copied()?;
                                Some(view! {
                                    <label class="flex flex-col gap-2">
                                        <span class="type-label text-secondary">
                                            {format!("{} NOTE", provider.label().to_uppercase())}
                                        </span>
                                        <TextInput
                                            value=note.read_only()
                                            placeholder="Only included for this provider…"
                                            aria_label=format!("Note for {}", provider.label())
                                            on_input=move |value| {
                                                note.set(value);
                                                if show_preview.get_untracked() {
                                                    set_preview_stale.set(true);
                                                }
                                            }
                                        />
                                    </label>
                                })
                            })
                            .collect_view()
                    })}
                    <Checkbox
                        checked=Signal::derive(move || include_target_prior_turns.get())
                        label="Include each target’s own prior assistant turns".to_owned()
                        on_change=move |checked| {
                            set_include_target_prior_turns.set(checked);
                            if show_preview.get_untracked() {
                                set_preview_stale.set(true);
                            }
                        }
                    />
                </div>
            })}

            // Package preview (toggle)
            {move || show_preview.get().then(|| view! {
                <div class="mb-3">
                    <PackagePreview
                        packages=preview_packages
                        loading=preview_loading
                        error=preview_error
                        stale=preview_stale
                        edited_targets=edited_targets
                        on_edit=move |target, value| {
                            set_preview_packages.update(|packages| {
                                if let Some(package) = packages
                                    .iter_mut()
                                    .find(|package| package.target_participant_id == target)
                                {
                                    package.character_count = value.chars().count();
                                    package.rendered_payload.clone_from(&value);
                                }
                            });
                            set_payload_overrides.update(|overrides| {
                                overrides.insert(target, value);
                            });
                            set_edited_targets.update(|targets| {
                                targets.insert(target);
                            });
                        }
                        on_remove_source=move |message_id| {
                            let mut remaining = preview_packages
                                .get_untracked()
                                .into_iter()
                                .flat_map(|package| {
                                    package.source_blocks.into_iter().map(|block| block.message_id)
                                })
                                .collect::<BTreeSet<_>>();
                            remaining.remove(&message_id);
                            set_selected_context_ids.set(remaining);
                            set_payload_overrides.set(BTreeMap::new());
                            set_edited_targets.set(BTreeSet::new());
                            set_preview_stale.set(true);
                            request_preview.with_value(|request| request());
                        }
                        on_close=move || set_show_preview.set(false)
                        on_refresh=move || {
                            request_preview.with_value(|request| request());
                        }
                    />
                </div>
            })}

            {move || branch_parent_id.get().map(|_parent_id| view! {
                <div class="flex items-center gap-2 mb-3 p-2 surface-sunken rounded-md">
                    <Badge variant=BadgeVariant::Info>
                        <Icon kind=IconKind::GitBranch size=12 />
                        "Branch source"
                    </Badge>
                    <span class="type-caption text-secondary truncate" title=move || branch_source_label.get()>
                        {move || branch_source_label.get()}
                    </span>
                    <span class="flex-1"></span>
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        title="Clear branch source".to_string()
                        aria_label="Clear branch source".to_string()
                        on_click=Box::new({
                            let on_clear = on_clear_branch_parent_button.clone();
                            move |_| on_clear()
                        })
                    >
                        <Icon kind=IconKind::Close size=14 />
                    </Button>
                </div>
            })}

            // Input area
            <textarea
                class="type-body w-full mb-3"
                style="\
                    min-height: 52px; max-height: 208px; \
                    padding: var(--space-4); \
                    background: var(--surface-sunken); \
                    border: 1px solid var(--border-default); \
                    border-radius: var(--radius-md); \
                    color: var(--text-primary); \
                    resize: none; overflow-y: auto;"
                placeholder="Type a message…"
                prop:value=move || draft.get()
                on:input=move |ev| {
                    set_draft.set(event_target_value(&ev));
                    if show_preview.get_untracked() {
                        set_preview_stale.set(true);
                    }
                }
                on:keydown=move |ev| {
                    // Ctrl+Enter or Cmd+Enter to send
                    if ev.key() == "Enter"
                        && (ev.ctrl_key() || ev.meta_key())
                        && can_send.get_untracked()
                    {
                        let submission = ComposerSubmission {
                            text: draft.get_untracked(),
                            mode: mode.get_untracked(),
                            targets: selected_targets.get_untracked(),
                            selected_message_ids: selected_context_ids.get_untracked(),
                            pinned_note: non_empty_note(pinned_note.get_untracked()),
                            target_notes: current_target_notes(target_note_signals),
                            include_target_prior_turns: include_target_prior_turns.get_untracked(),
                            payload_overrides: if show_preview.get_untracked()
                                && !preview_stale.get_untracked()
                            {
                                payload_overrides.get_untracked()
                            } else {
                                BTreeMap::new()
                            },
                            parent_message_id: branch_parent_id.get_untracked(),
                        };
                        ev.prevent_default();
                        on_send_keydown(submission);
                    }
                }
            />

            // Action row: mode selector (left) + send button (right)
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <ModeSelector mode=mode on_change=move |m| set_mode.set(m) />
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        title="Pick messages for context".to_owned()
                        aria_label="Pick messages for context".to_owned()
                        aria_pressed=context_selection_mode
                        on_click=Box::new(move |_| {
                            set_context_selection_mode
                                .set(!context_selection_mode.get_untracked());
                        })
                    >
                        <Icon kind=IconKind::Crosshair size=16 />
                        {move || (!selected_context_ids.get().is_empty()).then(|| view! {
                            <Badge variant=BadgeVariant::Accent>
                                {selected_context_ids.get().len().to_string()}
                            </Badge>
                        })}
                    </Button>
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        title="Edit pinned and per-target notes".to_owned()
                        aria_label="Edit package notes".to_owned()
                        aria_pressed=show_notes
                        on_click=Box::new(move |_| set_show_notes.update(|shown| *shown = !*shown))
                    >
                        <Icon kind=IconKind::Pin size=16 />
                    </Button>
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        title="Preview exact outbound packages".to_owned()
                        aria_label="Preview exact outbound packages".to_owned()
                        aria_pressed=show_preview
                        disabled=Signal::derive(move || {
                            draft.get().trim().is_empty() || selected_targets.get().is_empty()
                        })
                        on_click=Box::new(move |_| {
                            if show_preview.get_untracked() {
                                set_show_preview.set(false);
                            } else {
                                set_show_preview.set(true);
                                request_preview.with_value(|request| request());
                            }
                        })
                    >
                        <Icon kind=IconKind::Eye size=16 />
                    </Button>
                </div>

                <Button
                    variant=ButtonVariant::Primary
                    disabled=Signal::derive(move || !can_send.get())
                    loading=submitting
                    title="Requires text and at least one selected, ready provider".to_owned()
                    on_click=Box::new(move |_| {
                        if can_send.get_untracked() {
                            let submission = ComposerSubmission {
                                text: draft.get_untracked(),
                                mode: mode.get_untracked(),
                                targets: selected_targets.get_untracked(),
                                selected_message_ids: selected_context_ids.get_untracked(),
                                pinned_note: non_empty_note(pinned_note.get_untracked()),
                                target_notes: current_target_notes(target_note_signals),
                                include_target_prior_turns: include_target_prior_turns
                                    .get_untracked(),
                                payload_overrides: if show_preview.get_untracked()
                                    && !preview_stale.get_untracked()
                                {
                                    payload_overrides.get_untracked()
                                } else {
                                    BTreeMap::new()
                                },
                                parent_message_id: branch_parent_id.get_untracked(),
                            };
                            on_send_click(submission);
                        }
                    })
                >
                    {move || if submitting.get() { "Working…" } else { mode.get().label() }}
                </Button>
            </div>
            {move || kill_switch_active.get().then(|| view! {
                <p class="type-caption text-error mt-2" role="alert">
                    "Kill switch active — sending is disabled."
                </p>
            })}
            {move || (mode.get() == ComposerMode::CopyOnly && selected_targets.get().len() > 1).then(|| view! {
                <p class="type-caption text-secondary mt-2" role="status">
                    "Copy mode requires exactly one target so the clipboard contains one exact rendered payload."
                </p>
            })}
        </div>
    }
}

fn short_message_id(message_id: MessageId) -> String {
    message_id.0.to_string().chars().take(8).collect()
}

fn non_empty_note(note: String) -> Option<String> {
    (!note.trim().is_empty()).then_some(note)
}

fn current_target_notes(
    signals: StoredValue<BTreeMap<ProviderId, RwSignal<String>>>,
) -> BTreeMap<ProviderId, String> {
    signals.with_value(|signals| {
        signals
            .iter()
            .filter_map(|(provider, note)| {
                let value = note.get_untracked();
                (!value.trim().is_empty()).then_some((*provider, value))
            })
            .collect()
    })
}
