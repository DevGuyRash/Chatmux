//! Gemini provider adapter.

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
mod dom_contract;

use chatmux_common::{
    AdapterError, AdapterToBackground, BackgroundToAdapter, BlockingState, ConversationRef,
    DiagnosticLevel, Message, MessageId, ProviderAdapter, ProviderHealth, ProviderId,
};
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use chatmux_common::{CaptureConfidence, MessageRole, WorkspaceId};
#[cfg(target_arch = "wasm32")]
use chrono::Utc;

const APP_LANDMARK_SELECTORS: &[&str] = &["chat-app", "main"];
const HISTORY_SELECTOR: &str = "user-query, model-response, dual-model-response, response-container, [data-message-author-role], [data-testid='user-message'], [data-testid='assistant-message']";
const INPUT_SELECTORS: &[&str] = &[
    "div[aria-label='Enter a prompt for Gemini'][role='textbox'][contenteditable='true']",
    "rich-textarea [contenteditable='true']",
];
const SEND_SELECTORS: &[&str] = &["button[aria-label='Send message']"];
// Gemini keeps progress-bar elements mounted after a response completes.  The
// stop control is the reliable live-generation signal; treating any mounted
// progress bar as active leaves dispatches waiting until timeout.
const GENERATING_SELECTORS: &[&str] = &["button[aria-label*='Stop']"];
// Gemini occasionally renders an A/B comparison instead of a single
// model-response. Once both choices and their selection controls exist, the
// turn is complete even if a transient stop control is still mounted.
const DUAL_RESPONSE_COMPLETE_SELECTORS: &[&str] =
    &["dual-model-response response-selection-panel button"];
const LOGIN_SELECTORS: &[&str] = &[
    "button[aria-label*='Sign in']",
    "a[href*='accounts.google.com'][href*='signin']",
];
const RATE_LIMIT_SELECTORS: &[&str] = &["snackbar-container", "[role='alert']"];
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const ASSISTANT_BODY_SELECTOR: &str = "message-content";

#[derive(Debug, Default)]
pub struct GeminiAdapter;

