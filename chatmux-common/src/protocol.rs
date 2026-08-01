//! Serialized message contracts for UI, background, and content-script communication.

use crate::{
    ApprovalMode, BlockingState, DiagnosticEvent, DiagnosticLevel, DiagnosticsQuery,
    DiagnosticsSnapshot, Dispatch, EdgePolicy, ExportFormat, ExportLayout, ExportProfile,
    ExportRequest, Message, NextRoundPackage, ProviderControlDefaults, ProviderControlSnapshot,
    ProviderHealth, ProviderId, Round, Run, RunLedger, Template, Workspace, WorkspaceId,
    WorkspaceSnapshot,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiCommand {
    RequestWorkspaceList,
    CreateWorkspace {
        name: String,
    },
    DeleteWorkspace {
        workspace_id: WorkspaceId,
    },
    RenameWorkspace {
        workspace_id: WorkspaceId,
        name: String,
    },
    DuplicateWorkspace {
        workspace_id: WorkspaceId,
    },
    ExportWorkspaceArchive {
        workspace_id: WorkspaceId,
    },
    ImportWorkspaceArchive {
        body: String,
    },
    SetWorkspaceArchived {
        workspace_id: WorkspaceId,
        archived: bool,
    },
    OpenWorkspace {
        workspace_id: WorkspaceId,
    },
    PersistTemplate {
        template: Template,
    },
    PersistEdgePolicy {
        policy: EdgePolicy,
    },
    PersistPinnedSummary {
        workspace_id: WorkspaceId,
        summary_message_id: Option<crate::MessageId>,
        name: String,
        body: String,
    },
    DeletePinnedSummary {
        workspace_id: WorkspaceId,
        summary_message_id: crate::MessageId,
    },
    ResetDeliveryCursor {
        cursor_id: crate::DeliveryCursorId,
    },
    SetDeliveryCursorFrozen {
        cursor_id: crate::DeliveryCursorId,
        frozen: bool,
    },
    PersistExportProfile {
        profile: ExportProfile,
    },
    DeleteTemplate {
        template_id: crate::TemplateId,
    },
    StartRun {
        workspace_id: WorkspaceId,
        mode: crate::OrchestrationMode,
    },
    StartConfiguredRun {
        workspace_id: WorkspaceId,
        configuration: crate::RunConfiguration,
    },
    PauseRun {
        run_id: crate::RunId,
    },
    ResumeRun {
        run_id: crate::RunId,
    },
    PreviewNextRound {
        run_id: crate::RunId,
    },
    ResumeRunWithOverrides {
        run_id: crate::RunId,
        payload_overrides: std::collections::BTreeMap<ProviderId, String>,
        skipped_targets: std::collections::BTreeSet<ProviderId>,
        injected_user_message: Option<String>,
    },
    StepRun {
        run_id: crate::RunId,
    },
    StopRun {
        run_id: crate::RunId,
    },
    AbortRun {
        run_id: crate::RunId,
    },
    PreviewManualMessage {
        workspace_id: WorkspaceId,
        targets: Vec<ProviderId>,
        text: String,
        #[serde(default)]
        selected_message_ids: std::collections::BTreeSet<crate::MessageId>,
        #[serde(default)]
        pinned_note: Option<String>,
        #[serde(default)]
        target_notes: std::collections::BTreeMap<ProviderId, String>,
        #[serde(default)]
        include_target_prior_turns: bool,
        #[serde(default)]
        parent_message_id: Option<crate::MessageId>,
    },
    SendManualMessage {
        workspace_id: WorkspaceId,
        targets: Vec<ProviderId>,
        text: String,
        approval_mode: ApprovalMode,
        #[serde(default)]
        selected_message_ids: std::collections::BTreeSet<crate::MessageId>,
        #[serde(default)]
        pinned_note: Option<String>,
        #[serde(default)]
        target_notes: std::collections::BTreeMap<ProviderId, String>,
        #[serde(default)]
        include_target_prior_turns: bool,
        #[serde(default)]
        payload_overrides: std::collections::BTreeMap<ProviderId, String>,
        #[serde(default)]
        parent_message_id: Option<crate::MessageId>,
    },
    /// Acknowledge that the exact payload stored for a pending dispatch was delivered.
    ///
    /// The background runtime sends this only after provider-page I/O succeeds. Repeated
    /// acknowledgements for the same delivered dispatch are idempotent.
    AcknowledgeDispatchDelivered {
        dispatch_id: crate::DispatchId,
    },
    /// Acknowledge that provider-page I/O failed for a pending dispatch.
    ///
    /// The failure detail is persisted on the dispatch. This transition never advances a
    /// delivery cursor.
    AcknowledgeDispatchFailed {
        dispatch_id: crate::DispatchId,
        detail: String,
    },
    /// Attach provider responses captured after a delivered dispatch.
    ///
    /// Every captured message is linked to `dispatch_id` by the coordinator before it is
    /// persisted. This transition never changes a failed or skipped dispatch to delivered.
    AcknowledgeDispatchCaptured {
        dispatch_id: crate::DispatchId,
        messages: Vec<Message>,
    },
    SyncProviderConversation {
        workspace_id: WorkspaceId,
        provider: ProviderId,
    },
    RequestProviderTabCandidates {
        workspace_id: WorkspaceId,
        provider: ProviderId,
    },
    BindProviderTab {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        tab_id: u32,
        window_id: Option<u32>,
        origin: Option<String>,
        tab_title: Option<String>,
        tab_url: Option<String>,
        conversation_id: Option<String>,
        conversation_title: Option<String>,
        conversation_url: Option<String>,
        pin: bool,
    },
    OpenProviderTab {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        prefer_existing: bool,
    },
    ExportSelection {
        workspace_id: WorkspaceId,
        format: ExportFormat,
        layout: ExportLayout,
        profile_id: Option<crate::ExportProfileId>,
    },
    ExportConfigured {
        request: ExportRequest,
    },
    RequestMessageInspection {
        message_id: crate::MessageId,
    },
    SetKillSwitch {
        active: bool,
    },
    ClearWorkspaceData {
        workspace_id: WorkspaceId,
    },
    ToggleProvider {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        enabled: bool,
    },
    RequestProviderControlState {
        workspace_id: WorkspaceId,
        provider: ProviderId,
    },
    CreateProviderProject {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        title: String,
    },
    SelectProviderProject {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        project_id: String,
    },
    CreateProviderConversation {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        project_id: Option<String>,
        title: String,
    },
    SelectProviderConversation {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        conversation_id: String,
    },
    SetProviderModel {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        model_id: String,
    },
    SetProviderReasoning {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        reasoning_id: String,
    },
    SetProviderFeatureFlag {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        key: String,
        enabled: bool,
    },
    PersistProviderDefaults {
        provider: ProviderId,
        defaults: ProviderControlDefaults,
    },
    RequestWorkspaceSnapshot {
        workspace_id: WorkspaceId,
    },
    RequestRunLedger {
        run_id: crate::RunId,
    },
    RequestDiagnosticsSnapshot {
        query: DiagnosticsQuery,
    },
    ClearDiagnostics {
        query: DiagnosticsQuery,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)] // Reason: serialized UI wire variants stay inline for stable serde contracts.
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiEvent {
    WorkspaceList {
        workspaces: Vec<Workspace>,
    },
    WorkspaceSnapshot {
        snapshot: WorkspaceSnapshot,
    },
    RunUpdated {
        run: Run,
        rounds: Vec<Round>,
    },
    NextRoundPreview {
        run_id: crate::RunId,
        round_number: u32,
        packages: Vec<NextRoundPackage>,
    },
    ManualMessagePreview {
        packages: Vec<NextRoundPackage>,
    },
    RunLedgerSnapshot {
        ledger: RunLedger,
    },
    MessageCaptured {
        message: Message,
    },
    DispatchUpdated {
        dispatch: Dispatch,
    },
    DiagnosticRaised {
        diagnostic: DiagnosticEvent,
    },
    DiagnosticsSnapshot {
        snapshot: DiagnosticsSnapshot,
    },
    ProviderHealthChanged {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        health: ProviderHealth,
        blocking_state: Option<BlockingState>,
    },
    ProviderControlUpdated {
        workspace_id: WorkspaceId,
        snapshot: ProviderControlSnapshot,
    },
    ProviderTabCandidates {
        workspace_id: WorkspaceId,
        provider: ProviderId,
        candidates: Vec<crate::ProviderTabCandidate>,
    },
    ProviderDefaultsUpdated {
        provider: ProviderId,
        defaults: ProviderControlDefaults,
    },
    ExportRendered {
        format: ExportFormat,
        mime_type: String,
        filename: String,
        body: String,
    },
    MessageInspection {
        message: Option<Message>,
        dispatch: Option<Dispatch>,
        sent_payload: Option<String>,
        raw_response_text: Option<String>,
        network_capture: Option<crate::ProviderNetworkCapture>,
    },
    KillSwitchChanged {
        active: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackgroundToAdapter {
    StructuralProbe,
    GetHealth,
    InjectInput {
        text: String,
    },
    Send,
    ExtractLatestResponse,
    ExtractFullHistory,
    ExtractIncrementalDelta {
        after_message_id: Option<crate::MessageId>,
    },
    DetectBlockingState,
    GetConversationRef,
    GetProviderSnapshot,
    CreateProject {
        title: String,
    },
    SelectProject {
        project_id: String,
    },
    CreateConversation {
        project_id: Option<String>,
        title: String,
    },
    SelectConversation {
        conversation_id: String,
    },
    SetModel {
        model_id: String,
    },
    SetReasoning {
        reasoning_id: String,
    },
    SetFeatureFlag {
        key: String,
        enabled: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)] // Reason: serialized adapter wire variants stay inline for stable serde contracts.
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdapterToBackground {
    StructuralProbePassed {
        provider: ProviderId,
    },
    StructuralProbeFailed {
        provider: ProviderId,
        detail: String,
    },
    HealthReport {
        provider: ProviderId,
        health: ProviderHealth,
    },
    BlockingStateDetected {
        provider: ProviderId,
        blocking_state: BlockingState,
    },
    MessagesCaptured {
        provider: ProviderId,
        messages: Vec<Message>,
    },
    ConversationRefDiscovered {
        provider: ProviderId,
        conversation_ref: Option<crate::ConversationRef>,
    },
    ProviderControlSnapshotCaptured {
        provider: ProviderId,
        snapshot: crate::ProviderControlSnapshot,
    },
    CommandFailed {
        provider: ProviderId,
        level: DiagnosticLevel,
        detail: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContextStrategy, MetadataIncludeFlags, OrchestrationMode, TimingPolicy};
    use chrono::Utc;
    use std::collections::BTreeSet;

    #[test]
    fn serializes_ui_command_contract() {
        let command = UiCommand::StartRun {
            workspace_id: WorkspaceId::new(),
            mode: OrchestrationMode::Roundtable,
        };

        let json = serde_json::to_string(&command).expect("command should serialize");
        assert!(json.contains("\"start_run\""));
    }

    #[test]
    fn send_manual_message_deserializes_without_parent_message_id() {
        let command = serde_json::json!({
            "type": "send_manual_message",
            "workspace_id": WorkspaceId::new(),
            "targets": ["gpt"],
            "text": "hello",
            "approval_mode": "auto_send"
        });

        let command: UiCommand =
            serde_json::from_value(command).expect("legacy command should deserialize");

        let UiCommand::SendManualMessage {
            parent_message_id, ..
        } = command
        else {
            panic!("expected send_manual_message");
        };
        assert_eq!(parent_message_id, None);
    }

    #[test]
    fn dispatch_acknowledgement_commands_have_stable_wire_names() {
        let dispatch_id = crate::DispatchId::new();
        let commands = [
            UiCommand::AcknowledgeDispatchDelivered { dispatch_id },
            UiCommand::AcknowledgeDispatchFailed {
                dispatch_id,
                detail: "failed".to_owned(),
            },
            UiCommand::AcknowledgeDispatchCaptured {
                dispatch_id,
                messages: Vec::new(),
            },
        ];

        let wire_names = commands
            .iter()
            .map(|command| {
                serde_json::to_value(command)
                    .ok()
                    .and_then(|value| value["type"].as_str().map(ToOwned::to_owned))
            })
            .collect::<Vec<_>>();

        assert_eq!(
            wire_names,
            vec![
                Some("acknowledge_dispatch_delivered".to_owned()),
                Some("acknowledge_dispatch_failed".to_owned()),
                Some("acknowledge_dispatch_captured".to_owned()),
            ]
        );
    }

    #[test]
    fn provider_id_display_names_are_stable() {
        assert_eq!(crate::ProviderId::Gpt.display_name(), "ChatGPT");
        assert_eq!(crate::ProviderId::Claude.display_name(), "Claude");
    }

    #[test]
    fn metadata_flags_default_to_disabled() {
        let flags = MetadataIncludeFlags::default();
        assert!(!flags.workspace_name);
    }

    #[test]
    fn timing_policy_has_expected_defaults() {
        let policy = TimingPolicy::default();
        assert_eq!(policy.per_provider_generation_timeout_secs, 120);
        assert_eq!(policy.jitter_percent, 20);
    }

    #[test]
    fn workspace_snapshot_defaults_empty() {
        let snapshot = WorkspaceSnapshot::default();
        assert!(snapshot.workspace.is_none());
        assert!(snapshot.bindings.is_empty());
    }

    #[test]
    fn run_support_types_compile_together() {
        let _ = (
            ContextStrategy::WorkspaceDefault,
            Utc::now(),
            BTreeSet::<crate::ProviderId>::new(),
        );
    }
}
