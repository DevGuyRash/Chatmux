//! Message log state.

use leptos::prelude::*;

use crate::models::Message;

/// Message log state for the current workspace.
#[derive(Clone, Copy)]
pub struct MessageState {
    pub messages: ReadSignal<Vec<Message>>,
    pub set_messages: WriteSignal<Vec<Message>>,
    /// Whether the user is scrolled to the bottom of the log.
    pub at_bottom: ReadSignal<bool>,
    pub set_at_bottom: WriteSignal<bool>,
    /// Count of new messages below viewport (for the indicator).
    pub new_below_count: ReadSignal<u32>,
    pub set_new_below_count: WriteSignal<u32>,
    /// Message selected as the parent for the next composer send.
    pub branch_parent_id: ReadSignal<Option<crate::models::MessageId>>,
    pub set_branch_parent_id: WriteSignal<Option<crate::models::MessageId>>,
}

pub fn provide_message_state() -> MessageState {
    let (messages, set_messages) = signal(Vec::<Message>::new());
    let (at_bottom, set_at_bottom) = signal(true);
    let (new_below_count, set_new_below_count) = signal(0u32);
    let (branch_parent_id, set_branch_parent_id) = signal(None);

    let state = MessageState {
        messages,
        set_messages,
        at_bottom,
        set_at_bottom,
        new_below_count,
        set_new_below_count,
        branch_parent_id,
        set_branch_parent_id,
    };

    provide_context(state);
    state
}
