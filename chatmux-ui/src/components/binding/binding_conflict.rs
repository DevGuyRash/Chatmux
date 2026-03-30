//! Multi-workspace binding conflict (§3.28).
//!
//! Confirmation popover when a tab is already bound in another workspace.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};

/// Binding conflict popover.
#[component]
pub fn BindingConflict(
    /// Name of the other workspace holding the binding.
    other_workspace: String,
    /// Called to steal the binding.
    on_bind_here: impl Fn() + 'static + Copy + Send,
    /// Called to cancel.
    on_cancel: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <div
            class="binding-conflict surface-overlay p-5 rounded-md shadow-md"
            style="max-width: 320px;"
        >
            <p class="type-body text-primary mb-3">
                {format!("This tab is already bound to workspace '{}'.", other_workspace)}
            </p>
            <p class="type-caption text-secondary mb-4">
                "Binding it here will unbind it from that workspace. The other workspace's provider will become disconnected."
            </p>
            <div class="flex gap-2 justify-end">
                <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                        on_click=Box::new(move |_| on_cancel())>
                    "Cancel"
                </Button>
                <Button variant=ButtonVariant::Primary size=ButtonSize::Small
                        on_click=Box::new(move |_| on_bind_here())>
                    "Bind Here"
                </Button>
            </div>
        </div>
    }
}
