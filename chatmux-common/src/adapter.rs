//! Shared adapter contract and adapter-facing support types.

use crate::{
    AdapterError, BindingId, BlockingState, CaptureConfidence, ConversationRef, Message, MessageId,
    ProviderHealth, ProviderId,
};

pub trait ProviderAdapter {
    fn codename(&self) -> ProviderId;
    fn display_name(&self) -> &'static str;
    fn structural_probe(&self) -> Result<(), AdapterError>;
    fn health(&self) -> ProviderHealth;
    fn inject_input(&self, text: &str) -> Result<(), AdapterError>;
    fn send(&self) -> Result<(), AdapterError>;
    fn is_generating(&self) -> bool;
    fn extract_latest_response(&self) -> Result<Message, AdapterError>;
    fn extract_full_history(&self) -> Result<Vec<Message>, AdapterError>;
    fn extract_incremental_delta(
        &self,
        after_message_id: Option<MessageId>,
    ) -> Result<Vec<Message>, AdapterError>;
    fn supports_follow_up_while_generating(&self) -> bool;
    fn detect_blocking_state(&self) -> Option<BlockingState>;
    fn conversation_ref(&self) -> Option<ConversationRef>;
}

#[derive(Debug, Clone)]
pub struct AdapterSnapshot {
    pub binding_id: Option<BindingId>,
    pub health: ProviderHealth,
    pub blocking_state: Option<BlockingState>,
    pub conversation_ref: Option<ConversationRef>,
    pub latest_capture_confidence: CaptureConfidence,
}
