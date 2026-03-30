//! Truncation warning component (§3.26).
//!
//! Inline warning banner when a payload exceeds the soft character limit.
//! status-warning-muted background, status-warning-border left border.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};

/// Truncation warning banner.
#[component]
pub fn TruncationWarning(
    /// Target provider name.
    target_name: String,
    /// Current character count.
    current_chars: u32,
    /// Soft character limit.
    limit: u32,
    /// Called to auto-trim oldest context blocks.
    on_auto_trim: impl Fn() + 'static + Copy + Send,
    /// Called to dismiss the warning.
    on_dismiss: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <div
            class="truncation-warning flex flex-col gap-3"
            role="alert"
            style="\
                padding: var(--space-4); \
                background: var(--status-warning-muted); \
                border-left: 3px solid var(--status-warning-border); \
                border-radius: var(--radius-md);"
        >
            <div class="flex items-start gap-2">
                <span style="color: var(--status-warning-text); flex-shrink: 0;">"⚠"</span>
                <span class="type-body text-primary">
                    {format!(
                        "This package exceeds the soft limit for {} ({} / {} characters).",
                        target_name, current_chars, limit
                    )}
                </span>
            </div>
            <div class="flex items-center gap-2">
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Small
                    on_click=Box::new(move |_| on_auto_trim())
                >
                    "Auto-trim oldest"
                </Button>
                <button
                    class="type-caption cursor-pointer"
                    style="color: var(--text-link); background: none; border: none;"
                    on:click=move |_| on_dismiss()
                >
                    "Dismiss"
                </button>
            </div>
        </div>
    }
}
