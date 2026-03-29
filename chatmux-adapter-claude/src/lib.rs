//! Claude provider adapter.

use chatmux_common::{
    AdapterError, AdapterToBackground, BackgroundToAdapter, BlockingState, ConversationRef,
    DiagnosticLevel, Message, MessageId, ProviderAdapter, ProviderHealth, ProviderId,
};
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use chatmux_common::{CaptureConfidence, MessageRole, WorkspaceId};
#[cfg(target_arch = "wasm32")]
use chrono::Utc;

const TRANSCRIPT_SELECTORS: &[&str] = &["main", "article", "[data-testid*='message']"];
const RESPONSE_SELECTORS: &[&str] = &["article", "[data-testid*='assistant']", ".prose"];
const INPUT_SELECTORS: &[&str] = &[
    "textarea",
    "div[contenteditable='true']",
    "[contenteditable='true']",
];
const SEND_SELECTORS: &[&str] = &["button[aria-label*='Send']", "button[type='submit']"];
const GENERATING_SELECTORS: &[&str] = &["button[aria-label*='Stop']", "[data-state='streaming']"];
const LOGIN_SELECTORS: &[&str] = &["input[type='email']", "form"];
const RATE_LIMIT_SELECTORS: &[&str] = &["[role='alert']", ".text-danger"];

#[derive(Debug, Default)]
pub struct ClaudeAdapter;

impl ProviderAdapter for ClaudeAdapter {
    fn codename(&self) -> ProviderId {
        ProviderId::Claude
    }
    fn display_name(&self) -> &'static str {
        "Claude"
    }
    fn structural_probe(&self) -> Result<(), AdapterError> {
        query::structural_probe(TRANSCRIPT_SELECTORS, INPUT_SELECTORS, "Claude")
    }
    fn health(&self) -> ProviderHealth {
        if self.detect_blocking_state().is_some() {
            ProviderHealth::Blocked
        } else if self.is_generating() {
            ProviderHealth::Generating
        } else {
            ProviderHealth::Ready
        }
    }
    fn inject_input(&self, text: &str) -> Result<(), AdapterError> {
        query::inject_text(INPUT_SELECTORS, text, "Claude")
    }
    fn send(&self) -> Result<(), AdapterError> {
        query::click_first(SEND_SELECTORS, "Claude")
    }
    fn is_generating(&self) -> bool {
        query::exists_any(GENERATING_SELECTORS)
    }
    fn extract_latest_response(&self) -> Result<Message, AdapterError> {
        query::extract_last_message(RESPONSE_SELECTORS, ProviderId::Claude)
    }
    fn extract_full_history(&self) -> Result<Vec<Message>, AdapterError> {
        query::extract_message_list(RESPONSE_SELECTORS, ProviderId::Claude)
    }
    fn extract_incremental_delta(
        &self,
        after_message_id: Option<MessageId>,
    ) -> Result<Vec<Message>, AdapterError> {
        query::extract_incremental(RESPONSE_SELECTORS, ProviderId::Claude, after_message_id)
    }
    fn supports_follow_up_while_generating(&self) -> bool {
        true
    }
    fn detect_blocking_state(&self) -> Option<BlockingState> {
        if query::exists_any(LOGIN_SELECTORS) {
            Some(BlockingState::LoginRequired {
                detail: "Claude login prompt detected".to_owned(),
            })
        } else if query::exists_any(RATE_LIMIT_SELECTORS) {
            Some(BlockingState::RateLimited {
                detail: "Claude rate limiting banner detected".to_owned(),
            })
        } else {
            None
        }
    }
    fn conversation_ref(&self) -> Option<ConversationRef> {
        query::conversation_ref()
    }
}

