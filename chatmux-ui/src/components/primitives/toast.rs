//! Toast notification component (§3.21).
//!
//! Appears at top-right of extension viewport. Max width 320px.
//! surface-raised, shadow-md, radius-md. Auto-dismiss: 5s info/success,
//! 8s warnings, manual dismiss for errors.

use leptos::prelude::*;

/// Toast notification type.
#[derive(Clone, Debug, PartialEq)]
pub enum ToastKind {
    Success,
    Info,
    Warning,
    Error,
    Provider { provider: String },
}

impl ToastKind {
    fn border_color(&self) -> &'static str {
        match self {
            Self::Success => "var(--status-success-border)",
            Self::Info => "var(--status-info-border)",
            Self::Warning => "var(--status-warning-border)",
            Self::Error => "var(--status-error-border)",
            Self::Provider { .. } => "var(--border-default)",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Success => "✔",
            Self::Info => "ℹ",
            Self::Warning => "⚠",
            Self::Error => "✖",
            Self::Provider { .. } => "●",
        }
    }
}

/// A single toast notification.
#[derive(Clone, Debug)]
pub struct ToastData {
    pub id: u32,
    pub kind: ToastKind,
    pub message: String,
}

/// Individual toast component.
#[component]
pub fn Toast(
    /// Toast data.
    data: ToastData,
    /// Called to dismiss this toast.
    on_dismiss: impl Fn(u32) + 'static,
) -> impl IntoView {
    let id = data.id;

    view! {
        <div
            class="toast flex items-start gap-3"
            role="alert"
            style=format!(
                "max-width: 320px; \
                 padding: var(--space-5); \
                 background: var(--surface-raised); \
                 border-radius: var(--radius-md); \
                 box-shadow: var(--shadow-md); \
                 border-left: 3px solid {};",
                data.kind.border_color(),
            )
        >
            <span style="flex-shrink: 0; margin-top: 1px;">{data.kind.icon()}</span>
            <span class="type-body text-primary flex-1">{data.message}</span>
            <button
                class="cursor-pointer"
                style="color: var(--text-tertiary); background: none; \
                       border: none; padding: 0; flex-shrink: 0;"
                aria-label="Dismiss notification"
                on:click=move |_| on_dismiss(id)
            >
                "✕"
            </button>
        </div>
    }
}
