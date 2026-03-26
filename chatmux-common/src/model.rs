//! Canonical Chatmux data model.

use crate::{
    BindingId, DeliveryCursorId, DiagnosticEventId, DispatchId, EdgePolicyId, ExportProfileId,
    MessageId, RoundId, RunId, TemplateId, WorkspaceId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    User,
    System,
    Gpt,
    Gemini,
    Grok,
    Claude,
}

impl ProviderId {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::User => "You",
            Self::System => "System",
            Self::Gpt => "ChatGPT",
            Self::Gemini => "Gemini",
            Self::Grok => "Grok",
            Self::Claude => "Claude",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureConfidence {
    Certain,
    Uncertain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Created,
    Running,
    Paused,
    Completed,
    Aborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoundStatus {
    Pending,
    Running,
    Completed,
    TimedOut,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchOutcome {
    Delivered,
    Timeout,
    Error,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderHealth {
    Disconnected,
    Ready,
    Composing,
    Sending,
    Generating,
    Completed,
    PermissionMissing,
    LoginRequired,
    DomMismatch,
    Blocked,
    RateLimited,
    SendFailed,
    CaptureUncertain,
    DegradedManualOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BlockingState {
    PermissionMissing { detail: String },
    LoginRequired { detail: String },
    RateLimited { detail: String },
    ProviderError { detail: String },
    InputUnavailable { detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationRef {
    pub conversation_id: Option<String>,
    pub title: Option<String>,
    pub model_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Paragraph {
        text: String,
    },
    Heading {
        level: u8,
        text: String,
    },
    CodeFence {
        language: Option<String>,
        code: String,
    },
    BulletedList {
        items: Vec<String>,
    },
    NumberedList {
        items: Vec<String>,
    },
    Quote {
        text: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub enabled_providers: BTreeSet<ProviderId>,
    pub default_mode: OrchestrationMode,
    pub default_context_strategy: ContextStrategy,
    pub default_template_id: Option<TemplateId>,
    pub active_export_profile_ids: Vec<ExportProfileId>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantBinding {
    pub id: BindingId,
    pub workspace_id: WorkspaceId,
    pub provider_id: ProviderId,
    pub tab_id: Option<u32>,
    pub window_id: Option<u32>,
    pub origin: Option<String>,
    pub conversation_ref: Option<ConversationRef>,
    pub health_state: ProviderHealth,
    pub capability_snapshot: CapabilitySnapshot,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub workspace_id: WorkspaceId,
    pub participant_id: ProviderId,
    pub role: MessageRole,
    pub round: Option<u32>,
    pub timestamp: DateTime<Utc>,
    pub body_text: String,
    pub body_blocks: Vec<Block>,
    pub source_binding_id: Option<BindingId>,
    pub dispatch_id: Option<DispatchId>,
    pub raw_capture_ref: Option<String>,
    pub tags: Vec<String>,
    pub capture_confidence: CaptureConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    pub id: RunId,
    pub workspace_id: WorkspaceId,
    pub mode: OrchestrationMode,
    pub graph_snapshot: RoutingGraph,
    pub participant_set: BTreeSet<ProviderId>,
    pub barrier_policy: BarrierPolicy,
    pub timing_policy: TimingPolicy,
    pub stop_policy: StopPolicy,
    pub status: RunStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Round {
    pub id: RoundId,
    pub run_id: RunId,
    pub round_number: u32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: RoundStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dispatch {
    pub id: DispatchId,
    pub run_id: RunId,
    pub round_id: Option<RoundId>,
    pub round_number: u32,
    pub target_participant_id: ProviderId,
    pub source_message_ids: Vec<MessageId>,
    pub template_id: Option<TemplateId>,
    pub rendered_payload: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub captured_at: Option<DateTime<Utc>>,
    pub outcome: DispatchOutcome,
    pub error_detail: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgePolicy {
    pub id: EdgePolicyId,
    pub workspace_id: WorkspaceId,
    pub source_participant_id: ProviderId,
    pub target_participant_id: ProviderId,
    pub enabled: bool,
    pub catch_up_policy: CatchUpPolicy,
    pub incremental_policy: IncrementalPolicy,
    pub self_exclusion: bool,
    pub include_user_turns: bool,
    pub include_system_notes: bool,
    pub include_pinned_summaries: bool,
    pub include_moderator_annotations: bool,
    pub include_target_prior_turns: bool,
    pub truncation_policy: TruncationPolicy,
    pub priority: i32,
    pub approval_mode: ApprovalMode,
    pub template_id: Option<TemplateId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryCursor {
    pub id: DeliveryCursorId,
    pub workspace_id: WorkspaceId,
    pub source_participant_id: ProviderId,
    pub target_participant_id: ProviderId,
    pub last_delivered_message_id: Option<MessageId>,
    pub last_delivered_at: Option<DateTime<Utc>>,
    pub frozen: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    pub id: TemplateId,
    pub workspace_id: WorkspaceId,
    pub kind: TemplateKind,
    pub name: String,
    pub version: String,
    pub body_template: String,
    pub preamble: Option<String>,
    pub metadata_template: Option<String>,
    pub filename_template: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProfile {
    pub id: ExportProfileId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub scope_preset: ExportScopePreset,
    pub filter_preset: ExportFilterPreset,
    pub format: ExportFormat,
    pub layout: ExportLayout,
    pub include_flags: MetadataIncludeFlags,
    pub filename_template: String,
    pub metadata_template: Option<String>,
    pub prefer_copy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticEvent {
    pub id: DiagnosticEventId,
    pub workspace_id: WorkspaceId,
    pub binding_id: Option<BindingId>,
    pub timestamp: DateTime<Utc>,
    pub level: DiagnosticLevel,
    pub code: String,
    pub detail: String,
    pub snapshot_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySnapshot {
    pub supports_follow_up_while_generating: bool,
    pub can_auto_send: bool,
    pub can_capture_full_history: bool,
    pub can_capture_delta: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingGraph {
    pub nodes: BTreeSet<ProviderId>,
    pub edges: Vec<RouteEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteEdge {
    pub source: ProviderId,
    pub target: ProviderId,
    pub policy_id: Option<EdgePolicyId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationMode {
    Broadcast,
    Directed,
    RelayToOne,
    RelayToMany,
    DraftOnly,
    CopyOnly,
    Roundtable,
    ModeratorJury,
    RelayChain,
    ModeratedAutonomous,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum ContextStrategy {
    WorkspaceDefault,
    FullHistory,
    LastN {
        count: usize,
    },
    SpecificRange {
        start: Option<MessageId>,
        end: Option<MessageId>,
    },
    PinnedSummary {
        summary: String,
    },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CatchUpPolicy {
    FullHistory,
    LastN {
        count: usize,
    },
    SelectedRange {
        start: Option<MessageId>,
        end: Option<MessageId>,
    },
    PinnedSummary {
        summary_message_id: Option<MessageId>,
    },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncrementalPolicy {
    UnseenDeltaOnly,
    LastResponseOnly,
    SlidingWindow { count: usize },
    FullHistoryEveryTime,
    ManualOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TruncationPolicy {
    None,
    WarnOnly {
        soft_character_limit: usize,
    },
    TrimOldest {
        soft_character_limit: usize,
    },
    SwapForSummary {
        soft_character_limit: usize,
        summary_message_id: Option<MessageId>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalMode {
    AutoSend,
    RequireUserConfirmation,
    DraftOnly,
    CopyOnly,
    ManualSend,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BarrierPolicy {
    WaitForAll,
    Quorum { providers: BTreeSet<ProviderId> },
    FirstFinisher,
    ManualAdvance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimingPolicy {
    pub per_provider_generation_timeout_secs: u64,
    pub per_provider_cooldown_secs: u64,
    pub inter_round_delay_secs: u64,
    pub jitter_percent: u8,
    pub max_concurrent_sends: usize,
    pub max_rounds: Option<u32>,
    pub global_run_timeout_secs: Option<u64>,
    pub exponential_backoff_base_secs: u64,
}

impl Default for TimingPolicy {
    fn default() -> Self {
        Self {
            per_provider_generation_timeout_secs: 120,
            per_provider_cooldown_secs: 2,
            inter_round_delay_secs: 5,
            jitter_percent: 20,
            max_concurrent_sends: 4,
            max_rounds: None,
            global_run_timeout_secs: None,
            exponential_backoff_base_secs: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopPolicy {
    pub stop_on_max_rounds: bool,
    pub stop_on_manual_pause: bool,
    pub stop_on_sentinel_phrase: Option<String>,
    pub repeated_provider_failure_limit: Option<u32>,
    pub repeated_timeout_limit: Option<u32>,
    pub stagnation_window: Option<u32>,
    pub require_approval_between_rounds: bool,
}

impl Default for StopPolicy {
    fn default() -> Self {
        Self {
            stop_on_max_rounds: true,
            stop_on_manual_pause: true,
            stop_on_sentinel_phrase: None,
            repeated_provider_failure_limit: Some(3),
            repeated_timeout_limit: Some(3),
            stagnation_window: Some(2),
            require_approval_between_rounds: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateKind {
    BuiltinWrappedXml,
    BuiltinMarkdownSections,
    BuiltinPlainLabels,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportScopePreset {
    EntireWorkspace,
    SingleProvider,
    SingleRun,
    SelectedRounds,
    SelectedMessages,
    ProviderOnlySubset,
    DispatchSubset,
    DiagnosticSubset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExportFilterPreset {
    pub participants: BTreeSet<ProviderId>,
    pub roles: BTreeSet<MessageRole>,
    pub round_range: Option<(u32, u32)>,
    pub time_range_iso: Option<(String, String)>,
    pub run_id: Option<RunId>,
    pub tags: Vec<String>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Markdown,
    Json,
    Toml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportLayout {
    Chronological,
    GroupedByRound,
    GroupedByParticipant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MetadataIncludeFlags {
    pub workspace_name: bool,
    pub workspace_id: bool,
    pub export_title: bool,
    pub export_timestamp: bool,
    pub scope_type: bool,
    pub selected_participants: bool,
    pub orchestration_mode: bool,
    pub run_id: bool,
    pub round_range: bool,
    pub message_count: bool,
    pub template_used: bool,
    pub context_strategy_snapshot: bool,
    pub edge_policy_snapshot: bool,
    pub conversation_refs: bool,
    pub model_labels: bool,
    pub browser_name: bool,
    pub extension_version: bool,
    pub export_profile_name: bool,
    pub tags_and_notes: bool,
    pub diagnostics_summary: bool,
    pub raw_payload_inclusion: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Critical,
    Warning,
    Info,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkspaceSnapshot {
    pub workspace: Option<Workspace>,
    pub bindings: Vec<ParticipantBinding>,
    pub runs: Vec<Run>,
    pub recent_messages: Vec<Message>,
    pub diagnostics: Vec<DiagnosticEvent>,
    pub edge_policies: Vec<EdgePolicy>,
    pub delivery_cursors: Vec<DeliveryCursor>,
    pub templates: Vec<Template>,
    pub export_profiles: Vec<ExportProfile>,
    pub kill_switch_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RunLedger {
    pub run: Option<Run>,
    pub rounds: Vec<Round>,
    pub dispatches: Vec<Dispatch>,
    pub delivery_cursors: Vec<DeliveryCursor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MetadataBag {
    pub values: BTreeMap<String, String>,
}
