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
