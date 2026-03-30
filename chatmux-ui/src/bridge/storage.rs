//! Extension storage.local bridge.

use crate::bridge::webextension;
use crate::theme::ThemePreference;
use chatmux_common::{TimingPolicy, WorkspaceId};

/// UI settings — stored in extension storage.local.
/// This is a UI-local type, not from chatmux-common.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UiSettings {
    pub theme: ThemePreference,
    pub surface_preference: SurfacePreference,
    pub timing: TimingPolicy,
    pub kill_switch_active: bool,
    pub last_active_workspace_id: Option<WorkspaceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfacePreference {
    Sidebar,
    FullTab,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: ThemePreference::Dark,
            surface_preference: SurfacePreference::Sidebar,
            timing: TimingPolicy::default(),
            kill_switch_active: false,
            last_active_workspace_id: None,
        }
    }
}

pub async fn load_settings() -> Option<UiSettings> {
    let value = webextension::storage_local_get("ui_settings").await.ok()?;
    if value.is_undefined() || value.is_null() {
        return Some(UiSettings::default());
    }

    serde_wasm_bindgen::from_value(value).ok()
}

pub async fn save_settings(settings: &UiSettings) -> bool {
    let Ok(value) = serde_wasm_bindgen::to_value(settings) else {
        return false;
    };

    webextension::storage_local_set("ui_settings", value)
        .await
        .is_ok()
}

pub async fn get_storage_usage() -> (u64, u64) {
    let used = webextension::storage_local_get_bytes_in_use()
        .await
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0) as u64;

    // Chrome and Firefox differ here; use the current known bytes and treat quota as unlimited.
    (used, 0)
}
