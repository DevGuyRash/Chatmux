//! Extension permissions bridge.

use js_sys::Array;

use crate::bridge::webextension;

fn origins_arg(provider_origin: &str) -> wasm_bindgen::JsValue {
    Array::of1(&wasm_bindgen::JsValue::from_str(provider_origin)).into()
}

/// Request host permission for a provider origin.
pub async fn request_host_permission(provider_origin: &str) -> bool {
    webextension::permissions_request_origins(origins_arg(provider_origin))
        .await
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

/// Check if host permission is granted for a provider origin.
pub async fn check_host_permission(provider_origin: &str) -> bool {
    webextension::permissions_contains_origins(origins_arg(provider_origin))
        .await
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}
