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
    /// Modifier class that selects the status edge colour in components.css.
    fn modifier_class(&self) -> &'static str {
        match self {
            Self::Success => "toast--success",
            Self::Info => "toast--info",
            Self::Warning => "toast--warning",
            Self::Error => "toast--error",
            Self::Provider { .. } => "toast--provider",
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
            class=format!("toast {}", data.kind.modifier_class())
            role="alert"
        >
            <span class="toast__icon" aria-hidden="true">{data.kind.icon()}</span>
            <span class="toast__message type-body flex-1">{data.message}</span>
            <button
                class="toast__dismiss cursor-pointer"
                aria-label="Dismiss notification"
                on:click=move |_| on_dismiss(id)
            >
                "✕"
            </button>
        </div>
    }
}
