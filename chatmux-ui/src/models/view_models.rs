//! UI-local view models.
//!
//! These types exist only for rendering convenience and are never
//! serialized across the wire. They wrap or derive from chatmux-common
//! canonical types.

use chatmux_common::{ParticipantBinding, Run, RunStatus, Workspace};

/// Workspace list item — display-oriented wrapper around Workspace.
/// Computed fields derived from WorkspaceSnapshot data.
#[derive(Clone, Debug)]
pub struct WorkspaceListItem {
    pub workspace: Workspace,
    pub provider_count: u32,
    pub message_count: u32,
    pub has_active_run: bool,
    pub is_archived: bool,
}

impl WorkspaceListItem {
    /// Derive a list item from a workspace and optional snapshot data.
    pub fn from_workspace(
        ws: Workspace,
        bindings: &[ParticipantBinding],
        run: Option<&Run>,
    ) -> Self {
        Self {
            provider_count: bindings.iter().filter(|b| b.workspace_id == ws.id).count() as u32,
            message_count: 0, // Derived from snapshot when available
            has_active_run: run.map(|r| r.status == RunStatus::Running).unwrap_or(false),
            is_archived: ws.tags.iter().any(|t| t == "archived"),
            workspace: ws,
        }
    }
}

/// Provider binding view — display wrapper around ParticipantBinding.
#[derive(Clone, Debug)]
pub struct ProviderBindingView {
    pub binding: ParticipantBinding,
    pub tab_info: Option<String>,
    pub last_activity: Option<String>,
}

impl From<ParticipantBinding> for ProviderBindingView {
    fn from(b: ParticipantBinding) -> Self {
        let tab_info = b.tab_id.map(|id| format!("Tab #{} bound", id));
        Self {
            binding: b,
            tab_info,
            last_activity: None,
        }
    }
}

/// Message display view — adds computed display fields.
#[derive(Clone, Debug)]
pub struct MessageView {
    pub message: chatmux_common::Message,
    /// Computed character count (body_text.len()).
    pub character_count: u32,
    /// Display status derived from associated dispatch outcome.
    pub display_status: Option<MessageDisplayStatus>,
}

/// Display-only message status, derived from DispatchOutcome + CaptureConfidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageDisplayStatus {
    Delivered,
    Pending,
    Timeout,
    Error,
    UncertainCapture,
}

impl MessageView {
    pub fn from_message(
        msg: chatmux_common::Message,
        dispatch_outcome: Option<chatmux_common::DispatchOutcome>,
    ) -> Self {
        let character_count = msg.body_text.len() as u32;
        let display_status = dispatch_outcome.map(|o| match o {
            chatmux_common::DispatchOutcome::Pending => MessageDisplayStatus::Pending,
            chatmux_common::DispatchOutcome::Delivered => MessageDisplayStatus::Delivered,
            chatmux_common::DispatchOutcome::Timeout => MessageDisplayStatus::Timeout,
            chatmux_common::DispatchOutcome::Error => MessageDisplayStatus::Error,
            chatmux_common::DispatchOutcome::Skipped => MessageDisplayStatus::Pending,
        });
        Self {
            message: msg,
            character_count,
            display_status,
        }
    }
}

// ---------------------------------------------------------------------------
// Human labels for wire enums
// ---------------------------------------------------------------------------
//
// These names appear on several screens each. Deriving them with `{:?}` leaks
// Rust variant spelling into the interface ("RelayToMany", "UnseenDeltaOnly"),
// and re-spelling them per call site is how the same mode ends up written three
// different ways in three places. One definition per enum, used everywhere.

/// Product name for an orchestration mode.
pub fn orchestration_mode_label(mode: chatmux_common::OrchestrationMode) -> &'static str {
    use chatmux_common::OrchestrationMode as Mode;
    match mode {
        Mode::Broadcast => "Broadcast",
        Mode::Directed => "Directed",
        Mode::RelayToOne => "Relay to one",
        Mode::RelayToMany => "Relay to many",
        Mode::DraftOnly => "Draft only",
        Mode::CopyOnly => "Copy only",
        Mode::Roundtable => "Roundtable",
        Mode::ModeratorJury => "Moderator / jury",
        Mode::RelayChain => "Relay chain",
        Mode::ModeratedAutonomous => "Moderated autonomous",
    }
}

/// Product name for a catch-up rule, including its configured size where the
/// number is the point of the rule.
pub fn catch_up_policy_label(policy: &chatmux_common::CatchUpPolicy) -> String {
    use chatmux_common::CatchUpPolicy as Policy;
    match policy {
        Policy::FullHistory => "Full history once".to_owned(),
        Policy::LastN { count } => format!("Last {count} messages"),
        Policy::SelectedRange { .. } => "Selected range".to_owned(),
        Policy::PinnedSummary { .. } => "Pinned summary".to_owned(),
        Policy::None => "No catch-up".to_owned(),
    }
}

/// Product name for an incremental delivery rule.
pub fn incremental_policy_label(policy: &chatmux_common::IncrementalPolicy) -> String {
    use chatmux_common::IncrementalPolicy as Policy;
    match policy {
        Policy::UnseenDeltaOnly => "Unseen delta only".to_owned(),
        Policy::LastResponseOnly => "Last response only".to_owned(),
        Policy::SlidingWindow { count } => format!("Sliding window of {count}"),
        Policy::FullHistoryEveryTime => "Full history every time".to_owned(),
        Policy::ManualOnly => "Manual only".to_owned(),
    }
}

/// Product name for a round barrier policy.
pub fn barrier_policy_label(policy: &chatmux_common::BarrierPolicy) -> &'static str {
    use chatmux_common::BarrierPolicy as Policy;
    match policy {
        Policy::WaitForAll => "Wait for all",
        Policy::Quorum { .. } => "Quorum",
        Policy::FirstFinisher => "First finisher",
        Policy::ManualAdvance => "Manual advance",
    }
}

/// Product name for a context strategy.
pub fn context_strategy_label(strategy: &chatmux_common::ContextStrategy) -> String {
    use chatmux_common::ContextStrategy as Strategy;
    match strategy {
        Strategy::WorkspaceDefault => "Workspace default".to_owned(),
        Strategy::FullHistory => "Full history".to_owned(),
        Strategy::LastN { count } => format!("Last {count} messages"),
        Strategy::SpecificRange { .. } => "Specific range".to_owned(),
        Strategy::PinnedSummary { .. } => "Pinned summary".to_owned(),
        Strategy::None => "No context".to_owned(),
    }
}
