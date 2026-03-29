//! WebExtension runtime messaging bridge.
//!
//! Wraps the `UiCommand`/`UiEvent` protocol from `chatmux-common`.
//! Sends commands as JSON via `runtime.sendMessage`, receives events
//! via `runtime.onMessage` listener.

use chatmux_common::{
    self as common, ApprovalMode, ExportFormat, ExportLayout, MessageId, OrchestrationMode,
    ProviderId, RunId, TemplateId, UiCommand, UiEvent, WorkspaceId,
};
use js_sys::Function;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::bridge::webextension;

#[derive(serde::Deserialize)]
struct CommandResponse {
    ok: bool,
    events: Option<Vec<UiEvent>>,
    error: Option<String>,
}

#[derive(Serialize)]
struct CommandEnvelope<'a> {
    channel: &'static str,
    payload: &'a UiCommand,
}

/// Send a `UiCommand` to the background coordinator and return the response.
///
/// This is the core bridge function. All other functions are thin wrappers.
/// The background.js listener receives this on the `chatmux_ui_command` channel.
async fn send_command(command: &UiCommand) -> Result<Vec<UiEvent>, String> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    let message = CommandEnvelope {
        channel: "chatmux_ui_command",
        payload: command,
    }
    .serialize(&serializer)
    .map_err(|error| error.to_string())?;

    let response = webextension::runtime_send_message(message)
        .await
        .map_err(js_error)?;
    let envelope: CommandResponse =
        serde_wasm_bindgen::from_value(response).map_err(|error| error.to_string())?;

    if envelope.ok {
        Ok(envelope.events.unwrap_or_default())
    } else {
        Err(envelope
            .error
            .unwrap_or_else(|| "runtime.sendMessage returned an unknown error".to_owned()))
    }
}

/// Parse a `UiEvent` from a JSON string received via `runtime.onMessage`.
pub fn parse_event(json: &str) -> Result<UiEvent, String> {
    serde_json::from_str(json).map_err(|e| format!("Failed to parse UiEvent: {e}"))
}

// ---------------------------------------------------------------------------
// Workspace commands
// ---------------------------------------------------------------------------

pub async fn request_workspace_list() -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::RequestWorkspaceList).await
}

pub async fn create_workspace(name: String) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::CreateWorkspace { name }).await
}

pub async fn rename_workspace(
    workspace_id: WorkspaceId,
    name: String,
) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::RenameWorkspace { workspace_id, name }).await
}

pub async fn delete_workspace(workspace_id: WorkspaceId) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::DeleteWorkspace { workspace_id }).await
}

pub async fn set_workspace_archived(
    workspace_id: WorkspaceId,
    archived: bool,
) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::SetWorkspaceArchived {
        workspace_id,
        archived,
    })
    .await
}

pub async fn open_workspace(workspace_id: WorkspaceId) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::OpenWorkspace { workspace_id }).await
}

pub async fn request_workspace_snapshot(
    workspace_id: WorkspaceId,
) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::RequestWorkspaceSnapshot { workspace_id }).await
}

// ---------------------------------------------------------------------------
// Run commands
// ---------------------------------------------------------------------------

pub async fn start_run(
    workspace_id: WorkspaceId,
    mode: OrchestrationMode,
) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::StartRun {
        workspace_id,
        mode,
    })
    .await
}

pub async fn pause_run(run_id: RunId) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::PauseRun { run_id }).await
}

pub async fn resume_run(run_id: RunId) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::ResumeRun { run_id }).await
}

pub async fn step_run(run_id: RunId) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::StepRun { run_id }).await
}

pub async fn stop_run(run_id: RunId) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::StopRun { run_id }).await
}

pub async fn abort_run(run_id: RunId) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::AbortRun { run_id }).await
}

// ---------------------------------------------------------------------------
// Message commands
// ---------------------------------------------------------------------------

pub async fn send_manual_message(
    workspace_id: WorkspaceId,
    targets: Vec<ProviderId>,
    text: String,
    approval_mode: ApprovalMode,
) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::SendManualMessage {
        workspace_id,
        targets,
        text,
        approval_mode,
    })
    .await
}

pub async fn request_message_inspection(
    message_id: MessageId,
) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::RequestMessageInspection { message_id }).await
}

// ---------------------------------------------------------------------------
// Provider commands
// ---------------------------------------------------------------------------

pub async fn toggle_provider(
    workspace_id: WorkspaceId,
    provider: ProviderId,
    enabled: bool,
) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::ToggleProvider {
        workspace_id,
        provider,
        enabled,
    })
    .await
}

// ---------------------------------------------------------------------------
// Template commands
// ---------------------------------------------------------------------------

pub async fn persist_template(template: common::Template) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::PersistTemplate { template }).await
}

pub async fn delete_template(template_id: TemplateId) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::DeleteTemplate { template_id }).await
}

// ---------------------------------------------------------------------------
// Edge policy commands
// ---------------------------------------------------------------------------

pub async fn persist_edge_policy(policy: common::EdgePolicy) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::PersistEdgePolicy { policy }).await
}

// ---------------------------------------------------------------------------
// Export commands
// ---------------------------------------------------------------------------

pub async fn export_selection(
    workspace_id: WorkspaceId,
    format: ExportFormat,
    layout: ExportLayout,
    profile_id: Option<common::ExportProfileId>,
) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::ExportSelection {
        workspace_id,
        format,
        layout,
        profile_id,
    })
    .await
}

pub async fn persist_export_profile(
    profile: common::ExportProfile,
) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::PersistExportProfile { profile }).await
}

// ---------------------------------------------------------------------------
// Kill switch
// ---------------------------------------------------------------------------

pub async fn set_kill_switch(active: bool) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::SetKillSwitch { active }).await
}

// ---------------------------------------------------------------------------
// Clear data
// ---------------------------------------------------------------------------

pub async fn clear_workspace_data(workspace_id: WorkspaceId) -> Result<Vec<UiEvent>, String> {
    send_command(&UiCommand::ClearWorkspaceData { workspace_id }).await
}

// ---------------------------------------------------------------------------
// Event listener (incoming UiEvents from background)
// ---------------------------------------------------------------------------

/// Set up a listener for incoming `UiEvent` messages from the background.
///
/// The callback receives parsed `UiEvent` values. The listener uses
/// `runtime.onMessage` to receive JSON messages from the background.
pub fn listen_for_events(on_event: impl Fn(UiEvent) + 'static) {
    let closure = Closure::wrap(Box::new(move |message: JsValue| {
        let Ok(channel) = js_sys::Reflect::get(&message, &JsValue::from_str("channel")) else {
            return;
        };

        if channel.as_string().as_deref() != Some("chatmux_ui_event") {
            return;
        }

        let Ok(payload) = js_sys::Reflect::get(&message, &JsValue::from_str("payload")) else {
            return;
        };

        if let Ok(event) = serde_wasm_bindgen::from_value::<UiEvent>(payload) {
            on_event(event);
        }
    }) as Box<dyn FnMut(JsValue)>);

    let callback: &Function = closure.as_ref().unchecked_ref();
    if let Err(error) = webextension::runtime_add_listener(callback) {
        log::error!("Failed to register runtime listener: {}", js_error(error));
    }
    closure.forget();
}

fn js_error(error: JsValue) -> String {
    error
        .as_string()
        .or_else(|| js_sys::JSON::stringify(&error).ok().and_then(|value| value.as_string()))
        .unwrap_or_else(|| "unknown JS error".to_owned())
}
