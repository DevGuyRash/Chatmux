//! Clipboard API bridge.

use crate::bridge::webextension;

/// Write text to the system clipboard.
pub async fn write_clipboard(text: &str) -> bool {
    webextension::clipboard_write_text(text).await.is_ok()
}
