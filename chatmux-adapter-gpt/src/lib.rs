//! ChatGPT provider adapter.

use chatmux_common::{
    AdapterError, AdapterToBackground, BackgroundToAdapter, BlockingState, ConversationRef,
    DiagnosticLevel, Message, MessageId, ProviderAdapter, ProviderHealth, ProviderId,
};
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use chatmux_common::{
    CaptureConfidence, MessageRole, ProviderControlCapabilities, ProviderControlSnapshot,
    ProviderControlState, ProviderConversation, ProviderFeatureFlag, ProviderModelOption,
    ProviderNetworkCapture, ProviderProject, ProviderReasoningOption, ProviderStrategy,
    WorkspaceId,
};
#[cfg(target_arch = "wasm32")]
use chrono::Utc;
#[cfg(target_arch = "wasm32")]
use js_sys::Reflect;
#[cfg(target_arch = "wasm32")]
use std::collections::BTreeMap;
#[cfg(target_arch = "wasm32")]
use uuid::Uuid;

const TRANSCRIPT_SELECTORS: &[&str] = &["main", "[data-message-author-role]", "article"];
const HISTORY_SELECTORS: &[&str] = &["[data-message-author-role]"];
const RESPONSE_SELECTORS: &[&str] = &[
    "[data-message-author-role='assistant']",
    "article[data-testid*='conversation-turn']",
];
const INPUT_SELECTORS: &[&str] = &["#prompt-textarea", "textarea", "[contenteditable='true']"];
const SEND_SELECTORS: &[&str] = &[
    "button[data-testid='send-button']",
    "button[aria-label='Send prompt']",
    "button[aria-label*='Send']",
];
const GENERATING_SELECTORS: &[&str] =
    &["button[aria-label*='Stop']", "[data-testid='stop-button']"];
const LOGIN_SELECTORS: &[&str] = &[
    "form[data-provider='auth0']",
    "button[data-testid='login-button']",
];
const RATE_LIMIT_SELECTORS: &[&str] = &["[role='alert']", ".text-red-500"];
const RATE_LIMIT_TEXT_PATTERNS: &[&str] = &[
    "rate limit",
    "too many requests",
    "too many messages",
    "try again later",
    "unusual activity",
    "our systems are a bit busy",
    "you've reached",
];

#[derive(Debug, Default)]
pub struct GptAdapter;

impl ProviderAdapter for GptAdapter {
    fn codename(&self) -> ProviderId {
        ProviderId::Gpt
    }

