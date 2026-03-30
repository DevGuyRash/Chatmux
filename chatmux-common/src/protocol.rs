//! Serialized message contracts for UI, background, and content-script communication.

use crate::{
    ApprovalMode, BlockingState, DiagnosticEvent, DiagnosticLevel, DiagnosticsQuery,
    DiagnosticsSnapshot, Dispatch, EdgePolicy, ExportFormat, ExportLayout, ExportProfile, Message,
    ProviderControlDefaults,
    ProviderControlSnapshot, ProviderHealth, ProviderId, Round, Run, Template, Workspace,
    WorkspaceId, WorkspaceSnapshot,
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
    PauseRun {
        run_id: crate::RunId,
    },
    ResumeRun {
        run_id: crate::RunId,
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
    SendManualMessage {
        workspace_id: WorkspaceId,
        targets: Vec<ProviderId>,
        text: String,
        approval_mode: ApprovalMode,
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
    ExportSelection {
        workspace_id: WorkspaceId,
        format: ExportFormat,
        layout: ExportLayout,
        profile_id: Option<crate::ExportProfileId>,
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
    RequestDiagnosticsSnapshot {
        query: DiagnosticsQuery,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
