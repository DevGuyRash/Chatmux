//! Sent payload tab (§3.4).
//!
//! Shows the exact rendered text that was injected into the provider's input.
//! font-mono, type-code, surface-sunken, scrollable.

use leptos::prelude::*;

/// Sent payload tab content.
#[component]
pub fn SentPayloadTab(
    /// The sent payload text, if available.
    payload: ReadSignal<Option<String>>,
) -> impl IntoView {
    view! {
        {move || match payload.get() {
            Some(text) => view! {
                <pre
                    class="type-code surface-sunken"
                    style="padding: var(--space-4); border-radius: var(--radius-md); \
                           overflow-x: auto; white-space: pre-wrap; word-break: break-word;"
                >
                    {text}
                </pre>
            }.into_any(),
            None => view! {
                <p class="type-body text-secondary" style="text-align: center; padding: var(--space-7);">
                    "No dispatch payload — message was captured directly."
                </p>
            }.into_any(),
        }}
    }
}
