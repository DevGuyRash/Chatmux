//! Clipboard API bridge.

use crate::bridge::webextension;

/// Write text to the system clipboard.
pub async fn write_clipboard(text: &str) -> bool {
    webextension::clipboard_write_text(text).await.is_ok()
}

/// Download text using a browser-managed blob URL.
pub async fn download_text(filename: &str, mime_type: &str, body: &str) -> bool {
    webextension::download_text(filename, mime_type, body)
        .await
        .is_ok()
}
