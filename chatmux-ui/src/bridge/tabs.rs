//! Extension tabs API bridge.

/// TODO(backend): Open a new browser tab to the given URL.
/// Used for "Open {Provider} Tab" action on binding cards.
pub async fn open_tab(_url: &str) -> bool {
    log::warn!("STUB: open_tab");
    false
}

/// TODO(backend): List open tabs matching a URL pattern.
/// Used for manual provider binding — shows candidate tabs.
/// Returns a list of (tab_id, title, url).
pub async fn list_matching_tabs(_url_pattern: &str) -> Vec<(u32, String, String)> {
    log::warn!("STUB: list_matching_tabs");
    vec![]
}
