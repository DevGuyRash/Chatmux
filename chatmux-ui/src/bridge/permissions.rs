//! Extension permissions bridge.

/// TODO(backend): Request host permission for a provider's origin.
/// Triggers the browser's permission prompt.
/// Returns true if the user granted the permission.
pub async fn request_host_permission(_provider_origin: &str) -> bool {
    log::warn!("STUB: request_host_permission");
    false
}

/// TODO(backend): Check if host permission is granted for a provider's origin.
pub async fn check_host_permission(_provider_origin: &str) -> bool {
    log::warn!("STUB: check_host_permission");
    false
}
