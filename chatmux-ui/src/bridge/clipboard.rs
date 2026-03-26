//! Clipboard API bridge.

/// TODO(backend): Write text to the system clipboard.
/// Uses navigator.clipboard.writeText.
pub async fn write_clipboard(_text: &str) -> bool {
    log::warn!("STUB: write_clipboard");
    // In production, this will use:
    // web_sys::window().navigator().clipboard().write_text(text)
    false
}
