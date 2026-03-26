//! Wasm entrypoints used by thin background bootstrap scripts.

use crate::coordinator::BackgroundCoordinator;
use crate::storage::RuntimeStateStore;
use chatmux_common::{AdapterToBackground, UiCommand};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn bootstrap_background() -> Result<JsValue, JsValue> {
    let coordinator = BackgroundCoordinator::new(RuntimeStateStore::default());
    let settings = coordinator
        .load_settings()
        .await
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    // TODO(frontend): The extension shell needs to know when the background coordinator has
    // completed startup and what initial status it has (ready, degraded, diagnostics pending),
    // including whether a kill switch is active and whether paused runs can be resumed.
    serde_wasm_bindgen::to_value(&BootstrapStatus {
        ready: true,
        kill_switch_active: settings.kill_switch_active,
        resume_marker_count: settings.resume_markers.len() as u32,
    })
    .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub async fn handle_ui_command_json(payload: String) -> Result<JsValue, JsValue> {
    let command: UiCommand =
        serde_json::from_str(&payload).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let coordinator = BackgroundCoordinator::new(RuntimeStateStore::default());
    let events = coordinator
        .handle_ui_command(command)
        .await
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    serde_wasm_bindgen::to_value(&events).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub async fn handle_adapter_event_json(
    workspace_id: String,
    payload: String,
) -> Result<JsValue, JsValue> {
    let workspace_id = parse_workspace_id(&workspace_id)?;
    let event: AdapterToBackground =
        serde_json::from_str(&payload).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let coordinator = BackgroundCoordinator::new(RuntimeStateStore::default());
    let events = coordinator
        .ingest_adapter_event(workspace_id, event)
        .await
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    serde_wasm_bindgen::to_value(&events).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapStatus {
    pub ready: bool,
    pub kill_switch_active: bool,
    pub resume_marker_count: u32,
}

fn parse_workspace_id(value: &str) -> Result<chatmux_common::WorkspaceId, JsValue> {
    let uuid = value
        .parse()
        .map_err(|error: uuid::Error| JsValue::from_str(&error.to_string()))?;
    Ok(chatmux_common::WorkspaceId(uuid))
}
