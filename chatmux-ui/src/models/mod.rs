//! UI data models.
//!
//! Wire types are imported directly from `chatmux_common`.
//! This module re-exports them for convenience and defines
//! UI-local view models for display-only concerns.

pub mod view_models;

// Re-export canonical wire types from chatmux-common.
// These are the types used in UiCommand/UiEvent serialization.
pub use chatmux_common::{
    // Identifiers
    BindingId, DeliveryCursorId, DiagnosticEventId, DispatchId, EdgePolicyId, ExportProfileId,
    MessageId, RoundId, RunId, TemplateId, WorkspaceId,

    // Core entities
    Workspace, Message, Run, Round, Dispatch, DiagnosticEvent,
    EdgePolicy, DeliveryCursor, ParticipantBinding, Template, ExportProfile,
    WorkspaceSnapshot, RunLedger,

    // Template kind
    TemplateKind,

    // Enums
    ProviderId, ProviderHealth, OrchestrationMode, ContextStrategy,
    CatchUpPolicy, IncrementalPolicy, TruncationPolicy, ApprovalMode,
    BarrierPolicy, TimingPolicy, StopPolicy,
    RunStatus, DispatchOutcome, DiagnosticLevel, BlockingState,
    ExportFormat, ExportLayout, ExportScopePreset, ExportFilterPreset,
    MetadataIncludeFlags, CaptureConfidence, ConversationRef, ProviderControlCapabilities,
    ProviderControlDefaults, ProviderControlSnapshot, ProviderControlState, ProviderConversation,
    ProviderFeatureFlag, ProviderModelOption, ProviderProject, ProviderReasoningOption,
    ProviderStrategy,
    Block, MessageRole,

    // Protocol
    UiCommand, UiEvent,
};
