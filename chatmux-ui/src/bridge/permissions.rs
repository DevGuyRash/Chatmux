//! Extension permissions bridge.

use js_sys::Array;

use crate::bridge::webextension;

fn origins_arg(provider_origins: &[&str]) -> wasm_bindgen::JsValue {
    let origins = Array::new();
    for origin in provider_origins {
        origins.push(&wasm_bindgen::JsValue::from_str(origin));
    }
    origins.into()
}

/// Request every host permission needed by a provider adapter.
pub async fn request_host_permissions(provider_origins: &[&str]) -> bool {
    webextension::permissions_request_origins(origins_arg(provider_origins))
        .await
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

/// Check whether every host permission needed by a provider adapter is granted.
pub async fn check_host_permissions(provider_origins: &[&str]) -> bool {
    webextension::permissions_contains_origins(origins_arg(provider_origins))
        .await
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}
