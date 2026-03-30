//! UI data models.
//!
//! Wire types are imported directly from `chatmux_common`.
//! This module re-exports them for convenience and defines
//! UI-local view models for display-only concerns.

pub mod view_models;

// Re-export canonical wire types from chatmux-common.
// These are the types used in UiCommand/UiEvent serialization.
pub use chatmux_common::{
    ApprovalMode,
    BarrierPolicy,
    // Identifiers
    BindingId,
    Block,
    BlockingState,
    CaptureConfidence,
    CatchUpPolicy,
    ContextStrategy,
    ConversationRef,
    DeliveryCursor,
    DeliveryCursorId,
    DiagnosticArtifactKind,
    DiagnosticArtifactRef,

    DiagnosticEvent,
    DiagnosticEventId,
    DiagnosticLevel,
    DiagnosticScope,
    DiagnosticSource,
    DiagnosticsDetailLevel,
    DiagnosticsQuery,
    DiagnosticsSearchMode,
    DiagnosticsSnapshot,
    Dispatch,
    DispatchId,
    DispatchOutcome,
    EdgePolicy,
    EdgePolicyId,
    ExportFilterPreset,
    ExportFormat,
    ExportLayout,
    ExportProfile,
    ExportProfileId,
    ExportScopePreset,
    IncrementalPolicy,
    Message,
    MessageId,
    MessageRole,

    MetadataIncludeFlags,
    OrchestrationMode,
    ParticipantBinding,
    ProviderControlCapabilities,
    ProviderControlDefaults,
    ProviderControlSnapshot,
    ProviderControlState,
    ProviderConversation,
    ProviderFeatureFlag,
    ProviderHealth,
    // Enums
    ProviderId,
    ProviderModelOption,
    ProviderNetworkCapture,
    ProviderProject,
    ProviderReasoningOption,
    ProviderStrategy,
    ProviderTabCandidate,
    Round,
    RoundId,
    Run,
    RunId,
    RunLedger,
    RunStatus,
    StopPolicy,
    Template,
    TemplateId,
    // Template kind
    TemplateKind,

    TimingPolicy,
    TruncationPolicy,
    // Protocol
    UiCommand,
    UiEvent,
    // Core entities
    Workspace,
    WorkspaceDiagnosticsSummary,
    WorkspaceId,

    WorkspaceSnapshot,
};
