//! Extension tabs API bridge.

use serde::Deserialize;

use crate::bridge::webextension;

#[derive(Debug, Deserialize)]
struct RawTab {
    id: Option<u32>,
    title: Option<String>,
    url: Option<String>,
}

/// Open a new browser tab to the given URL.
pub async fn open_tab(url: &str) -> bool {
    webextension::tabs_open(url).await.is_ok()
}

/// List open tabs matching a URL pattern.
pub async fn list_matching_tabs(url_pattern: &str) -> Vec<(u32, String, String)> {
    let Ok(patterns) = serde_wasm_bindgen::to_value(&vec![url_pattern.to_owned()]) else {
        return Vec::new();
    };
    let Ok(value) = webextension::tabs_query(patterns).await else {
        return Vec::new();
    };

    serde_wasm_bindgen::from_value::<Vec<RawTab>>(value)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|tab| Some((tab.id?, tab.title.unwrap_or_default(), tab.url.unwrap_or_default())))
        .collect()
}
