//! Message log component (§3.3).
//!
//! Scrollable message list. Occupies all vertical space between
//! workspace header and composer. Handles round grouping, selection
//! mode, search highlighting, and new messages indicator.

use leptos::prelude::*;

use crate::models::Message;
use super::message_card::MessageCard;
use super::round_divider::RoundDivider;
use super::new_messages_indicator::NewMessagesIndicator;

/// Message log component.
#[component]
pub fn MessageLog(
    /// Messages to display.
    messages: ReadSignal<Vec<Message>>,
    /// Count of new messages below viewport.
    new_below_count: ReadSignal<u32>,
    /// Called when a message card is clicked (for inspection).
    on_message_click: impl Fn(uuid::Uuid) + 'static + Copy + Send,
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
                                            on_click=Box::new(move || on_message_click(msg_id))
                                        />
                                    </>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                }
            }}

            // New messages indicator
            <NewMessagesIndicator count=new_below_count on_click=on_scroll_to_bottom />
        </div>
    }
}
