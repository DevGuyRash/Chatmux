//! Badge component.
//!
//! Pill-shaped label used for mode badges, strategy badges, round badges,
//! status indicators, and provider attribution.

use leptos::prelude::*;

/// Badge visual variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// Default: surface-sunken background, text-primary.
    #[default]
    Default,
    /// Accent: accent-primary tinted background.
    Accent,
    /// Success status.
    Success,
    /// Warning status.
    Warning,
    /// Error status.
    Error,
    /// Info status.
    Info,
    /// Neutral/gray.
    Neutral,
}

impl BadgeVariant {
    fn bg_color(&self) -> &'static str {
        match self {
            Self::Default => "var(--surface-sunken)",
            Self::Accent => "var(--surface-selected)",
            Self::Success => "var(--status-success-muted)",
            Self::Warning => "var(--status-warning-muted)",
            Self::Error => "var(--status-error-muted)",
            Self::Info => "var(--status-info-muted)",
            Self::Neutral => "var(--status-neutral-muted)",
        }
    }

    fn text_color(&self) -> &'static str {
        match self {
            Self::Default => "var(--text-primary)",
            Self::Accent => "var(--accent-primary)",
            Self::Success => "var(--status-success-text)",
            Self::Warning => "var(--status-warning-text)",
            Self::Error => "var(--status-error-text)",
            Self::Info => "var(--status-info-text)",
            Self::Neutral => "var(--status-neutral-text)",
        }
    }
}

/// Badge component — pill-shaped label.
#[component]
pub fn Badge(
    /// Visual variant.
    #[prop(default = BadgeVariant::Default)]
    variant: BadgeVariant,
    /// Badge content.
    children: Children,
) -> impl IntoView {
    view! {
        <span
            class="badge type-caption-strong select-none"
            style=format!(
                "display: inline-flex; align-items: center; gap: var(--space-1); \
                 padding: var(--space-1) var(--space-3); \
                 border-radius: var(--radius-xl); \
                 background: {}; color: {}; \
                 white-space: nowrap;",
                variant.bg_color(),
                variant.text_color(),
            )
        >
            {children()}
        </span>
    }
}