    fn display_name(&self) -> &'static str {
        "ChatGPT"
    }

    fn structural_probe(&self) -> Result<(), AdapterError> {
        query::structural_probe(TRANSCRIPT_SELECTORS, INPUT_SELECTORS)
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
        query::inject_text(INPUT_SELECTORS, text)
    }

    fn send(&self) -> Result<(), AdapterError> {
        query::click_first(SEND_SELECTORS)
    }

    fn is_generating(&self) -> bool {
        query::exists_any(GENERATING_SELECTORS)
    }

    fn extract_latest_response(&self) -> Result<Message, AdapterError> {
        query::extract_last_message(RESPONSE_SELECTORS, ProviderId::Gpt)
    }

    fn extract_full_history(&self) -> Result<Vec<Message>, AdapterError> {
        query::extract_message_list(HISTORY_SELECTORS, ProviderId::Gpt)
    }

    fn extract_incremental_delta(
        &self,
        after_message_id: Option<MessageId>,
    ) -> Result<Vec<Message>, AdapterError> {
        query::extract_incremental(RESPONSE_SELECTORS, ProviderId::Gpt, after_message_id)
    }

    fn supports_follow_up_while_generating(&self) -> bool {
        false
    }

    fn detect_blocking_state(&self) -> Option<BlockingState> {
        if query::exists_any(LOGIN_SELECTORS) {
            Some(BlockingState::LoginRequired {
                detail: "ChatGPT login prompt detected".to_owned(),
            })
        } else if query::matches_text_any(RATE_LIMIT_SELECTORS, RATE_LIMIT_TEXT_PATTERNS) {
            Some(BlockingState::RateLimited {
                detail: "ChatGPT blocking banner detected".to_owned(),
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
pub fn bootstrap_gpt_content_script() -> Result<(), JsValue> {
    GptAdapter
        .structural_probe()
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn handle_adapter_command_json(payload: String) -> Result<JsValue, JsValue> {
    let command: BackgroundToAdapter =
        serde_json::from_str(&payload).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let adapter = GptAdapter;
    let events = execute_command(&adapter, command).unwrap_or_else(|error| {
        vec![AdapterToBackground::CommandFailed {
            provider: ProviderId::Gpt,
            level: DiagnosticLevel::Critical,
            detail: error.to_string(),
        }]
    });
    serde_wasm_bindgen::to_value(&events).map_err(|error| JsValue::from_str(&error.to_string()))
}

fn execute_command(
    adapter: &GptAdapter,
    command: BackgroundToAdapter,
) -> Result<Vec<AdapterToBackground>, AdapterError> {
    Ok(match command {
        BackgroundToAdapter::StructuralProbe => match adapter.structural_probe() {
            Ok(()) => vec![AdapterToBackground::StructuralProbePassed {
                provider: ProviderId::Gpt,
            }],
            Err(error) => vec![AdapterToBackground::StructuralProbeFailed {
                provider: ProviderId::Gpt,
                detail: error.to_string(),
            }],
        },
        BackgroundToAdapter::GetHealth => vec![AdapterToBackground::HealthReport {
            provider: ProviderId::Gpt,
            health: adapter.health(),
        }],
        BackgroundToAdapter::InjectInput { text } => {
            adapter.inject_input(&text)?;
            vec![AdapterToBackground::HealthReport {
                provider: ProviderId::Gpt,
                health: adapter.health(),
            }]
        }
        BackgroundToAdapter::Send => {
            adapter.send()?;
            vec![AdapterToBackground::HealthReport {
                provider: ProviderId::Gpt,
                health: adapter.health(),
            }]
        }
        BackgroundToAdapter::ExtractLatestResponse => {
            vec![AdapterToBackground::MessagesCaptured {
                provider: ProviderId::Gpt,
                messages: vec![adapter.extract_latest_response()?],
            }]
        }
        BackgroundToAdapter::ExtractFullHistory => vec![AdapterToBackground::MessagesCaptured {
            provider: ProviderId::Gpt,
            messages: adapter.extract_full_history()?,
        }],
        BackgroundToAdapter::ExtractIncrementalDelta { after_message_id } => {
            vec![AdapterToBackground::MessagesCaptured {
                provider: ProviderId::Gpt,
                messages: adapter.extract_incremental_delta(after_message_id)?,
            }]
        }
        BackgroundToAdapter::DetectBlockingState => {
            if let Some(blocking_state) = adapter.detect_blocking_state() {
                vec![AdapterToBackground::BlockingStateDetected {
                    provider: ProviderId::Gpt,
                    blocking_state,
                }]
            } else {
                vec![AdapterToBackground::HealthReport {
                    provider: ProviderId::Gpt,
                    health: adapter.health(),
                }]
            }
        }
        BackgroundToAdapter::GetConversationRef => {
            vec![AdapterToBackground::ConversationRefDiscovered {
                provider: ProviderId::Gpt,
                conversation_ref: adapter.conversation_ref(),
            }]
        }
        BackgroundToAdapter::GetProviderSnapshot => {
            vec![AdapterToBackground::ProviderControlSnapshotCaptured {
                provider: ProviderId::Gpt,
                snapshot: query::provider_snapshot()?,
            }]
        }
        BackgroundToAdapter::CreateProject { title } => {
            query::create_project(&title)?;
            vec![AdapterToBackground::ProviderControlSnapshotCaptured {
                provider: ProviderId::Gpt,
                snapshot: query::provider_snapshot()?,
            }]
        }
        BackgroundToAdapter::SelectProject { project_id } => {
            query::select_project(&project_id)?;
            vec![AdapterToBackground::ProviderControlSnapshotCaptured {
                provider: ProviderId::Gpt,
                snapshot: query::provider_snapshot()?,
            }]
        }
        BackgroundToAdapter::CreateConversation {
            project_id,
            title: _,
        } => {
            query::create_conversation(project_id.as_deref())?;
            vec![AdapterToBackground::ProviderControlSnapshotCaptured {
                provider: ProviderId::Gpt,
                snapshot: query::provider_snapshot()?,
            }]
        }
        BackgroundToAdapter::SelectConversation { conversation_id } => {
            query::select_conversation(&conversation_id)?;
            vec![AdapterToBackground::ProviderControlSnapshotCaptured {
                provider: ProviderId::Gpt,
                snapshot: query::provider_snapshot()?,
            }]
        }
        BackgroundToAdapter::SetModel { model_id } => {
            query::set_model(&model_id)?;
            vec![AdapterToBackground::ProviderControlSnapshotCaptured {
                provider: ProviderId::Gpt,
                snapshot: query::provider_snapshot()?,
            }]
        }
        BackgroundToAdapter::SetReasoning { reasoning_id } => {
            query::set_reasoning(&reasoning_id)?;
            vec![AdapterToBackground::ProviderControlSnapshotCaptured {
                provider: ProviderId::Gpt,
                snapshot: query::provider_snapshot()?,
            }]
        }
        BackgroundToAdapter::SetFeatureFlag { key, enabled } => {
            query::set_feature_flag(&key, enabled)?;
            vec![AdapterToBackground::ProviderControlSnapshotCaptured {
                provider: ProviderId::Gpt,
                snapshot: query::provider_snapshot()?,
            }]
        }
    })
}

mod query {
    use super::*;

    pub fn structural_probe(transcript: &[&str], inputs: &[&str]) -> Result<(), AdapterError> {
        if exists_any(transcript) && exists_any(inputs) {
            Ok(())
        } else {
            Err(AdapterError::DomMismatch {
                detail: "ChatGPT landmarks were not found".to_owned(),
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
            if let Some(index) = messages
                .iter()
                .position(|message| message.id == after_message_id)
            {
                Ok(messages.into_iter().skip(index + 1).collect())
            } else {
                Ok(messages)
            }
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
        let selector = selectors.join(", ");
        let nodes =
            document
                .query_selector_all(&selector)
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
    pub fn extract_message_list(_: &[&str], _: ProviderId) -> Result<Vec<Message>, AdapterError> {
        Ok(vec![])
    }

    #[cfg(target_arch = "wasm32")]
    pub fn inject_text(selectors: &[&str], text: &str) -> Result<(), AdapterError> {
        use wasm_bindgen::JsCast;
        let document = document()?;
        for selector in selectors {
            if let Ok(Some(node)) = document.query_selector(selector) {
                if let Some(textarea) = node.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                    textarea.set_value(text);
                    dispatch_input_events(textarea.as_ref())?;
                    return Ok(());
                }
                if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
                    input.set_value(text);
                    dispatch_input_events(input.as_ref())?;
                    return Ok(());
                }
                if let Some(element) = node.dyn_ref::<web_sys::HtmlElement>() {
                    element.set_text_content(Some(text));
                    dispatch_input_events(element)?;
                    return Ok(());
                }
            }
        }
        Err(AdapterError::NotFound {
            detail: "no writable ChatGPT input found".to_owned(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn inject_text(_: &[&str], _: &str) -> Result<(), AdapterError> {
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn click_first(selectors: &[&str]) -> Result<(), AdapterError> {
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
            detail: "no ChatGPT send control found".to_owned(),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn click_first(_: &[&str]) -> Result<(), AdapterError> {
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

    pub fn matches_text_any(_selectors: &[&str], _patterns: &[&str]) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(document) = document() {
                for selector in _selectors {
                    let Ok(nodes) = document.query_selector_all(selector) else {
                        continue;
                    };
                    for index in 0..nodes.length() {
                        let Some(node) = nodes.item(index) else {
                            continue;
                        };
                        let Some(element) = node.dyn_ref::<web_sys::Element>() else {
                            continue;
                        };
                        let text = element.text_content().unwrap_or_default();
                        if text_matches_any_pattern(&text, _patterns) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn conversation_ref() -> Option<ConversationRef> {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window()?;
            let location = window.location();
            let pathname = location.pathname().ok()?;
            let current_ref = current_location_ref(&pathname, &location.href().ok());
            let latest_model = current_model_label_from_window(&window);
            if let Some(network_ref) = preferred_network_ref(current_ref.as_ref()) {
                return Some(ConversationRef {
                    conversation_id: network_ref.conversation_id,
                    title: network_ref.title,
                    url: network_ref
                        .url
                        .or_else(|| current_ref.as_ref().and_then(|item| item.url.clone())),
                    model_label: network_ref.model_label.or(latest_model),
                });
            }
            return current_ref.map(|mut reference| {
                reference.model_label = latest_model;
                reference
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn provider_snapshot() -> Result<ProviderControlSnapshot, AdapterError> {
        let document = document()?;
        let location = web_sys::window()
            .ok_or(AdapterError::DomMismatch {
                detail: "window unavailable".to_owned(),
            })?
            .location();
        let pathname = location
            .pathname()
            .map_err(|error| AdapterError::Unsupported {
                detail: format!("failed to read location pathname: {error:?}"),
            })?;

        let current_project_id = project_id_from_path(&pathname);
        let current_location_ref = current_location_ref(&pathname, &location.href().ok());
        let network_ref = preferred_network_ref(current_location_ref.as_ref());
        let current_conversation_id = network_ref
            .as_ref()
            .and_then(|item| item.conversation_id.clone())
            .or_else(|| {
                current_location_ref
                    .as_ref()
                    .and_then(|item| item.conversation_id.clone())
            });
        let projects = list_projects(&document, current_project_id.as_deref())?;
        let conversations = list_conversations(&document, &pathname)?;
        let model_label = current_model_label(&document);
        let model_id = current_model_id(&document)
            .or_else(|| model_label.as_ref().map(|label| normalize_model_id(label)));
        let reasoning_id = infer_reasoning_id(model_id.as_deref(), &document);
        let reasoning_label = reasoning_id
            .as_deref()
            .map(reasoning_label_for_id)
            .map(str::to_owned);
        let auto_switch_enabled = detect_auto_switch_enabled(&document);
        let mut feature_flags = BTreeMap::new();
        if let Some(enabled) = auto_switch_enabled {
            feature_flags.insert("auto_switch_thinking".to_owned(), enabled);
        }

        Ok(ProviderControlSnapshot {
            provider: ProviderId::Gpt,
            capabilities: ProviderControlCapabilities {
                supports_projects: true,
                supports_project_creation: true,
                supports_conversations: true,
                supports_conversation_creation: true,
                supports_model_selection: true,
                supports_reasoning_selection: true,
                supports_feature_flags: auto_switch_enabled.is_some(),
                supports_sync: true,
            },
            state: ProviderControlState {
                project_id: current_project_id.clone(),
                project_title: projects
                    .iter()
                    .find(|project| project.is_active)
                    .map(|project| project.title.clone()),
                conversation_id: current_conversation_id.clone(),
                conversation_title: conversations
                    .iter()
                    .find(|conversation| conversation.is_active)
                    .map(|conversation| conversation.title.clone()),
                model_id: model_id.clone(),
                model_label: model_label.clone(),
                reasoning_id,
                reasoning_label,
                feature_flags,
                last_strategy: Some(if network_ref.is_some() {
                    ProviderStrategy::Network
                } else {
                    ProviderStrategy::Dom
                }),
                degraded: false,
            },
            projects,
            conversations,
            models: list_models(&document, model_id.as_deref()),
            reasoning_options: list_reasoning_options(&document),
            feature_flags: list_feature_flags(auto_switch_enabled),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn provider_snapshot() -> Result<chatmux_common::ProviderControlSnapshot, AdapterError> {
        Err(AdapterError::Unsupported {
            detail: "provider snapshot requires wasm32".to_owned(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_project(title: &str) -> Result<(), AdapterError> {
        let document = document()?;
        click_button_by_text(&document, "New project")?;
        let input = query_input(&document, "#project-name, input[name='projectName']")?;
        input.set_value(title);
        dispatch_input_events(input.unchecked_ref())?;
        click_button_by_text(&document, "Create project")
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_project(_: &str) -> Result<(), AdapterError> {
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn select_project(project_id: &str) -> Result<(), AdapterError> {
        let document = document()?;
        let href = find_project_href(&document, project_id)?;
        navigate_to_href(&href)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn select_project(_: &str) -> Result<(), AdapterError> {
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_conversation(project_id: Option<&str>) -> Result<(), AdapterError> {
        if let Some(project_id) = project_id {
            let document = document()?;
            let href = find_project_href(&document, project_id)?;
            return navigate_to_href(&href);
        }

        navigate_to_href("/")
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_conversation(_: Option<&str>) -> Result<(), AdapterError> {
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn select_conversation(conversation_id: &str) -> Result<(), AdapterError> {
        let document = document()?;
        let href = find_conversation_href(&document, conversation_id)?;
        navigate_to_href(&href)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn select_conversation(_: &str) -> Result<(), AdapterError> {
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_model(model_id: &str) -> Result<(), AdapterError> {
        let document = document()?;
        click_first(&["button[data-testid='model-switcher-dropdown-button']"])?;
        let selector = format!("[data-testid='model-switcher-{model_id}']");
        if let Ok(Some(node)) = document.query_selector(&selector) {
            if let Some(element) = node.dyn_ref::<web_sys::HtmlElement>() {
                element.click();
                return Ok(());
            }
        }
        Err(AdapterError::NotFound {
            detail: format!("model option not found: {model_id}"),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_model(_: &str) -> Result<(), AdapterError> {
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_reasoning(reasoning_id: &str) -> Result<(), AdapterError> {
        match reasoning_id {
            "instant" => set_model("gpt-5-3"),
            "thinking" => set_model("gpt-5-4-thinking"),
            "pro" => set_model("gpt-5-4-pro"),
            other => Err(AdapterError::Unsupported {
                detail: format!("unsupported reasoning option: {other}"),
            }),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_reasoning(_: &str) -> Result<(), AdapterError> {
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_feature_flag(key: &str, enabled: bool) -> Result<(), AdapterError> {
        match key {
            "auto_switch_thinking" => set_auto_switch(enabled),
            other => Err(AdapterError::Unsupported {
                detail: format!("unsupported feature flag: {other}"),
            }),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_feature_flag(_: &str, _: bool) -> Result<(), AdapterError> {
        Ok(())
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
    fn dispatch_input_events(element: &web_sys::HtmlElement) -> Result<(), AdapterError> {
        for event_name in ["input", "change"] {
            let event =
                web_sys::Event::new(event_name).map_err(|error| AdapterError::Unsupported {
                    detail: format!("failed to create {event_name} event: {error:?}"),
                })?;
            element
                .dispatch_event(&event)
                .map_err(|error| AdapterError::Unsupported {
                    detail: format!("failed to dispatch {event_name} event: {error:?}"),
                })?;
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn query_input(
        document: &web_sys::Document,
        selector: &str,
    ) -> Result<web_sys::HtmlInputElement, AdapterError> {
        use wasm_bindgen::JsCast;
        let node = document
            .query_selector(selector)
            .map_err(|error| AdapterError::Unsupported {
                detail: format!("query failed for {selector}: {error:?}"),
            })?
            .ok_or(AdapterError::NotFound {
                detail: format!("input not found for selector {selector}"),
            })?;
        node.dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| AdapterError::Unsupported {
                detail: format!("selector did not resolve to an input: {selector}"),
            })
    }

    #[cfg(target_arch = "wasm32")]
    fn click_button_by_text(document: &web_sys::Document, label: &str) -> Result<(), AdapterError> {
        use wasm_bindgen::JsCast;
        let buttons =
            document
                .query_selector_all("button")
                .map_err(|error| AdapterError::Unsupported {
                    detail: format!("failed to query buttons: {error:?}"),
                })?;
        for index in 0..buttons.length() {
            if let Some(node) = buttons.item(index) {
                let text = node.text_content().unwrap_or_default().trim().to_owned();
                if text == label {
                    if let Some(element) = node.dyn_ref::<web_sys::HtmlElement>() {
                        element.click();
                        return Ok(());
                    }
                }
            }
        }
        Err(AdapterError::NotFound {
            detail: format!("button not found: {label}"),
        })
    }

    #[cfg(target_arch = "wasm32")]
    fn list_projects(
        document: &web_sys::Document,
        current_project_id: Option<&str>,
    ) -> Result<Vec<ProviderProject>, AdapterError> {
        let links = document
            .query_selector_all("a[href*='/project']")
            .map_err(|error| AdapterError::Unsupported {
                detail: format!("failed to query project links: {error:?}"),
            })?;
        let mut projects = Vec::new();
        for index in 0..links.length() {
            if let Some(node) = links.item(index) {
                if let Some(element) = node.dyn_ref::<web_sys::Element>() {
                    let Some(href) = element.get_attribute("href") else {
                        continue;
                    };
                    let Some(id) = project_id_from_path(&href) else {
                        continue;
                    };
                    let title = element.text_content().unwrap_or_default().trim().to_owned();
                    if title.is_empty() {
                        continue;
                    }
                    projects.push(ProviderProject {
                        id: id.clone(),
                        title,
                        is_active: current_project_id == Some(id.as_str()),
                        provider_metadata: chatmux_common::MetadataBag::default(),
                    });
                }
            }
        }
        dedupe_projects(projects)
    }

    #[cfg(target_arch = "wasm32")]
    fn list_conversations(
        document: &web_sys::Document,
        pathname: &str,
    ) -> Result<Vec<ProviderConversation>, AdapterError> {
        let links = document
            .query_selector_all("a[href*='/c/']")
            .map_err(|error| AdapterError::Unsupported {
                detail: format!("failed to query conversation links: {error:?}"),
            })?;
        let current_conversation_id = conversation_id_from_path(pathname);
        let current_project_id = project_id_from_path(pathname);
        let mut conversations = Vec::new();
        for index in 0..links.length() {
            if let Some(node) = links.item(index) {
                if let Some(element) = node.dyn_ref::<web_sys::Element>() {
                    let Some(href) = element.get_attribute("href") else {
                        continue;
                    };
                    let Some(id) = conversation_id_from_path(&href) else {
                        continue;
                    };
                    let title = conversation_title_from_link(element);
                    if title.is_empty() {
                        continue;
                    }
                    conversations.push(ProviderConversation {
                        id: id.clone(),
                        project_id: project_id_from_path(&href)
                            .or_else(|| current_project_id.clone()),
                        title,
                        is_active: current_conversation_id == Some(id.clone()),
                        model_label: None,
                        provider_metadata: chatmux_common::MetadataBag::default(),
                    });
                }
            }
        }
        Ok(dedupe_conversations(conversations))
    }

    #[cfg(target_arch = "wasm32")]
    fn list_models(
        document: &web_sys::Document,
        current_model_id: Option<&str>,
    ) -> Vec<ProviderModelOption> {
        let known = [
            ("gpt-5-3", "Instant"),
            ("gpt-5-4-thinking", "Thinking"),
            ("gpt-5-4-pro", "Pro"),
        ];
        known
            .into_iter()
            .map(|(id, label)| ProviderModelOption {
                id: id.to_owned(),
                label: label.to_owned(),
                is_active: current_model_id == Some(id),
                provider_metadata: chatmux_common::MetadataBag {
                    values: BTreeMap::from([(
                        "source".to_owned(),
                        if document
                            .query_selector(&format!("[data-testid='model-switcher-{id}']"))
                            .ok()
                            .flatten()
                            .is_some()
                        {
                            "dom".to_owned()
                        } else {
                            "known_default".to_owned()
                        },
                    )]),
                },
            })
            .collect()
    }

    #[cfg(target_arch = "wasm32")]
    fn list_reasoning_options(_document: &web_sys::Document) -> Vec<ProviderReasoningOption> {
        [
            ("instant", "Instant", Some("Fast response mode")),
            ("thinking", "Thinking", Some("Extended thinking mode")),
            ("pro", "Pro", Some("Highest reasoning mode available")),
        ]
        .into_iter()
        .map(|(id, label, description)| ProviderReasoningOption {
            id: id.to_owned(),
            label: label.to_owned(),
            description: description.map(str::to_owned),
            is_active: false,
            provider_metadata: chatmux_common::MetadataBag::default(),
        })
        .collect()
    }

    #[cfg(target_arch = "wasm32")]
    fn list_feature_flags(auto_switch_enabled: Option<bool>) -> Vec<ProviderFeatureFlag> {
        auto_switch_enabled
            .map(|enabled| ProviderFeatureFlag {
                key: "auto_switch_thinking".to_owned(),
                label: "Auto-switch to Thinking".to_owned(),
                enabled,
            })
            .into_iter()
            .collect()
    }

    #[cfg(target_arch = "wasm32")]
    fn detect_auto_switch_enabled(document: &web_sys::Document) -> Option<bool> {
        document
            .query_selector("[role='switch'][aria-label*='Thinking']")
            .ok()
            .flatten()
            .and_then(|node| node.get_attribute("aria-checked"))
            .map(|value| value == "true")
    }

    #[cfg(target_arch = "wasm32")]
    fn current_model_label(document: &web_sys::Document) -> Option<String> {
        document
            .query_selector("[data-testid='model-switcher-dropdown-button']")
            .ok()
            .flatten()
            .map(|node| node.text_content().unwrap_or_default().trim().to_owned())
            .filter(|text| !text.is_empty())
    }

    #[cfg(target_arch = "wasm32")]
    fn current_model_id(document: &web_sys::Document) -> Option<String> {
        document
            .query_selector("[data-message-author-role='assistant'][data-message-model-slug]")
            .ok()
            .flatten()
            .and_then(|node| node.get_attribute("data-message-model-slug"))
    }

    #[cfg(target_arch = "wasm32")]
    fn infer_reasoning_id(model_id: Option<&str>, document: &web_sys::Document) -> Option<String> {
        if let Some(model_id) = model_id {
            if model_id.contains("pro") {
                return Some("pro".to_owned());
            }
            if model_id.contains("thinking") || model_id.contains("4") {
                return Some("thinking".to_owned());
            }
        }

        if document
            .query_selector("button[aria-label*='Extended thinking, click to remove']")
            .ok()
            .flatten()
            .is_some()
        {
            Some("thinking".to_owned())
        } else {
            Some("instant".to_owned())
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn reasoning_label_for_id(reasoning_id: &str) -> &'static str {
        match reasoning_id {
            "instant" => "Instant",
            "thinking" => "Thinking",
            "pro" => "Pro",
            _ => "Unknown",
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn normalize_model_id(label: &str) -> String {
        let normalized = label.to_ascii_lowercase();
        if normalized.contains("pro") {
            "gpt-5-4-pro".to_owned()
        } else if normalized.contains("thinking") {
            "gpt-5-4-thinking".to_owned()
        } else {
            "gpt-5-3".to_owned()
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn set_auto_switch(enabled: bool) -> Result<(), AdapterError> {
        let document = document()?;
        let maybe_switch = document
            .query_selector("[role='switch'][aria-label*='Thinking']")
            .map_err(|error| AdapterError::Unsupported {
                detail: format!("failed to query auto-switch control: {error:?}"),
            })?;
        let Some(node) = maybe_switch else {
            return Err(AdapterError::Unsupported {
                detail: "auto-switch control is only available while the ChatGPT configure modal is open".to_owned(),
            });
        };
        let current = node
            .get_attribute("aria-checked")
            .map(|value| value == "true")
            .unwrap_or(false);
        if current == enabled {
            return Ok(());
        }
        let Some(element) = node.dyn_ref::<web_sys::HtmlElement>() else {
            return Err(AdapterError::Unsupported {
                detail: "auto-switch control is not clickable".to_owned(),
            });
        };
        element.click();
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn project_id_from_path(path: &str) -> Option<String> {
        path.split('/')
            .find(|segment| segment.starts_with("g-p-"))
            .map(str::to_owned)
    }

    #[cfg(target_arch = "wasm32")]
    fn conversation_id_from_path(path: &str) -> Option<String> {
        let mut segments = path.split('/').peekable();
        while let Some(segment) = segments.next() {
            if segment == "c" {
                return segments.next().map(str::to_owned);
            }
        }
        None
    }

    #[cfg(target_arch = "wasm32")]
    fn find_project_href(
        document: &web_sys::Document,
        project_id: &str,
    ) -> Result<String, AdapterError> {
        let selector = format!("a[href*='{project_id}'][href*='/project']");
        document
            .query_selector(&selector)
            .map_err(|error| AdapterError::Unsupported {
                detail: format!("failed to query project href: {error:?}"),
            })?
            .and_then(|node| node.get_attribute("href"))
            .ok_or(AdapterError::NotFound {
                detail: format!("project not found: {project_id}"),
            })
    }

    #[cfg(target_arch = "wasm32")]
    fn find_conversation_href(
        document: &web_sys::Document,
        conversation_id: &str,
    ) -> Result<String, AdapterError> {
        let selector = format!("a[href*='/c/{conversation_id}']");
        document
            .query_selector(&selector)
            .map_err(|error| AdapterError::Unsupported {
                detail: format!("failed to query conversation href: {error:?}"),
            })?
            .and_then(|node| node.get_attribute("href"))
            .ok_or(AdapterError::NotFound {
                detail: format!("conversation not found: {conversation_id}"),
            })
    }

    #[cfg(target_arch = "wasm32")]
    fn navigate_to_href(href: &str) -> Result<(), AdapterError> {
        let window = web_sys::window().ok_or(AdapterError::DomMismatch {
            detail: "window unavailable".to_owned(),
        })?;
        window
            .location()
            .set_href(href)
            .map_err(|error| AdapterError::Unsupported {
                detail: format!("failed to navigate to {href}: {error:?}"),
            })
    }

    #[cfg(target_arch = "wasm32")]
    fn conversation_title_from_link(element: &web_sys::Element) -> String {
        element
            .get_attribute("aria-label")
            .unwrap_or_else(|| element.text_content().unwrap_or_default())
            .split(',')
            .next()
            .unwrap_or_default()
            .trim()
            .to_owned()
    }

    #[cfg(target_arch = "wasm32")]
    fn dedupe_projects(
        projects: Vec<ProviderProject>,
    ) -> Result<Vec<ProviderProject>, AdapterError> {
        let mut deduped = BTreeMap::new();
        for project in projects {
            deduped.entry(project.id.clone()).or_insert(project);
        }
        Ok(deduped.into_values().collect())
    }

    #[cfg(target_arch = "wasm32")]
    fn dedupe_conversations(conversations: Vec<ProviderConversation>) -> Vec<ProviderConversation> {
        let mut deduped = BTreeMap::new();
        for conversation in conversations {
            deduped
                .entry(conversation.id.clone())
                .or_insert(conversation);
        }
        deduped.into_values().collect()
    }

    #[cfg(target_arch = "wasm32")]
    fn message_from_element(
        provider: ProviderId,
        element: &web_sys::Element,
        dom_index: usize,
    ) -> Result<Option<Message>, AdapterError> {
        let role = match element.get_attribute("data-message-author-role").as_deref() {
            Some("assistant") => MessageRole::Assistant,
            Some("user") => MessageRole::User,
            Some("system") => MessageRole::System,
            _ => return Ok(None),
        };

        let text = element.text_content().unwrap_or_default().trim().to_owned();
        if text.is_empty() {
            return Ok(None);
        }

        let message_id = element
            .get_attribute("data-message-id")
            .and_then(|value| Uuid::parse_str(&value).ok())
            .map(MessageId)
            .unwrap_or_else(|| {
                stable_message_fallback_id(
                    provider,
                    role,
                    dom_index,
                    &text,
                    element.get_attribute("data-message-model-slug").as_deref(),
                    conversation_identity_key().as_deref(),
                )
            });
        let participant_id = match role {
            MessageRole::User => ProviderId::User,
            MessageRole::Assistant => provider,
            MessageRole::System => ProviderId::System,
        };

        let mut tags = Vec::new();
        if let Some(model_slug) = element.get_attribute("data-message-model-slug") {
            tags.push(format!("chatgpt_model:{model_slug}"));
        }

        let raw_response_text = match role {
            MessageRole::Assistant => Some(raw_assistant_capture(element, &text)?),
            _ => None,
        };
        let network_capture = match role {
            MessageRole::Assistant => latest_network_capture(),
            _ => None,
        };

        Ok(Some(Message {
            id: message_id,
            workspace_id: WorkspaceId::new(),
            participant_id,
            role,
            round: None,
            timestamp: Utc::now(),
            body_text: text.clone(),
            body_blocks: vec![chatmux_common::Block::Paragraph { text }],
            source_binding_id: None,
            dispatch_id: None,
            raw_response_text,
            network_capture,
            tags,
            capture_confidence: CaptureConfidence::Certain,
        }))
    }

    #[cfg(target_arch = "wasm32")]
    fn raw_assistant_capture(
        _element: &web_sys::Element,
        text: &str,
    ) -> Result<String, AdapterError> {
        Ok(text.to_owned())
    }

    #[cfg(target_arch = "wasm32")]
    fn latest_network_capture() -> Option<ProviderNetworkCapture> {
        let window = web_sys::window()?;
        let value = Reflect::get(
            window.as_ref(),
            &JsValue::from_str("__chatmuxLatestNetworkCapture"),
        )
        .ok()?;
        if value.is_undefined() || value.is_null() {
            return None;
        }
        serde_wasm_bindgen::from_value(value).ok()
    }

    #[cfg(target_arch = "wasm32")]
    fn current_model_label_from_window(window: &web_sys::Window) -> Option<String> {
        let document = window.document()?;
        document
            .query_selector("[data-message-author-role='assistant'][data-message-model-slug]")
            .ok()
            .flatten()
            .and_then(|node| node.get_attribute("data-message-model-slug"))
    }

    #[cfg(target_arch = "wasm32")]
    fn current_location_ref(pathname: &str, href: &Option<String>) -> Option<ConversationRef> {
        conversation_id_from_path(pathname).map(|conversation_id| ConversationRef {
            conversation_id: Some(conversation_id),
            title: None,
            url: href.clone(),
            model_label: None,
        })
    }

    #[cfg(target_arch = "wasm32")]
    fn preferred_network_ref(
        current_location_ref: Option<&ConversationRef>,
    ) -> Option<ConversationRef> {
        let network_ref = latest_network_capture()?.conversation_ref?;
        if current_location_ref.is_some_and(|current_ref| !network_ref.matches_target(current_ref))
        {
            return None;
        }
        Some(network_ref)
    }

    #[cfg(target_arch = "wasm32")]
    fn conversation_identity_key() -> Option<String> {
        let window = web_sys::window()?;
        let location = window.location();
        let pathname = location.pathname().ok()?;
        let current_ref = current_location_ref(&pathname, &location.href().ok());
        let preferred_ref = preferred_network_ref(current_ref.as_ref()).or(current_ref);
        preferred_ref.and_then(|reference| {
            reference.conversation_id.or_else(|| {
                reference
                    .url
                    .and_then(|url| normalized_chat_url_for_fallback(Some(&url)))
            })
        })
    }

    #[cfg_attr(not(any(target_arch = "wasm32", test)), allow(dead_code))]
    pub(super) fn text_matches_any_pattern(text: &str, patterns: &[&str]) -> bool {
        let normalized = text.trim().to_ascii_lowercase();
        !normalized.is_empty()
            && patterns
                .iter()
                .any(|pattern| normalized.contains(&pattern.to_ascii_lowercase()))
    }

    #[cfg_attr(not(any(target_arch = "wasm32", test)), allow(dead_code))]
    pub(super) fn stable_message_fallback_id(
        provider: ProviderId,
        role: chatmux_common::MessageRole,
        dom_index: usize,
        text: &str,
        model_slug: Option<&str>,
        conversation_key: Option<&str>,
    ) -> MessageId {
        let fingerprint = format!(
            "chatgpt:{provider:?}:{role:?}:{dom_index}:{conversation_key}:{model_slug}:{text}",
            conversation_key = conversation_key.unwrap_or("no-conversation"),
            model_slug = model_slug.unwrap_or("no-model"),
        );
        let lower = stable_u64(&fingerprint);
        let upper = stable_u64(&format!("chatmux-fallback:{fingerprint}"));
        MessageId(uuid::Uuid::from_u128(
            ((upper as u128) << 64) | lower as u128,
        ))
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn normalized_chat_url_for_fallback(url: Option<&str>) -> Option<String> {
        let value = url?.trim();
        if value.is_empty() {
            return None;
        }
        let without_fragment = value.split('#').next().unwrap_or(value);
        let trimmed = without_fragment.trim_end_matches('/');
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    }

    #[cfg_attr(not(any(target_arch = "wasm32", test)), allow(dead_code))]
    fn stable_u64(input: &str) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_sets_are_not_empty() {
        assert!(!TRANSCRIPT_SELECTORS.is_empty());
        assert!(!INPUT_SELECTORS.is_empty());
    }

    #[test]
    fn rate_limit_banner_text_must_match_known_blocking_phrases() {
        assert!(query::text_matches_any_pattern(
            "You've reached the rate limit. Try again later.",
            RATE_LIMIT_TEXT_PATTERNS
        ));
        assert!(!query::text_matches_any_pattern(
            "Temporary warning without any blocking language.",
            RATE_LIMIT_TEXT_PATTERNS
        ));
    }

    #[test]
    fn stable_message_fallback_id_is_repeatable() {
        let first = query::stable_message_fallback_id(
            ProviderId::Gpt,
            chatmux_common::MessageRole::Assistant,
            3,
            "Hello world",
            Some("gpt-5-4-thinking"),
            Some("conversation-123"),
        );
        let second = query::stable_message_fallback_id(
            ProviderId::Gpt,
            chatmux_common::MessageRole::Assistant,
            3,
            "Hello world",
            Some("gpt-5-4-thinking"),
            Some("conversation-123"),
        );

        assert_eq!(first, second);
    }

    #[test]
    fn stable_message_fallback_id_changes_when_conversation_changes() {
        let first = query::stable_message_fallback_id(
            ProviderId::Gpt,
            chatmux_common::MessageRole::Assistant,
            3,
            "Hello world",
            Some("gpt-5-4-thinking"),
            Some("conversation-123"),
        );
        let second = query::stable_message_fallback_id(
            ProviderId::Gpt,
            chatmux_common::MessageRole::Assistant,
            3,
            "Hello world",
            Some("gpt-5-4-thinking"),
            Some("conversation-456"),
        );

        assert_ne!(first, second);
    }
}
