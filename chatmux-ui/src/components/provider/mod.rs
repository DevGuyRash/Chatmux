//! Provider identity components (§2).
//!
//! Renders provider icons, colored chips, health dots, and status rows
//! using the provider identity color system.

pub mod health_badge;
pub mod provider_chip;
pub mod provider_dot;
pub mod provider_icon;
pub mod provider_status_row;

/// Provider identifier enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Provider {
    Gpt,
    Gemini,
    Grok,
    Claude,
    User,
    System,
}

impl Provider {
    /// Map from chatmux-common ProviderId to the UI Provider enum.
    pub fn from_provider_id(id: chatmux_common::ProviderId) -> Self {
        match id {
            chatmux_common::ProviderId::Gpt => Self::Gpt,
            chatmux_common::ProviderId::Gemini => Self::Gemini,
            chatmux_common::ProviderId::Grok => Self::Grok,
            chatmux_common::ProviderId::Claude => Self::Claude,
            chatmux_common::ProviderId::User => Self::User,
            chatmux_common::ProviderId::System => Self::System,
        }
    }

    pub fn to_provider_id(self) -> chatmux_common::ProviderId {
        match self {
            Self::Gpt => chatmux_common::ProviderId::Gpt,
            Self::Gemini => chatmux_common::ProviderId::Gemini,
            Self::Grok => chatmux_common::ProviderId::Grok,
            Self::Claude => chatmux_common::ProviderId::Claude,
            Self::User => chatmux_common::ProviderId::User,
            Self::System => chatmux_common::ProviderId::System,
        }
    }

    /// Display label — never abbreviated in UI text (§2.1).
    pub fn label(&self) -> &'static str {
        match self {
            Self::Gpt => "ChatGPT",
            Self::Gemini => "Gemini",
            Self::Grok => "Grok",
            Self::Claude => "Claude",
            Self::User => "You",
            Self::System => "System",
        }
    }

    /// CSS custom property prefix for this provider's color tokens.
    pub fn color_prefix(&self) -> &'static str {
        match self {
            Self::Gpt => "provider-gpt",
            Self::Gemini => "provider-gemini",
            Self::Grok => "provider-grok",
            Self::Claude => "provider-claude",
            Self::User => "provider-user",
            Self::System => "provider-system",
        }
    }

    /// CSS var reference for the solid color.
    pub fn solid_color(&self) -> String {
        format!("var(--{}-solid)", self.color_prefix())
    }

    /// CSS var reference for the muted background.
    pub fn muted_color(&self) -> String {
        format!("var(--{}-muted)", self.color_prefix())
    }

    /// CSS var reference for the text color.
    pub fn text_color(&self) -> String {
        format!("var(--{}-text)", self.color_prefix())
    }

    /// CSS var reference for the border color.
    pub fn border_color(&self) -> String {
        format!("var(--{}-border)", self.color_prefix())
    }

    /// Placeholder icon character.
    pub fn icon_char(&self) -> &'static str {
        match self {
            Self::Gpt => "⬡",
            Self::Gemini => "✦",
            Self::Grok => "⚡",
            Self::Claude => "◉",
            Self::User => "👤",
            Self::System => "⚙",
        }
    }
}

/// Provider health state (§3.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HealthState {
    Ready,
    Composing,
    Sending,
    Generating,
    Completed,
    Disconnected,
    PermissionMissing,
    LoginRequired,
    DomMismatch,
    Blocked,
    RateLimited,
    SendFailed,
    CaptureUncertain,
    DegradedManualOnly,
}

impl HealthState {
    /// Status color token for this health state.
    pub fn status_color(&self) -> &'static str {
        match self {
            Self::Ready | Self::Composing | Self::Sending | Self::Completed => "status-success",
            Self::Generating => "status-info",
            Self::Disconnected => "status-neutral",
            Self::PermissionMissing | Self::LoginRequired | Self::RateLimited
            | Self::CaptureUncertain | Self::DegradedManualOnly => "status-warning",
            Self::DomMismatch | Self::Blocked | Self::SendFailed => "status-error",
        }
    }

    /// Display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Composing => "Composing",
            Self::Sending => "Sending",
            Self::Generating => "Generating",
            Self::Completed => "Completed",
            Self::Disconnected => "Disconnected",
            Self::PermissionMissing => "Permission Missing",
            Self::LoginRequired => "Login Required",
            Self::DomMismatch => "DOM Mismatch",
            Self::Blocked => "Blocked",
            Self::RateLimited => "Rate Limited",
            Self::SendFailed => "Send Failed",
            Self::CaptureUncertain => "Capture Uncertain",
            Self::DegradedManualOnly => "Manual Only",
        }
    }

    /// Icon placeholder.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Ready => "●",
            Self::Composing => "✎",
            Self::Sending => "↗",
            Self::Generating => "⟳",
            Self::Completed => "✔",
            Self::Disconnected => "⛓",
            Self::PermissionMissing => "🔒",
            Self::LoginRequired => "👤",
            Self::DomMismatch => "⚠",
            Self::Blocked => "⊘",
            Self::RateLimited => "⏱",
            Self::SendFailed => "✖",
            Self::CaptureUncertain => "?",
            Self::DegradedManualOnly => "✋",
        }
    }
}
