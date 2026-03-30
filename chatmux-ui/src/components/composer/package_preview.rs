//! Package preview component (§3.6).
//!
//! Shows the fully rendered outbound payload. Editable text area.
//! font-mono, type-code, surface-sunken background.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant, ButtonSize};

/// Package preview panel.
#[component]
pub fn PackagePreview(
    /// The rendered payload text.
    payload: ReadSignal<String>,
    /// Called when payload is edited.
    on_edit: impl Fn(String) + 'static,
    /// Called to close the preview.
    on_close: impl Fn() + 'static + Copy + Send,
    /// Whether the payload has been manually modified.
    is_modified: ReadSignal<bool>,
    /// Character count.
    char_count: ReadSignal<u32>,
) -> impl IntoView {
    view! {
        <div
            class="package-preview flex flex-col"
            style=move || format!(
                "background: var(--surface-sunken); \
                 border: 1px solid {}; \
                 border-radius: var(--radius-md); \
                 max-height: 50%; overflow: hidden;",
                if is_modified.get() { "var(--status-warning-border)" } else { "var(--border-default)" },
            )
        >
            // Header
            <div class="flex items-center justify-between px-4 py-3 border-b">
                <span class="type-subtitle text-primary">"Outbound Package"</span>
                <button
                    class="cursor-pointer"
                    style="background: none; border: none; color: var(--text-secondary);"
                    on:click=move |_| on_close()
                >
                    "✕"
                </button>
            </div>

            // Editable payload
            <textarea
                class="type-code flex-1"
                style="\
                    padding: var(--space-4); \
                    background: transparent; \
                    color: var(--text-primary); \
                    font-family: var(--font-mono); \
                    border: none; resize: none; \
                    min-height: 100px; overflow-y: auto;"
                on:input=move |ev| {
                    on_edit(event_target_value(&ev));
                }
            >
                {move || payload.get()}
            </textarea>

            // Footer
            <div class="flex items-center justify-between px-4 py-2 border-t">
                <Button variant=ButtonVariant::Secondary size=ButtonSize::Small>
                    "Copy to Clipboard"
                </Button>
                <span class="type-caption text-secondary">
                    {move || format!("{} chars", char_count.get())}
                </span>
            </div>
        </div>
    }
}
