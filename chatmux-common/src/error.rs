//! Error definitions shared by all Chatmux crates.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Error)]
pub enum ChatmuxError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("adapter error: {0}")]
    Adapter(String),
    #[error("routing error: {0}")]
    Routing(String),
    #[error("export error: {0}")]
    Export(String),
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Error)]
pub enum AdapterError {
    #[error("dom mismatch: {detail}")]
    DomMismatch { detail: String },
    #[error("permission missing: {detail}")]
    PermissionMissing { detail: String },
    #[error("login required: {detail}")]
    LoginRequired { detail: String },
    #[error("blocked: {detail}")]
    Blocked { detail: String },
    #[error("rate limited: {detail}")]
    RateLimited { detail: String },
    #[error("send failed: {detail}")]
    SendFailed { detail: String },
    #[error("capture uncertain: {detail}")]
    CaptureUncertain { detail: String },
    #[error("not found: {detail}")]
    NotFound { detail: String },
    #[error("unsupported: {detail}")]
    Unsupported { detail: String },
}

impl From<AdapterError> for ChatmuxError {
    fn from(value: AdapterError) -> Self {
        Self::Adapter(value.to_string())
    }
}
