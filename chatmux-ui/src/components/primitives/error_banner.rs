//! Inline error banner component (§3.19).
//!
//! status-error-muted background, status-error-border left border (3px),
//! space-4 padding. Icon: warning triangle.

use leptos::prelude::*;

/// Inline error banner for contextual error display.
#[component]
pub fn ErrorBanner(
    /// Error title.
    title: String,
    /// Error description.
    description: String,
    /// Optional action button label.
    #[prop(optional, into)]
    action_label: Option<String>,
    /// Optional action callback.
    #[prop(optional)]
    on_action: Option<Box<dyn Fn() + Send>>,
    /// Optional dismiss callback.
    #[prop(optional)]
    on_dismiss: Option<Box<dyn Fn() + Send>>,
) -> impl IntoView {
    view! {
        <div
            class="error-banner flex items-start gap-3"
            role="alert"
            style="\
                padding: var(--space-4); \
                background: var(--status-error-muted); \
                border-left: 3px solid var(--status-error-border); \
                border-radius: var(--radius-md);"
        >
            <span style="color: var(--status-error-text); flex-shrink: 0; margin-top: 1px;">
                "⚠"
            </span>
            <div class="flex-1 flex flex-col gap-1">
                <span class="type-body-strong" style="color: var(--status-error-text);">
                    {title}
                </span>
                <span class="type-body text-primary">{description}</span>
                {action_label.map(|label| {
                    let handler = on_action;
                    view! {
                        <button
                            class="type-caption-strong cursor-pointer"
                            style="color: var(--status-error-text); \
                                   background: none; border: none; \
                                   text-align: left; padding: 0; \
                                   margin-top: var(--space-2);"
                            on:click=move |_| {
                                if let Some(ref h) = handler {
                                    h();
                                }
                            }
                        >
                            {label}
                        </button>
                    }
                })}
            </div>
            {on_dismiss.map(|dismiss| view! {
                <button
                    class="cursor-pointer"
                    style="color: var(--text-secondary); background: none; \
                           border: none; padding: var(--space-1); flex-shrink: 0;"
                    aria-label="Dismiss"
                    on:click=move |_| dismiss()
                >
                    "✕"
                </button>
            })}
        </div>
    }
}
