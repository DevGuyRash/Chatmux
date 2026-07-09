//! Main composer component (§3.5).
//!
//! Fixed at the bottom of the active workspace view.
//! Contains: target selector, input area, mode selector,
//! send button, package preview toggle, context pick button.

use leptos::prelude::*;

use super::mode_selector::{ComposerMode, ModeSelector};
use super::target_selector::{Target, TargetSelector};
use crate::components::primitives::badge::{Badge, BadgeVariant};
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::icon::{Icon, IconKind};
use crate::components::provider::Provider;
use crate::models::MessageId;

#[derive(Clone, Debug)]
pub struct ComposerSubmission {
    pub text: String,
    pub mode: ComposerMode,
    pub targets: Vec<Provider>,
    pub parent_message_id: Option<MessageId>,
}

/// Main composer component.
#[component]
pub fn Composer(
    /// Available targets.
    targets: Signal<Vec<Target>>,
    /// Message selected as the parent for the next send.
    #[prop(into)]
    branch_parent_id: Signal<Option<MessageId>>,
    /// Called when the branch parent should be cleared.
    on_clear_branch_parent: impl Fn() + 'static + Clone + Send,
    /// Called when the user sends a message.
    on_send: impl Fn(ComposerSubmission) + 'static + Clone + Send,
) -> impl IntoView {
    let (text, set_text) = signal(String::new());
    let (mode, set_mode) = signal(ComposerMode::Send);
    let (selected_targets, set_selected_targets) = signal(vec![
        Provider::Gpt,
        Provider::Gemini,
        Provider::Grok,
        Provider::Claude,
    ]);
    let (show_preview, _set_show_preview) = signal(false);

    let can_send =
        Signal::derive(move || !text.get().trim().is_empty() && !selected_targets.get().is_empty());
    let on_send_keydown = on_send.clone();
    let on_send_click = on_send.clone();
    let on_clear_branch_parent_keydown = on_clear_branch_parent.clone();
    let on_clear_branch_parent_click = on_clear_branch_parent.clone();
    let on_clear_branch_parent_button = on_clear_branch_parent.clone();

    let toggle_target = move |provider: Provider| {
        set_selected_targets.update(|targets| {
            if let Some(pos) = targets.iter().position(|&p| p == provider) {
                targets.remove(pos);
            } else {
                targets.push(provider);
            }
        });
    };

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

            // Package preview (toggle)
            {move || show_preview.get().then(|| view! {
                <div class="type-caption text-secondary p-4 surface-sunken rounded-md mb-3">
                    "Package preview placeholder"
                </div>
            })}

            {move || branch_parent_id.get().map(|parent_id| view! {
                <div class="flex items-center gap-2 mb-3 p-2 surface-sunken rounded-md">
                    <Badge variant=BadgeVariant::Info>
                        <Icon kind=IconKind::GitBranch size=12 />
                        "Branch source"
                    </Badge>
                    <span class="type-caption text-secondary truncate">
                        {short_message_id(parent_id)}
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
                prop:value=move || text.get()
                on:input=move |ev| set_text.set(event_target_value(&ev))
                on:keydown=move |ev| {
                    // Ctrl+Enter or Cmd+Enter to send
                    if ev.key() == "Enter" && (ev.ctrl_key() || ev.meta_key()) {
                        if can_send.get_untracked() {
                            let submission = ComposerSubmission {
                                text: text.get_untracked(),
                                mode: mode.get_untracked(),
                                targets: selected_targets.get_untracked(),
                                parent_message_id: branch_parent_id.get_untracked(),
                            };
                            set_text.set(String::new());
                            on_clear_branch_parent_keydown();
                            on_send_keydown(submission);
                        }
                    }
                }
            />

            // Action row: mode selector (left) + send button (right)
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <ModeSelector mode=mode on_change=move |m| set_mode.set(m) />
                </div>

                <Button
                    variant=ButtonVariant::Primary
                    disabled=false
                    on_click=Box::new(move |_| {
                        if can_send.get_untracked() {
                            let submission = ComposerSubmission {
                                text: text.get_untracked(),
                                mode: mode.get_untracked(),
                                targets: selected_targets.get_untracked(),
                                parent_message_id: branch_parent_id.get_untracked(),
                            };
                            set_text.set(String::new());
                            on_clear_branch_parent_click();
                            on_send_click(submission);
                        }
                    })
                >
                    {move || mode.get().label()}
                </Button>
            </div>
        </div>
    }
}

fn short_message_id(message_id: MessageId) -> String {
    message_id.0.to_string().chars().take(8).collect()
}