impl ProviderAdapter for GeminiAdapter {
    fn codename(&self) -> ProviderId {
        ProviderId::Gemini
    }
    fn display_name(&self) -> &'static str {
        "Gemini"
    }
    fn structural_probe(&self) -> Result<(), AdapterError> {
        query::structural_probe(APP_LANDMARK_SELECTORS, INPUT_SELECTORS, "Gemini")
    }
    fn health(&self) -> ProviderHealth {
        if let Some(blocking_state) = self.detect_blocking_state() {
            match blocking_state {
                BlockingState::PermissionMissing { .. } => ProviderHealth::PermissionMissing,
                BlockingState::LoginRequired { .. } => ProviderHealth::LoginRequired,
                BlockingState::RateLimited { .. } => ProviderHealth::RateLimited,
                BlockingState::ProviderError { .. } | BlockingState::InputUnavailable { .. } => {
                    ProviderHealth::Blocked
                }
            }
        } else if self.is_generating() {
            ProviderHealth::Generating
        } else {
            ProviderHealth::Ready
        }
    }
    fn inject_input(&self, text: &str) -> Result<(), AdapterError> {
        query::inject_text(INPUT_SELECTORS, text, "Gemini")
    }
    fn send(&self) -> Result<(), AdapterError> {
        query::click_first(SEND_SELECTORS, "Gemini")
    }
    fn is_generating(&self) -> bool {
        !query::exists_any(DUAL_RESPONSE_COMPLETE_SELECTORS)
            && query::exists_visible_any(GENERATING_SELECTORS)
    }
    fn extract_latest_response(&self) -> Result<Message, AdapterError> {
        query::extract_last_message(HISTORY_SELECTOR, ProviderId::Gemini)
    }
    fn extract_full_history(&self) -> Result<Vec<Message>, AdapterError> {
        query::extract_message_list(HISTORY_SELECTOR, ProviderId::Gemini)
    }
    fn extract_incremental_delta(
        &self,
        after_message_id: Option<MessageId>,
    ) -> Result<Vec<Message>, AdapterError> {
        query::extract_incremental(HISTORY_SELECTOR, ProviderId::Gemini, after_message_id)
    }
    fn supports_follow_up_while_generating(&self) -> bool {
        false
    }
    fn detect_blocking_state(&self) -> Option<BlockingState> {
        if query::exists_any(LOGIN_SELECTORS) {
            Some(BlockingState::LoginRequired {
                detail: "Gemini login prompt detected".to_owned(),
            })
        } else if query::matches_text_any(
            RATE_LIMIT_SELECTORS,
            dom_contract::text_indicates_rate_limit,
        ) {
            Some(BlockingState::RateLimited {
                detail: "Gemini rate limiting banner detected".to_owned(),
            })
        } else if query::exists_any(APP_LANDMARK_SELECTORS) && !query::exists_any(INPUT_SELECTORS) {
            Some(BlockingState::InputUnavailable {
                detail: "Gemini prompt editor is unavailable".to_owned(),
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
pub fn bootstrap_gemini_content_script() -> Result<(), JsValue> {
    GeminiAdapter
        .structural_probe()
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn handle_adapter_command_json(payload: String) -> Result<JsValue, JsValue> {
    let command: BackgroundToAdapter =
        serde_json::from_str(&payload).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let adapter = GeminiAdapter;
    let events = execute_command(&adapter, command).unwrap_or_else(|error| {
        vec![AdapterToBackground::CommandFailed {
            provider: ProviderId::Gemini,
            level: DiagnosticLevel::Critical,
            detail: error.to_string(),
        }]
    });
    serde_wasm_bindgen::to_value(&events).map_err(|error| JsValue::from_str(&error.to_string()))
}

fn execute_command(
    adapter: &GeminiAdapter,
    command: BackgroundToAdapter,
) -> Result<Vec<AdapterToBackground>, AdapterError> {
    Ok(match command {
        BackgroundToAdapter::StructuralProbe => match adapter.structural_probe() {
            Ok(()) => vec![AdapterToBackground::StructuralProbePassed {
                provider: ProviderId::Gemini,
            }],
            Err(error) => vec![AdapterToBackground::StructuralProbeFailed {
                provider: ProviderId::Gemini,
                detail: error.to_string(),
            }],
        },
        BackgroundToAdapter::GetHealth => vec![AdapterToBackground::HealthReport {
            provider: ProviderId::Gemini,
            health: adapter.health(),
        }],
        BackgroundToAdapter::InjectInput { text } => {
            adapter.inject_input(&text)?;
            vec![AdapterToBackground::HealthReport {
                provider: ProviderId::Gemini,
                health: adapter.health(),
            }]
        }
        BackgroundToAdapter::Send => {
            adapter.send()?;
            vec![AdapterToBackground::HealthReport {
                provider: ProviderId::Gemini,
                health: adapter.health(),
            }]
        }
        BackgroundToAdapter::ExtractLatestResponse => {
            vec![AdapterToBackground::MessagesCaptured {
                provider: ProviderId::Gemini,
                messages: vec![adapter.extract_latest_response()?],
            }]
        }
        BackgroundToAdapter::ExtractFullHistory => vec![AdapterToBackground::MessagesCaptured {
            provider: ProviderId::Gemini,
            messages: adapter.extract_full_history()?,
        }],
        BackgroundToAdapter::ExtractIncrementalDelta { after_message_id } => {
            vec![AdapterToBackground::MessagesCaptured {
                provider: ProviderId::Gemini,
                messages: adapter.extract_incremental_delta(after_message_id)?,
            }]
        }
        BackgroundToAdapter::DetectBlockingState => {
            if let Some(blocking_state) = adapter.detect_blocking_state() {
                vec![AdapterToBackground::BlockingStateDetected {
                    provider: ProviderId::Gemini,
                    blocking_state,
                }]
            } else {
                vec![AdapterToBackground::HealthReport {
                    provider: ProviderId::Gemini,
                    health: adapter.health(),
                }]
            }
        }
        BackgroundToAdapter::GetConversationRef => {
            vec![AdapterToBackground::ConversationRefDiscovered {
                provider: ProviderId::Gemini,
                conversation_ref: adapter.conversation_ref(),
            }]
        }
        BackgroundToAdapter::GetProviderSnapshot => {
            vec![AdapterToBackground::ProviderControlSnapshotCaptured {
                provider: ProviderId::Gemini,
                snapshot: query::provider_snapshot(),
            }]
        }
        BackgroundToAdapter::CreateProject { .. }
        | BackgroundToAdapter::SelectProject { .. }
        | BackgroundToAdapter::CreateConversation { .. }
        | BackgroundToAdapter::SelectConversation { .. }
        | BackgroundToAdapter::SetModel { .. }
        | BackgroundToAdapter::SetReasoning { .. }
        | BackgroundToAdapter::SetFeatureFlag { .. } => {
            return Err(AdapterError::Unsupported {
                detail: "Gemini does not expose this provider control through its current web UI contract".to_owned(),
            });
        }
    })
}

mod query {
    use super::*;

    pub fn provider_snapshot() -> chatmux_common::ProviderControlSnapshot {
        let reference = conversation_ref();
        chatmux_common::ProviderControlSnapshot {
            provider: ProviderId::Gemini,
            capabilities: chatmux_common::ProviderControlCapabilities {
                supports_sync: true,
                ..chatmux_common::ProviderControlCapabilities::default()
            },
            state: chatmux_common::ProviderControlState {
                conversation_id: reference
                    .as_ref()
                    .and_then(|item| item.conversation_id.clone()),
                conversation_title: reference.as_ref().and_then(|item| item.title.clone()),
                model_label: reference.as_ref().and_then(|item| item.model_label.clone()),
                last_strategy: Some(chatmux_common::ProviderStrategy::Dom),
                ..chatmux_common::ProviderControlState::default()
            },
            projects: Vec::new(),
            conversations: Vec::new(),
            models: Vec::new(),
            reasoning_options: Vec::new(),
            feature_flags: Vec::new(),
        }
    }

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
        selector: &str,
        provider: ProviderId,
    ) -> Result<Message, AdapterError> {
        extract_message_list(selector, provider)?
            .into_iter()
            .rev()
            .find(|message| message.role == chatmux_common::MessageRole::Assistant)
            .ok_or(AdapterError::NotFound {
                detail: "no assistant response found".to_owned(),
            })
    }

    pub fn extract_incremental(
        selector: &str,
        provider: ProviderId,
        after_message_id: Option<MessageId>,
    ) -> Result<Vec<Message>, AdapterError> {
        let messages = extract_message_list(selector, provider)?;
        if let Some(after_message_id) = after_message_id {
            let Some(index) = messages
                .iter()
                .position(|message| message.id == after_message_id)
            else {
                return Err(AdapterError::CaptureUncertain {
                    detail: "Gemini capture baseline is no longer visible; sync the transcript before retrying"
                        .to_owned(),
                });
            };
            Ok(messages.into_iter().skip(index + 1).collect())
        } else {
            Ok(messages)
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn extract_message_list(
        selector: &str,
        provider: ProviderId,
    ) -> Result<Vec<Message>, AdapterError> {
        let document = document()?;
        let nodes =
            document
                .query_selector_all(selector)
                .map_err(|error| AdapterError::Unsupported {
                    detail: format!("query failed: {error:?}"),
                })?;
        let mut messages = Vec::new();
        for index in 0..nodes.length() {
            if let Some(node) = nodes.item(index) {
                if let Some(element) = node.dyn_ref::<web_sys::Element>() {
                    if let Some(message) = message_from_element(provider, element, index as usize)?
                    {
                        messages.push(message);
                    }
                }
            }
        }
        Ok(messages)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn extract_message_list(_: &str, _: ProviderId) -> Result<Vec<Message>, AdapterError> {
        Ok(vec![])
    }

    #[cfg(target_arch = "wasm32")]
    pub fn inject_text(selectors: &[&str], text: &str, label: &str) -> Result<(), AdapterError> {
        use wasm_bindgen::JsCast;
        let document = document()?;
        for selector in selectors {
            if let Ok(Some(node)) = document.query_selector(selector) {
                let Some(element) = node.dyn_ref::<web_sys::HtmlElement>() else {
                    continue;
                };
                element.focus().map_err(|error| AdapterError::SendFailed {
                    detail: format!("failed to focus {label} prompt editor: {error:?}"),
                })?;
                dispatch_input_event(element, "beforeinput", text, "insertText")?;
                if let Some(textarea) = node.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                    textarea.set_value(text);
                } else if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
                    input.set_value(text);
                } else {
                    replace_contenteditable_text(&document, element, text)?;
                }
                dispatch_input_event(element, "input", text, "insertText")?;
                dispatch_change_event(element)?;

                let actual = readable_input_text(&node);
                if normalize_text(&actual) != normalize_text(text) {
                    return Err(AdapterError::SendFailed {
                        detail: format!(
                            "{label} prompt editor did not retain the injected text; focus the provider tab and retry"
                        ),
                    });
                }
                return Ok(());
            }
        }
        Err(AdapterError::NotFound {
            detail: format!("no writable {label} input found"),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn inject_text(_: &[&str], _: &str, _: &str) -> Result<(), AdapterError> {
        Err(AdapterError::Unsupported {
            detail: "Gemini input injection requires wasm32".to_owned(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn click_first(selectors: &[&str], label: &str) -> Result<(), AdapterError> {
        let document = document()?;
        let composer = first_element(&document, INPUT_SELECTORS).ok_or(AdapterError::NotFound {
            detail: format!("no writable {label} input found"),
        })?;
        if let Some(element) = find_scoped_action(&composer, selectors)? {
            element.click();
            return Ok(());
        }
        Err(AdapterError::NotFound {
            detail: format!("no visible and enabled {label} send control found"),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn click_first(_: &[&str], _: &str) -> Result<(), AdapterError> {
        Err(AdapterError::Unsupported {
            detail: "Gemini send action requires wasm32".to_owned(),
        })
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

    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
    pub fn exists_visible_any(selectors: &[&str]) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            if let Ok(document) = document() {
                for selector in selectors {
                    let Ok(nodes) = document.query_selector_all(selector) else {
                        continue;
                    };
                    for index in 0..nodes.length() {
                        let Some(node) = nodes.item(index) else {
                            continue;
                        };
                        let Some(element) = node.dyn_ref::<web_sys::HtmlElement>() else {
                            continue;
                        };
                        if element_is_actionable(element) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn matches_text_any(selectors: &[&str], classifier: fn(&str) -> bool) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(document) = document() {
                for selector in selectors {
                    let Ok(nodes) = document.query_selector_all(selector) else {
                        continue;
                    };
                    for index in 0..nodes.length() {
                        let Some(node) = nodes.item(index) else {
                            continue;
                        };
                        if classifier(&node.text_content().unwrap_or_default()) {
                            return true;
                        }
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = (selectors, classifier);
        false
    }

    pub fn conversation_ref() -> Option<ConversationRef> {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window()?;
            let pathname = window.location().pathname().ok()?;
            let conversation_id = dom_contract::conversation_id_from_path(&pathname)?;
            return Some(ConversationRef {
                conversation_id: Some(conversation_id),
                title: window
                    .document()
                    .map(|document| document.title())
                    .filter(|title| !title.trim().is_empty()),
                url: window.location().href().ok(),
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
    fn message_from_element(
        provider: ProviderId,
        element: &web_sys::Element,
        dom_index: usize,
    ) -> Result<Option<Message>, AdapterError> {
        // Current Gemini markup nests response-container inside model-response.
        // Retain the inner tag as a compatibility fallback without emitting the
        // same assistant turn twice when both wrappers are present.
        if element
            .tag_name()
            .eq_ignore_ascii_case("response-container")
            && element.closest("model-response").ok().flatten().is_some()
        {
            return Ok(None);
        }
        let role = dom_contract::classify_role(
            &element.tag_name(),
            element.get_attribute("data-testid").as_deref(),
            element.get_attribute("data-message-author-role").as_deref(),
        );
        let Some(role) = role else {
            return Ok(None);
        };
        let text_element = if role == MessageRole::Assistant {
            element
                .query_selector(ASSISTANT_BODY_SELECTOR)
                .ok()
                .flatten()
                .unwrap_or_else(|| element.clone())
        } else {
            element.clone()
        };
        let text = text_element
            .text_content()
            .unwrap_or_default()
            .trim()
            .to_owned();
        if text.is_empty() {
            return Ok(None);
        }
        let source_key = ["data-message-id", "data-turn-id", "id"]
            .iter()
            .find_map(|attribute| element.get_attribute(attribute))
            .filter(|value| !value.trim().is_empty());
        let conversation_key = conversation_ref().and_then(|reference| reference.conversation_id);
        let message_id = dom_contract::stable_message_id(dom_contract::FingerprintInput {
            conversation_key: conversation_key.as_deref(),
            source_key: source_key.as_deref(),
            role,
            dom_index,
            text: &text,
        });
        let participant_id = match role {
            MessageRole::User => ProviderId::User,
            MessageRole::Assistant => provider,
            MessageRole::System => ProviderId::System,
        };
        let raw_response_text = (role == MessageRole::Assistant).then(|| text.clone());
        let capture_confidence = if !exists_any(DUAL_RESPONSE_COMPLETE_SELECTORS)
            && exists_visible_any(GENERATING_SELECTORS)
        {
            CaptureConfidence::Uncertain
        } else {
            CaptureConfidence::Certain
        };
        Ok(Some(Message {
            id: message_id,
            workspace_id: WorkspaceId::new(),
            participant_id,
            role,
            round: None,
            parent_message_id: None,
            child_message_ids: Vec::new(),
            branch_index: None,
            timestamp: Utc::now(),
            body_text: text.clone(),
            body_blocks: vec![chatmux_common::Block::Paragraph { text }],
            source_binding_id: None,
            dispatch_id: None,
            raw_response_text,
            network_capture: None,
            tags: vec![],
            capture_confidence,
        }))
    }

    #[cfg(target_arch = "wasm32")]
    fn first_element(document: &web_sys::Document, selectors: &[&str]) -> Option<web_sys::Element> {
        selectors
            .iter()
            .find_map(|selector| document.query_selector(selector).ok().flatten())
    }

    #[cfg(target_arch = "wasm32")]
    fn find_scoped_action(
        composer: &web_sys::Element,
        selectors: &[&str],
    ) -> Result<Option<web_sys::HtmlElement>, AdapterError> {
        use wasm_bindgen::JsCast;
        let mut scope = composer
            .closest("form")
            .map_err(|error| AdapterError::Unsupported {
                detail: format!("failed to locate Gemini composer form: {error:?}"),
            })?
            .or_else(|| composer.parent_element());
        for _ in 0..6 {
            let Some(current_scope) = scope else {
                break;
            };
            for selector in selectors {
                if let Ok(candidates) = current_scope.query_selector_all(selector) {
                    for index in 0..candidates.length() {
                        let Some(candidate) = candidates.item(index) else {
                            continue;
                        };
                        if let Some(element) = candidate.dyn_ref::<web_sys::HtmlElement>() {
                            if element_is_actionable(element) {
                                return Ok(Some(element.clone()));
                            }
                        }
                    }
                }
            }
            scope = current_scope.parent_element();
        }
        Ok(None)
    }

    #[cfg(target_arch = "wasm32")]
    fn element_is_actionable(element: &web_sys::HtmlElement) -> bool {
        use wasm_bindgen::JsCast;
        let enabled = element
            .dyn_ref::<web_sys::HtmlButtonElement>()
            .is_none_or(|button| !button.disabled())
            && element.get_attribute("aria-disabled").as_deref() != Some("true");
        // Gemini collapses the layout box for controls in a background tab,
        // even though the current composer's enabled send button remains the
        // authoritative action. Attribute-level hiding still excludes stale
        // or intentionally hidden controls; the layout-size requirement only
        // applies while the document is visible.
        let document_hidden = web_sys::window()
            .and_then(|window| window.document())
            .is_some_and(|document| document.hidden());
        let visible = element.get_attribute("hidden").is_none()
            && element.get_attribute("aria-hidden").as_deref() != Some("true")
            && (document_hidden || element.offset_width() > 0 || element.offset_height() > 0);
        enabled && visible
    }

    #[cfg(target_arch = "wasm32")]
    fn replace_contenteditable_text(
        document: &web_sys::Document,
        element: &web_sys::HtmlElement,
        text: &str,
    ) -> Result<(), AdapterError> {
        use wasm_bindgen::JsCast;
        let range = document
            .create_range()
            .map_err(|error| AdapterError::SendFailed {
                detail: format!("failed to select Gemini prompt text: {error:?}"),
            })?;
        range
            .select_node_contents(element.unchecked_ref())
            .map_err(|error| AdapterError::SendFailed {
                detail: format!("failed to select Gemini prompt contents: {error:?}"),
            })?;
        if let Ok(Some(selection)) = document.get_selection() {
            selection
                .remove_all_ranges()
                .and_then(|()| selection.add_range(&range))
                .map_err(|error| AdapterError::SendFailed {
                    detail: format!("failed to replace Gemini prompt selection: {error:?}"),
                })?;
        }
        let inserted = document
            .dyn_ref::<web_sys::HtmlDocument>()
            .and_then(|html_document| {
                html_document
                    .exec_command_with_show_ui_and_value("insertText", false, text)
                    .ok()
            })
            .unwrap_or(false);
        if !inserted {
            element.set_text_content(Some(text));
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn dispatch_input_event(
        element: &web_sys::HtmlElement,
        event_name: &str,
        text: &str,
        input_type: &str,
    ) -> Result<(), AdapterError> {
        let init = web_sys::InputEventInit::new();
        init.set_bubbles(true);
        init.set_cancelable(event_name == "beforeinput");
        init.set_composed(true);
        init.set_data(Some(text));
        init.set_input_type(input_type);
        let event =
            web_sys::InputEvent::new_with_event_init_dict(event_name, &init).map_err(|error| {
                AdapterError::SendFailed {
                    detail: format!("failed to create Gemini {event_name} event: {error:?}"),
                }
            })?;
        element
            .dispatch_event(&event)
            .map_err(|error| AdapterError::SendFailed {
                detail: format!("failed to dispatch Gemini {event_name} event: {error:?}"),
            })?;
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn dispatch_change_event(element: &web_sys::HtmlElement) -> Result<(), AdapterError> {
        let event = web_sys::Event::new("change").map_err(|error| AdapterError::SendFailed {
            detail: format!("failed to create Gemini change event: {error:?}"),
        })?;
        element
            .dispatch_event(&event)
            .map_err(|error| AdapterError::SendFailed {
                detail: format!("failed to dispatch Gemini change event: {error:?}"),
            })?;
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn readable_input_text(element: &web_sys::Element) -> String {
        use wasm_bindgen::JsCast;
        if let Some(textarea) = element.dyn_ref::<web_sys::HtmlTextAreaElement>() {
            textarea.value()
        } else if let Some(input) = element.dyn_ref::<web_sys::HtmlInputElement>() {
            input.value()
        } else if let Some(editable) = element.dyn_ref::<web_sys::HtmlElement>() {
            editable.inner_text()
        } else {
            element.text_content().unwrap_or_default()
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn normalize_text(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selector_sets_are_not_empty() {
        assert!(!APP_LANDMARK_SELECTORS.is_empty());
        assert!(!INPUT_SELECTORS.is_empty());
        assert_eq!(ASSISTANT_BODY_SELECTOR, "message-content");
        assert!(HISTORY_SELECTOR.contains("dual-model-response"));
        assert!(!DUAL_RESPONSE_COMPLETE_SELECTORS.is_empty());
    }
}
