//! Shared JS interop helpers for WebExtension APIs.

use js_sys::Function;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/bridge/webextension.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub async fn runtime_send_message(message: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub fn runtime_add_listener(callback: &Function) -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn storage_local_get(key: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn storage_local_set(key: &str, value: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn storage_local_get_bytes_in_use() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn permissions_contains_origins(origins: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn permissions_request_origins(origins: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn commands_get_all() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn tabs_open(url: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn tabs_query(url_patterns: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn clipboard_write_text(text: &str) -> Result<JsValue, JsValue>;
}
