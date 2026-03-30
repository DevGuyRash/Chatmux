//! Canonical Chatmux data model.

use crate::{
    BindingId, DeliveryCursorId, DiagnosticEventId, DispatchId, EdgePolicyId, ExportProfileId,
    MessageId, RoundId, RunId, TemplateId, WorkspaceId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
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
    pub url: Option<String>,
    pub model_label: Option<String>,
}

impl ConversationRef {
    pub fn matches_target(&self, target: &ConversationRef) -> bool {
        match (&self.conversation_id, &target.conversation_id) {
            (Some(current_id), Some(target_id)) => current_id == target_id,
            _ => normalized_chat_url(self.url.as_deref())
                .zip(normalized_chat_url(target.url.as_deref()))
                .is_some_and(|(current_url, target_url)| current_url == target_url),
        }
    }

    pub fn has_identity(&self) -> bool {
        self.conversation_id.is_some() || normalized_chat_url(self.url.as_deref()).is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderNetworkCapture {
    pub request_method: Option<String>,
    pub request_url: Option<String>,
    pub request_body: Option<String>,
    pub response_status: Option<u16>,
    pub response_body: Option<String>,
    pub capture_strategy: Option<String>,
    pub conversation_ref: Option<ConversationRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderTabCandidate {
    pub tab_id: u32,
    pub window_id: Option<u32>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub conversation_id: Option<String>,
    pub conversation_title: Option<String>,
    pub is_active: bool,
    pub is_bound: bool,
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStrategy {
    PublicApi,
    Network,
    Dom,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderProject {
    pub id: String,
    pub title: String,
    pub is_active: bool,
    pub provider_metadata: MetadataBag,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderConversation {
    pub id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub is_active: bool,
    pub model_label: Option<String>,
    pub provider_metadata: MetadataBag,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderModelOption {
    pub id: String,
    pub label: String,
    pub is_active: bool,
    pub provider_metadata: MetadataBag,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderReasoningOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub provider_metadata: MetadataBag,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderFeatureFlag {
    pub key: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderControlCapabilities {
    pub supports_projects: bool,
    pub supports_project_creation: bool,
    pub supports_conversations: bool,
    pub supports_conversation_creation: bool,
    pub supports_model_selection: bool,
    pub supports_reasoning_selection: bool,
    pub supports_feature_flags: bool,
    pub supports_sync: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderControlState {
    pub project_id: Option<String>,
    pub project_title: Option<String>,
    pub conversation_id: Option<String>,
    pub conversation_title: Option<String>,
    pub model_id: Option<String>,
    pub model_label: Option<String>,
    pub reasoning_id: Option<String>,
    pub reasoning_label: Option<String>,
    pub feature_flags: BTreeMap<String, bool>,
    pub last_strategy: Option<ProviderStrategy>,
    pub degraded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderControlSnapshot {
    pub provider: ProviderId,
    pub capabilities: ProviderControlCapabilities,
    pub state: ProviderControlState,
    pub projects: Vec<ProviderProject>,
    pub conversations: Vec<ProviderConversation>,
    pub models: Vec<ProviderModelOption>,
    pub reasoning_options: Vec<ProviderReasoningOption>,
    pub feature_flags: Vec<ProviderFeatureFlag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderControlDefaults {
    pub project_id: Option<String>,
    pub project_title: Option<String>,
    pub model_id: Option<String>,
    pub model_label: Option<String>,
    pub reasoning_id: Option<String>,
    pub reasoning_label: Option<String>,
    pub feature_flags: BTreeMap<String, bool>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParticipantBinding {
    pub id: BindingId,
    pub workspace_id: WorkspaceId,
    pub provider_id: ProviderId,
    pub tab_id: Option<u32>,
    pub window_id: Option<u32>,
    pub origin: Option<String>,
    pub tab_title: Option<String>,
    pub tab_url: Option<String>,
    pub pinned: bool,
    pub stale: bool,
    pub bound_conversation_ref: Option<ConversationRef>,
    pub conversation_ref: Option<ConversationRef>,
    pub provider_control: Option<ProviderControlState>,
    pub health_state: ProviderHealth,
    pub capability_snapshot: CapabilitySnapshot,
    pub last_seen_at: Option<DateTime<Utc>>,
}

impl<'de> Deserialize<'de> for ParticipantBinding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ParticipantBindingCompat {
            id: BindingId,
            workspace_id: WorkspaceId,
            provider_id: ProviderId,
            tab_id: Option<u32>,
            window_id: Option<u32>,
            origin: Option<String>,
            tab_title: Option<String>,
            tab_url: Option<String>,
            #[serde(default)]
            pinned: bool,
            #[serde(default)]
            stale: bool,
            #[serde(default)]
            bound_conversation_ref: Option<ConversationRef>,
            #[serde(default)]
            conversation_ref: Option<ConversationRef>,
            #[serde(default)]
            provider_control: Option<ProviderControlState>,
            #[serde(default = "default_binding_health_state")]
            health_state: ProviderHealth,
            #[serde(default)]
            capability_snapshot: Option<CapabilitySnapshot>,
            #[serde(default)]
            last_seen_at: Option<DateTime<Utc>>,
        }

        let compat = ParticipantBindingCompat::deserialize(deserializer)?;
        Ok(Self {
            id: compat.id,
            workspace_id: compat.workspace_id,
            provider_id: compat.provider_id,
            tab_id: compat.tab_id,
            window_id: compat.window_id,
            origin: compat.origin,
            tab_title: compat.tab_title,
            tab_url: compat.tab_url,
            pinned: compat.pinned,
            stale: compat.stale,
            bound_conversation_ref: compat
                .bound_conversation_ref
                .or_else(|| compat.conversation_ref.clone()),
            conversation_ref: compat.conversation_ref,
            provider_control: compat.provider_control,
            health_state: compat.health_state,
            capability_snapshot: compat
                .capability_snapshot
                .unwrap_or_else(|| default_capability_snapshot(compat.provider_id)),
            last_seen_at: compat.last_seen_at,
        })
    }
}

impl ParticipantBinding {
    pub fn has_bound_target(&self) -> bool {
        self.bound_conversation_ref
            .as_ref()
            .is_some_and(ConversationRef::has_identity)
    }

    pub fn matches_bound_target(&self) -> bool {
        match (&self.bound_conversation_ref, &self.conversation_ref) {
            (Some(bound), Some(current)) if bound.has_identity() => current.matches_target(bound),
            (Some(bound), None) => !bound.has_identity(),
            _ => true,
        }
    }
}

fn normalized_chat_url(url: Option<&str>) -> Option<String> {
    let url = url?;
    let without_fragment = url.split('#').next().unwrap_or(url);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment);
    Some(without_query.trim_end_matches('/').to_owned())
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
    pub raw_response_text: Option<String>,
    pub network_capture: Option<ProviderNetworkCapture>,
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
#[serde(default)]
pub struct DiagnosticEvent {
    pub id: DiagnosticEventId,
    pub workspace_id: WorkspaceId,
    pub scope: DiagnosticScope,
    pub source: DiagnosticSource,
    pub binding_id: Option<BindingId>,
    pub provider_id: Option<ProviderId>,
    pub run_id: Option<RunId>,
    pub round_id: Option<RoundId>,
    pub message_id: Option<MessageId>,
    pub dispatch_id: Option<DispatchId>,
    pub timestamp: DateTime<Utc>,
    pub level: DiagnosticLevel,
    pub code: String,
    pub title: String,
    pub summary: String,
    pub detail: String,
    pub tags: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub artifact_refs: Vec<DiagnosticArtifactRef>,
    pub snapshot_ref: Option<String>,
}

impl Default for DiagnosticEvent {
    fn default() -> Self {
        Self {
            id: DiagnosticEventId::new(),
            workspace_id: WorkspaceId::new(),
            scope: DiagnosticScope::default(),
            source: DiagnosticSource::default(),
            binding_id: None,
            provider_id: None,
            run_id: None,
            round_id: None,
            message_id: None,
            dispatch_id: None,
            timestamp: Utc::now(),
            level: DiagnosticLevel::Info,
            code: String::new(),
            title: String::new(),
            summary: String::new(),
            detail: String::new(),
            tags: Vec::new(),
            attributes: BTreeMap::new(),
            artifact_refs: Vec::new(),
            snapshot_ref: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticScope {
    #[default]
    Workspace,
    App,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSource {
    #[default]
    Background,
    Ui,
    Adapter,
    RunLoop,
    Storage,
    Bridge,
    Permissions,
    Export,
    BrowserApi,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DiagnosticArtifactRef {
    pub id: String,
    pub kind: DiagnosticArtifactKind,
    pub label: String,
    pub mime_type: String,
    pub storage_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticArtifactKind {
    #[default]
    Json,
    Text,
    Snapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySnapshot {
    pub supports_follow_up_while_generating: bool,
    pub can_auto_send: bool,
    pub can_capture_full_history: bool,
    pub can_capture_delta: bool,
}

fn default_binding_health_state() -> ProviderHealth {
    ProviderHealth::Ready
}

fn default_capability_snapshot(provider: ProviderId) -> CapabilitySnapshot {
    match provider {
        ProviderId::Gpt => CapabilitySnapshot {
            supports_follow_up_while_generating: false,
            can_auto_send: true,
            can_capture_full_history: true,
            can_capture_delta: true,
        },
        ProviderId::Gemini | ProviderId::Grok | ProviderId::Claude => CapabilitySnapshot {
            supports_follow_up_while_generating: false,
            can_auto_send: true,
            can_capture_full_history: false,
            can_capture_delta: false,
        },
        ProviderId::User | ProviderId::System => CapabilitySnapshot {
            supports_follow_up_while_generating: false,
            can_auto_send: false,
            can_capture_full_history: false,
            can_capture_delta: false,
        },
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticsSearchMode {
    #[default]
    Plain,
    Regex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticsDetailLevel {
    Overview,
    #[default]
    Standard,
    Verbose,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DiagnosticsQuery {
    pub workspace_id: Option<WorkspaceId>,
    pub include_global: bool,
    pub levels: Vec<DiagnosticLevel>,
    pub sources: Vec<DiagnosticSource>,
    pub providers: Vec<ProviderId>,
    pub text_query: Option<String>,
    pub search_mode: DiagnosticsSearchMode,
    pub case_sensitive: bool,
    pub context_before: u32,
    pub context_after: u32,
    pub detail_level: DiagnosticsDetailLevel,
    pub include_artifacts: bool,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WorkspaceDiagnosticsSummary {
    pub workspace_id: Option<WorkspaceId>,
    pub total: u32,
    pub critical: u32,
    pub warning: u32,
    pub info: u32,
    pub debug: u32,
    pub last_event_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DiagnosticsSnapshot {
    pub events: Vec<DiagnosticEvent>,
    pub summary: WorkspaceDiagnosticsSummary,
    pub queued_count: u32,
    pub total_available: u32,
    pub retention_event_cap: Option<u32>,
    pub retention_artifact_bytes_cap: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WorkspaceSnapshot {
    pub workspace: Option<Workspace>,
    pub bindings: Vec<ParticipantBinding>,
    pub provider_controls: Vec<ProviderControlSnapshot>,
    pub runs: Vec<Run>,
    pub recent_messages: Vec<Message>,
    pub diagnostics: Vec<DiagnosticEvent>,
    pub diagnostics_summary: WorkspaceDiagnosticsSummary,
    pub edge_policies: Vec<EdgePolicy>,
    pub delivery_cursors: Vec<DeliveryCursor>,
    pub templates: Vec<Template>,
    pub export_profiles: Vec<ExportProfile>,
    pub kill_switch_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn participant_binding_deserializes_legacy_records_without_pinned() {
        let payload = json!({
            "id": BindingId::new(),
            "workspace_id": WorkspaceId::new(),
            "provider_id": "gpt",
            "tab_id": 42,
            "window_id": 7,
            "origin": "https://chatgpt.com",
            "tab_title": "ChatGPT",
            "tab_url": "https://chatgpt.com/c/abc"
        });

        let binding: ParticipantBinding =
            serde_json::from_value(payload).expect("legacy binding should deserialize");

        assert!(!binding.pinned);
        assert!(!binding.stale);
        assert_eq!(binding.bound_conversation_ref, binding.conversation_ref);
        assert_eq!(binding.health_state, ProviderHealth::Ready);
        assert_eq!(
            binding.capability_snapshot,
            CapabilitySnapshot {
                supports_follow_up_while_generating: false,
                can_auto_send: true,
                can_capture_full_history: true,
                can_capture_delta: true,
            }
        );
    }

    #[test]
    fn conversation_ref_matches_by_conversation_id_first() {
        let current = ConversationRef {
            conversation_id: Some("chat-a".to_owned()),
            title: None,
            url: Some("https://chatgpt.com/c/chat-a?foo=bar".to_owned()),
            model_label: None,
        };
        let target = ConversationRef {
            conversation_id: Some("chat-a".to_owned()),
            title: None,
            url: Some("https://chatgpt.com/c/chat-b".to_owned()),
            model_label: None,
        };

        assert!(current.matches_target(&target));
    }

    #[test]
    fn conversation_ref_falls_back_to_normalized_url_when_id_missing() {
        let current = ConversationRef {
            conversation_id: None,
            title: None,
            url: Some("https://chatgpt.com/c/chat-a?foo=bar#frag".to_owned()),
            model_label: None,
        };
        let target = ConversationRef {
            conversation_id: None,
            title: None,
            url: Some("https://chatgpt.com/c/chat-a".to_owned()),
            model_label: None,
        };

        assert!(current.matches_target(&target));
    }
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
