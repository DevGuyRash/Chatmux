//! Message log component (§3.3).
//!
//! Scrollable message list. Occupies all vertical space between
//! workspace header and composer. Handles round grouping, selection
//! mode, search highlighting, and new messages indicator.

use leptos::prelude::*;
use std::collections::BTreeSet;

use super::message_card::MessageCard;
use super::new_messages_indicator::NewMessagesIndicator;
use super::round_divider::RoundDivider;
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::models::{Message, MessageId};

/// Message log component.
#[component]
pub fn MessageLog(
    /// Messages to display.
    messages: Signal<Vec<Message>>,
    /// Count of new messages below viewport.
    new_below_count: ReadSignal<u32>,
    /// Message selected as the parent for the next composer send.
    branch_parent_id: ReadSignal<Option<MessageId>>,
    /// Whether the log is selecting exact context for the next package.
    context_selection_mode: ReadSignal<bool>,
    /// Message identifiers selected as exact context.
    selected_context_ids: ReadSignal<BTreeSet<MessageId>>,
    /// Called when a message card is clicked (for inspection).
    on_message_click: impl Fn(MessageId) + 'static + Copy + Send + Sync,
    /// Called when one message is toggled in context-selection mode.
    on_toggle_context: impl Fn(MessageId) + 'static + Copy + Send + Sync,
    /// Called when the current context selection is accepted.
    on_context_done: impl Fn() + 'static + Copy + Send,
    /// Called when the current context selection is cleared.
    on_context_clear: impl Fn() + 'static + Copy + Send,
    /// Called when a message should become the composer branch parent.
    on_branch_from_message: impl Fn(MessageId) + 'static + Copy + Send,
    /// Called to scroll to bottom.
    on_scroll_to_bottom: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <div
            class="message-log flex-1 overflow-y-auto relative"
            role="log"
            aria-label="Message log"
            aria-live="polite"
            style="padding: var(--space-4);"
        >
            {move || {
                let msgs = messages.get();
                if msgs.is_empty() {
                    view! {
                        <crate::components::primitives::empty_state::EmptyState
                            icon=crate::components::primitives::icon::IconKind::ChatBubble
                            heading="No messages"
                            description="Send a message or start a run to begin the conversation."
                        />
                    }.into_any()
                } else {
                    let mut last_round: Option<u32> = None;
                    view! {
                        <div class="flex flex-col" style="gap: var(--space-4);">
                            {msgs.into_iter().map(|msg| {
                                let msg_id = msg.id.0;
                                let message_id = MessageId(msg_id);
                                let current_round = msg.round;

                                // Show round divider if round changed
                                let show_divider = match (last_round, current_round) {
                                    (Some(prev), Some(curr)) if curr != prev => {
                                        last_round = Some(curr);
                                        Some(curr)
                                    }
                                    (None, Some(curr)) => {
                                        last_round = Some(curr);
                                        Some(curr)
                                    }
                                    _ => {
                                        if current_round.is_some() {
                                            last_round = current_round;
                                        }
                                        None
                                    }
                                };

                                view! {
                                    <>
                                        {show_divider.map(|r| view! { <RoundDivider round=r /> })}
                                        <MessageCard
                                            message=msg
                                            selection_mode=context_selection_mode.get()
                                            selected=if context_selection_mode.get() {
                                                selected_context_ids.get().contains(&message_id)
                                            } else {
                                                branch_parent_id.get().is_some_and(|id| id == message_id)
                                            }
                                            on_click=Box::new(move || {
                                                if context_selection_mode.get_untracked() {
                                                    on_toggle_context(message_id);
                                                } else {
                                                    on_message_click(message_id);
                                                }
                                            })
                                            on_toggle_select=Box::new(move || on_toggle_context(message_id))
                                            on_branch=Box::new(move || on_branch_from_message(message_id))
                                        />
                                    </>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }
            }}

            {move || context_selection_mode.get().then(|| view! {
                <div
                    class="selection-toolbar--entering sticky flex items-center justify-between gap-3 p-4 surface-overlay rounded-md shadow-md"
                    style="bottom: var(--space-4); z-index: var(--z-raised);"
                    role="toolbar"
                    aria-label="Context message selection"
                >
                    <span class="type-caption-strong text-primary">
                        {move || format!(
                            "{} selected for context",
                            selected_context_ids.get().len(),
                        )}
                    </span>
                    <div class="flex items-center gap-2">
                        <Button
                            variant=ButtonVariant::Ghost
                            size=ButtonSize::Small
                            on_click=Box::new(move |_| on_context_clear())
                        >
                            "Clear"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Small
                            on_click=Box::new(move |_| on_context_done())
                        >
                            "Done"
                        </Button>
                    </div>
                </div>
            })}

            // New messages indicator
            <NewMessagesIndicator count=new_below_count on_click=on_scroll_to_bottom />
        </div>
    }
}
