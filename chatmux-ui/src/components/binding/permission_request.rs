//! Permission request flow (§3.16).
//!
//! Focused overlay/modal shown when a provider needs host permission.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::modal::Modal;
use crate::components::provider::Provider;
use crate::components::provider::provider_icon::ProviderIcon;

/// Permission request overlay.
#[component]
pub fn PermissionRequest(
    /// Whether the overlay is shown.
    open: ReadSignal<bool>,
    /// The provider requesting permission.
    provider: Provider,
    /// Called when permission is granted.
    on_grant: impl Fn() + 'static + Copy + Send,
    /// Called when the user skips.
    on_skip: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    // TODO(backend): Call request_host_permission when "Grant Permission" is clicked.
    // The bridge function will trigger the browser's permission prompt.

    view! {
        <Modal open=open on_close=on_skip max_width=400>
            <div class="flex flex-col items-center gap-5 text-center">
                // Provider icon
                <div style=format!(
                    "width: 64px; height: 64px; border-radius: var(--radius-full); \
                     background: {}; display: flex; align-items: center; justify-content: center;",
                    provider.muted_color(),
                )>
                    <ProviderIcon provider=provider size=32 />
                </div>

                // Heading
                <h2 class="type-title text-primary">
                    {format!("Connect to {}", provider.label())}
                </h2>

                // Explanation
                <p class="type-body text-secondary">
                    {format!(
                        "Chatmux needs permission to access {} so it can read your conversations \
                         and send messages on your behalf. This permission can be revoked at any time \
                         in your browser's extension settings.",
                        provider.label()
                    )}
                </p>

                // Actions
                <div class="flex flex-col items-center gap-3 w-full">
                    <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| on_grant())>
                        "Grant Permission"
                    </Button>
                    <button
                        class="type-caption cursor-pointer"
                        style="color: var(--text-link); background: none; border: none;"
                        title="You can grant this later from the provider binding card."
                        on:click=move |_| on_skip()
                    >
                        "Skip"
                    </button>
                </div>
            </div>
        </Modal>
    }
}
