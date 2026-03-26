//! WebExtension runtime messaging bridge.
//!
//! Wraps the `UiCommand`/`UiEvent` protocol from `chatmux-common`.
//! Sends commands as JSON via `runtime.sendMessage`, receives events
//! via `runtime.onMessage` listener.

use chatmux_common::{
    self as common, ApprovalMode, ExportFormat, ExportLayout, MessageId, OrchestrationMode,
    ProviderId, RunId, TemplateId, UiCommand, UiEvent, WorkspaceId,
};
use wasm_bindgen::prelude::*;

/// Send a `UiCommand` to the background coordinator and return the response.
///
/// This is the core bridge function. All other functions are thin wrappers.
/// The background.js listener receives this on the `chatmux_ui_command` channel.
async fn send_command(command: &UiCommand) -> Result<JsValue, String> {
    let json = serde_json::to_string(command).map_err(|e| e.to_string())?;

    // TODO(wiring): Replace with actual runtime.sendMessage call.
    // In production this will be:
    //   let msg = js_sys::JSON::parse(&json).unwrap();
    //   let promise = chrome.runtime.sendMessage(msg);
    //   let result = wasm_bindgen_futures::JsFuture::from(promise).await;
    log::warn!("BRIDGE: send_command (stub): {}", json);
    Err("Bridge not wired — stub".to_string())
}

/// Parse a `UiEvent` from a JSON string received via `runtime.onMessage`.
pub fn parse_event(json: &str) -> Result<UiEvent, String> {
    serde_json::from_str(json).map_err(|e| format!("Failed to parse UiEvent: {e}"))
}

// ---------------------------------------------------------------------------
// Workspace commands
// ---------------------------------------------------------------------------

pub async fn request_workspace_list() -> Result<JsValue, String> {
    send_command(&UiCommand::RequestWorkspaceList).await
}

pub async fn create_workspace(name: String) -> Result<JsValue, String> {
    send_command(&UiCommand::CreateWorkspace { name }).await
}

pub async fn rename_workspace(workspace_id: WorkspaceId, name: String) -> Result<JsValue, String> {
    send_command(&UiCommand::RenameWorkspace { workspace_id, name }).await
}

pub async fn delete_workspace(workspace_id: WorkspaceId) -> Result<JsValue, String> {
    send_command(&UiCommand::DeleteWorkspace { workspace_id }).await
}

pub async fn set_workspace_archived(
    workspace_id: WorkspaceId,
    archived: bool,
) -> Result<JsValue, String> {
    send_command(&UiCommand::SetWorkspaceArchived {
        workspace_id,
        archived,
    })
    .await
}

pub async fn open_workspace(workspace_id: WorkspaceId) -> Result<JsValue, String> {
    send_command(&UiCommand::OpenWorkspace { workspace_id }).await
}

pub async fn request_workspace_snapshot(workspace_id: WorkspaceId) -> Result<JsValue, String> {
    send_command(&UiCommand::RequestWorkspaceSnapshot { workspace_id }).await
}

// ---------------------------------------------------------------------------
// Run commands
// ---------------------------------------------------------------------------

pub async fn start_run(
    workspace_id: WorkspaceId,
    mode: OrchestrationMode,
) -> Result<JsValue, String> {
    send_command(&UiCommand::StartRun {
        workspace_id,
        mode,
    })
    .await
}

pub async fn pause_run(run_id: RunId) -> Result<JsValue, String> {
    send_command(&UiCommand::PauseRun { run_id }).await
}

pub async fn resume_run(run_id: RunId) -> Result<JsValue, String> {
    send_command(&UiCommand::ResumeRun { run_id }).await
}

pub async fn step_run(run_id: RunId) -> Result<JsValue, String> {
    send_command(&UiCommand::StepRun { run_id }).await
}

pub async fn stop_run(run_id: RunId) -> Result<JsValue, String> {
    send_command(&UiCommand::StopRun { run_id }).await
}

pub async fn abort_run(run_id: RunId) -> Result<JsValue, String> {
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
) -> Result<JsValue, String> {
    send_command(&UiCommand::SendManualMessage {
        workspace_id,
        targets,
        text,
        approval_mode,
    })
    .await
}

pub async fn request_message_inspection(message_id: MessageId) -> Result<JsValue, String> {
    send_command(&UiCommand::RequestMessageInspection { message_id }).await
}

// ---------------------------------------------------------------------------
// Provider commands
// ---------------------------------------------------------------------------

pub async fn toggle_provider(
    workspace_id: WorkspaceId,
    provider: ProviderId,
    enabled: bool,
) -> Result<JsValue, String> {
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

pub async fn persist_template(template: common::Template) -> Result<JsValue, String> {
    send_command(&UiCommand::PersistTemplate { template }).await
}

pub async fn delete_template(template_id: TemplateId) -> Result<JsValue, String> {
    send_command(&UiCommand::DeleteTemplate { template_id }).await
}

// ---------------------------------------------------------------------------
// Edge policy commands
// ---------------------------------------------------------------------------

pub async fn persist_edge_policy(policy: common::EdgePolicy) -> Result<JsValue, String> {
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
) -> Result<JsValue, String> {
    send_command(&UiCommand::ExportSelection {
        workspace_id,
        format,
        layout,
        profile_id,
    })
    .await
}

pub async fn persist_export_profile(profile: common::ExportProfile) -> Result<JsValue, String> {
    send_command(&UiCommand::PersistExportProfile { profile }).await
}

// ---------------------------------------------------------------------------
// Kill switch
// ---------------------------------------------------------------------------

pub async fn set_kill_switch(active: bool) -> Result<JsValue, String> {
    send_command(&UiCommand::SetKillSwitch { active }).await
}

// ---------------------------------------------------------------------------
// Clear data
// ---------------------------------------------------------------------------

pub async fn clear_workspace_data(workspace_id: WorkspaceId) -> Result<JsValue, String> {
    send_command(&UiCommand::ClearWorkspaceData { workspace_id }).await
}

// ---------------------------------------------------------------------------
// Event listener (incoming UiEvents from background)
// ---------------------------------------------------------------------------

/// Set up a listener for incoming `UiEvent` messages from the background.
///
/// The callback receives parsed `UiEvent` values. The listener uses
/// `runtime.onMessage` to receive JSON messages from the background.
pub fn listen_for_events(_on_event: impl Fn(UiEvent) + 'static) {
    // TODO(wiring): Register a runtime.onMessage listener that:
    // 1. Receives JSON messages from the background
    // 2. Parses them as UiEvent via parse_event()
    // 3. Calls on_event() with the parsed event
    //
    // In production:
    //   let closure = Closure::wrap(Box::new(move |msg: JsValue, ...| {
    //       let json = js_sys::JSON::stringify(&msg).unwrap().as_string().unwrap();
    //       if let Ok(event) = parse_event(&json) {
    //           on_event(event);
    //       }
    //   }) as Box<dyn Fn(...)>);
    //   chrome.runtime.onMessage.addListener(closure.as_ref().unchecked_ref());
    //   closure.forget();

    log::warn!("BRIDGE: listen_for_events (stub) — not wired to runtime.onMessage");
}