#[wasm_bindgen]
pub fn bootstrap_claude_content_script() -> Result<(), JsValue> {
    ClaudeAdapter
        .structural_probe()
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn handle_adapter_command_json(payload: String) -> Result<JsValue, JsValue> {
    let command: BackgroundToAdapter =
        serde_json::from_str(&payload).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let adapter = ClaudeAdapter;
    let events = execute_command(&adapter, command).unwrap_or_else(|error| {
        vec![AdapterToBackground::CommandFailed {
            provider: ProviderId::Claude,
            level: DiagnosticLevel::Critical,
            detail: error.to_string(),
        }]
    });
    serde_wasm_bindgen::to_value(&events).map_err(|error| JsValue::from_str(&error.to_string()))
}

fn execute_command(
    adapter: &ClaudeAdapter,
    command: BackgroundToAdapter,
) -> Result<Vec<AdapterToBackground>, AdapterError> {
    Ok(match command {
        BackgroundToAdapter::StructuralProbe => match adapter.structural_probe() {
            Ok(()) => vec![AdapterToBackground::StructuralProbePassed {
                provider: ProviderId::Claude,
            }],
            Err(error) => vec![AdapterToBackground::StructuralProbeFailed {
                provider: ProviderId::Claude,
                detail: error.to_string(),
            }],
        },
        BackgroundToAdapter::GetHealth => vec![AdapterToBackground::HealthReport {
            provider: ProviderId::Claude,
            health: adapter.health(),
        }],
        BackgroundToAdapter::InjectInput { text } => {
            adapter.inject_input(&text)?;
            vec![AdapterToBackground::HealthReport {
                provider: ProviderId::Claude,
                health: adapter.health(),
            }]
        }
        BackgroundToAdapter::Send => {
            adapter.send()?;
            vec![AdapterToBackground::HealthReport {
                provider: ProviderId::Claude,
                health: adapter.health(),
            }]
        }
        BackgroundToAdapter::ExtractLatestResponse => {
            vec![AdapterToBackground::MessagesCaptured {
                provider: ProviderId::Claude,
                messages: vec![adapter.extract_latest_response()?],
            }]
        }
        BackgroundToAdapter::ExtractFullHistory => vec![AdapterToBackground::MessagesCaptured {
            provider: ProviderId::Claude,
            messages: adapter.extract_full_history()?,
        }],
        BackgroundToAdapter::ExtractIncrementalDelta { after_message_id } => {
            vec![AdapterToBackground::MessagesCaptured {
                provider: ProviderId::Claude,
                messages: adapter.extract_incremental_delta(after_message_id)?,
            }]
        }
        BackgroundToAdapter::DetectBlockingState => {
            if let Some(blocking_state) = adapter.detect_blocking_state() {
                vec![AdapterToBackground::BlockingStateDetected {
                    provider: ProviderId::Claude,
                    blocking_state,
                }]
            } else {
                vec![AdapterToBackground::HealthReport {
                    provider: ProviderId::Claude,
                    health: adapter.health(),
                }]
            }
        }
        BackgroundToAdapter::GetConversationRef => {
            vec![AdapterToBackground::ConversationRefDiscovered {
                provider: ProviderId::Claude,
                conversation_ref: adapter.conversation_ref(),
            }]
        }
    })
}

mod query {
    use super::*;
    pub fn structural_probe(
        transcript: &[&str],
        inputs: &[&str],
        label: &str,
    ) -> Result<(), AdapterError> {
        if exists_any(transcript) && exists_any(inputs) {
            Ok(())
        } else {
            Err(AdapterError::DomMismatch {
                detail: format!("{label} landmarks were not found"),
            })
        }
    }
    pub fn extract_last_message(
        selectors: &[&str],
        provider: ProviderId,
    ) -> Result<Message, AdapterError> {
        let mut messages = extract_message_list(selectors, provider)?;
        messages.pop().ok_or(AdapterError::NotFound {
            detail: "no assistant response found".to_owned(),
        })
    }
    pub fn extract_incremental(
        selectors: &[&str],
        provider: ProviderId,
        after_message_id: Option<MessageId>,
    ) -> Result<Vec<Message>, AdapterError> {
        let messages = extract_message_list(selectors, provider)?;
        if let Some(after_message_id) = after_message_id {
            Ok(messages
                .into_iter()
                .filter(|message| message.id != after_message_id)
                .collect())
        } else {
            Ok(messages)
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn extract_message_list(
        selectors: &[&str],
        provider: ProviderId,
    ) -> Result<Vec<Message>, AdapterError> {
        let document = document()?;
        let nodes = document
            .query_selector_all(&selectors.join(", "))
            .map_err(|error| AdapterError::Unsupported {
                detail: format!("query failed: {error:?}"),
            })?;
        let mut messages = Vec::new();
        for index in 0..nodes.length() {
            if let Some(node) = nodes.item(index) {
                let text = node.text_content().unwrap_or_default().trim().to_owned();
                if !text.is_empty() {
                    messages.push(message_from_text(provider, text));
                }
            }
        }
        Ok(messages)
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn extract_message_list(_: &[&str], _: ProviderId) -> Result<Vec<Message>, AdapterError> {
        Ok(vec![])
    }
    #[cfg(target_arch = "wasm32")]
    pub fn inject_text(selectors: &[&str], text: &str, label: &str) -> Result<(), AdapterError> {
        use wasm_bindgen::JsCast;
        let document = document()?;
        for selector in selectors {
            if let Ok(Some(node)) = document.query_selector(selector) {
                if let Some(textarea) = node.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                    textarea.set_value(text);
                    return Ok(());
                }
                if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
                    input.set_value(text);
                    return Ok(());
                }
                if let Some(element) = node.dyn_ref::<web_sys::HtmlElement>() {
                    element.set_text_content(Some(text));
                    return Ok(());
                }
            }
        }
        Err(AdapterError::NotFound {
            detail: format!("no writable {label} input found"),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn inject_text(_: &[&str], _: &str, _: &str) -> Result<(), AdapterError> {
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    pub fn click_first(selectors: &[&str], label: &str) -> Result<(), AdapterError> {
        use wasm_bindgen::JsCast;
        let document = document()?;
        for selector in selectors {
            if let Ok(Some(node)) = document.query_selector(selector) {
                if let Some(element) = node.dyn_ref::<web_sys::HtmlElement>() {
                    element.click();
                    return Ok(());
                }
            }
        }
        Err(AdapterError::NotFound {
            detail: format!("no {label} send control found"),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn click_first(_: &[&str], _: &str) -> Result<(), AdapterError> {
        Ok(())
    }
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
    pub fn exists_any(selectors: &[&str]) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(document) = document() {
                return selectors
                    .iter()
                    .any(|selector| document.query_selector(selector).ok().flatten().is_some());
            }
        }
        false
    }
    pub fn conversation_ref() -> Option<ConversationRef> {
        #[cfg(target_arch = "wasm32")]
        {
            let pathname = web_sys::window()?.location().pathname().ok()?;
            return Some(ConversationRef {
                conversation_id: pathname.split('/').next_back().map(str::to_owned),
                title: None,
                model_label: None,
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None
        }
    }
    #[cfg(target_arch = "wasm32")]
    fn document() -> Result<web_sys::Document, AdapterError> {
        web_sys::window()
            .and_then(|window| window.document())
            .ok_or(AdapterError::DomMismatch {
                detail: "document unavailable".to_owned(),
            })
    }
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn message_from_text(provider: ProviderId, text: String) -> Message {
        Message {
            id: MessageId::new(),
            workspace_id: WorkspaceId::new(),
            participant_id: provider,
            role: MessageRole::Assistant,
            round: None,
            timestamp: Utc::now(),
            body_text: text.clone(),
            body_blocks: vec![chatmux_common::Block::Paragraph { text }],
            source_binding_id: None,
            dispatch_id: None,
            raw_capture_ref: None,
            tags: vec![],
            capture_confidence: CaptureConfidence::Certain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn claude_is_follow_up_capable() {
        assert!(ClaudeAdapter.supports_follow_up_while_generating());
    }
}
