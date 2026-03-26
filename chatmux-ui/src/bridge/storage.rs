//! Extension storage.local bridge.

use chatmux_common::TimingPolicy;
use crate::theme::ThemePreference;

/// UI settings — stored in extension storage.local.
/// This is a UI-local type, not from chatmux-common.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UiSettings {
    pub theme: ThemePreference,
    pub surface_preference: SurfacePreference,
    pub timing: TimingPolicy,
    pub kill_switch_active: bool,
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
        }
    }
}

/// TODO(backend): Load settings from storage.local.
pub async fn load_settings() -> Option<UiSettings> {
    log::warn!("STUB: load_settings");
    Some(UiSettings::default())
}

/// TODO(backend): Save settings to storage.local.
pub async fn save_settings(_settings: &UiSettings) -> bool {
    log::warn!("STUB: save_settings");
    false
}

/// TODO(backend): Get estimated storage usage in bytes.
pub async fn get_storage_usage() -> (u64, u64) {
    log::warn!("STUB: get_storage_usage");
    (0, 0)
}
