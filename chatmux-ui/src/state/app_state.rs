//! Global app-level state.

use leptos::prelude::*;
use std::collections::BTreeMap;

use crate::bridge::storage::UiSettings;
use crate::models::{
    BlockingState, Dispatch, ExportFormat, Message, ProviderControlDefaults,
    ProviderControlSnapshot, ProviderHealth, ProviderId, WorkspaceId,
};

#[derive(Clone, Debug, Default)]
pub struct MessageInspectionState {
    pub message: Option<Message>,
    pub dispatch: Option<Dispatch>,
    pub sent_payload: Option<String>,
    pub raw_capture_ref: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ExportState {
    pub format: ExportFormat,
    pub mime_type: String,
    pub filename: String,
    pub body: String,
}

#[derive(Clone, Debug)]
pub struct ProviderRuntimeState {
    pub health: ProviderHealth,
    pub blocking_state: Option<BlockingState>,
}

#[derive(Clone, Debug, Default)]
pub struct ProviderControlRegistry {
    pub snapshots: BTreeMap<ProviderId, ProviderControlSnapshot>,
    pub defaults: BTreeMap<ProviderId, ProviderControlDefaults>,
}

/// Global application state provided at the app root.
#[derive(Clone, Copy)]
pub struct AppState {
    /// The ID of the currently active workspace, if any.
    pub active_workspace_id: ReadSignal<Option<WorkspaceId>>,
    pub set_active_workspace_id: WriteSignal<Option<WorkspaceId>>,
    pub bridge_ready: ReadSignal<bool>,
    pub set_bridge_ready: WriteSignal<bool>,
    pub last_error: ReadSignal<Option<String>>,
    pub set_last_error: WriteSignal<Option<String>>,
    pub kill_switch_active: ReadSignal<bool>,
    pub set_kill_switch_active: WriteSignal<bool>,
    pub inspection: ReadSignal<Option<MessageInspectionState>>,
    pub set_inspection: WriteSignal<Option<MessageInspectionState>>,
    pub export: ReadSignal<Option<ExportState>>,
    pub set_export: WriteSignal<Option<ExportState>>,
    pub provider_health: ReadSignal<BTreeMap<ProviderId, ProviderRuntimeState>>,
    pub set_provider_health: WriteSignal<BTreeMap<ProviderId, ProviderRuntimeState>>,
    pub provider_controls: ReadSignal<ProviderControlRegistry>,
    pub set_provider_controls: WriteSignal<ProviderControlRegistry>,
    pub ui_settings: ReadSignal<UiSettings>,
    pub set_ui_settings: WriteSignal<UiSettings>,
}

/// Create and provide the global app state.
pub fn provide_app_state() -> AppState {
    let (active_workspace_id, set_active_workspace_id) = signal(None::<WorkspaceId>);
    let (bridge_ready, set_bridge_ready) = signal(false);
    let (last_error, set_last_error) = signal(None::<String>);
    let (kill_switch_active, set_kill_switch_active) = signal(false);
    let (inspection, set_inspection) = signal(None::<MessageInspectionState>);
    let (export, set_export) = signal(None::<ExportState>);
    let (provider_health, set_provider_health) = signal(BTreeMap::new());
    let (provider_controls, set_provider_controls) = signal(ProviderControlRegistry::default());
    let (ui_settings, set_ui_settings) = signal(UiSettings::default());

    let state = AppState {
        active_workspace_id,
        set_active_workspace_id,
        bridge_ready,
        set_bridge_ready,
        last_error,
        set_last_error,
        kill_switch_active,
        set_kill_switch_active,
        inspection,
        set_inspection,
        export,
        set_export,
        provider_health,
        set_provider_health,
        provider_controls,
        set_provider_controls,
        ui_settings,
        set_ui_settings,
    };

    provide_context(state);
    state
}
