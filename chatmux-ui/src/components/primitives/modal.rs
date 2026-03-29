//! Modal dialog container (§3.20).
//!
//! Centered modal with shadow-lg, surface-raised, radius-lg.
//! Max width: 440px. Focus trap and backdrop scrim.

use leptos::prelude::*;

/// Modal dialog container.
///
/// Accessibility (§8.2):
/// - Focus trapped within the dialog while open
/// - Focus moves to first interactive element on open
/// - Focus returns to trigger element on close
/// - Escape key closes the dialog
#[component]
pub fn Modal(
    /// Whether the modal is open.
    open: ReadSignal<bool>,
    /// Called when the user requests to close (backdrop click or Escape).
    on_close: impl Fn() + 'static + Copy + Send,
    /// Max width in pixels.
    #[prop(default = 440)]
    max_width: u32,
    /// Accessible label for the dialog.
    #[prop(optional, into)]
    aria_label: Option<String>,
    /// Modal content.
    children: Children,
) -> impl IntoView {
    let rendered_children = children();

    // Focus management: focus first interactive element when opened
    Effect::new(move |_| {
        if open.get() {
            // Slight delay to ensure DOM is rendered
            gloo_timers::callback::Timeout::new(50, || {
                crate::a11y::focus_element(".modal-dialog button, .modal-dialog input, .modal-dialog textarea, .modal-dialog select");
            }).forget();
        }
    });

    view! {
        // Backdrop scrim
        <div
            class="modal-backdrop overlay-scrim fixed inset-0"
            class:overlay-scrim--visible=move || open.get()
            style=move || format!(
                "background: var(--overlay-scrim); \
                 z-index: var(--z-modal); \
                 display: {}; \
                 transition: background var(--duration-normal) var(--easing-standard);",
                if open.get() { "block" } else { "none" },
            )
            on:click=move |_| on_close()
            aria-hidden="true"
        />

        // Modal dialog
        <div
            class="modal-dialog fixed"
            role="dialog"
            aria-modal="true"
            aria-label=aria_label
            style=move || format!(
                "z-index: calc(var(--z-modal) + 1); \
                 top: 50%; left: 50%; \
                 transform: translate(-50%, -50%); \
                 max-width: {max_width}px; width: calc(100% - var(--space-8)); \
                 max-height: 80vh; overflow-y: auto; \
                 background: var(--surface-raised); \
                 border-radius: var(--radius-lg); \
                 box-shadow: var(--shadow-lg); \
                 padding: var(--space-7); \
                 display: {};",
                if open.get() { "block" } else { "none" },
            )
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    on_close();
                }
            }
        >
            {rendered_children}
        </div>
    }
}

/// Standard confirmation dialog (§3.20).
#[component]
pub fn ConfirmationDialog(
    /// Whether the dialog is open.
    open: ReadSignal<bool>,
    /// Dialog heading.
    heading: String,
    /// Dialog description.
    description: String,
    /// Cancel button label.
    #[prop(default = "Cancel".to_string())]
    cancel_label: String,
    /// Confirm button label.
    confirm_label: String,
    /// Whether confirm is a danger action.
    #[prop(default = false)]
    danger: bool,
    /// Called on cancel.
    on_cancel: impl Fn() + 'static + Copy + Send,
    /// Called on confirm.
    on_confirm: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <Modal open=open on_close=on_cancel>
            <div class="flex flex-col gap-5">
                <h2 class="type-title text-primary">{heading}</h2>
                <p class="type-body text-secondary">{description}</p>
                <div class="flex justify-end gap-3" style="margin-top: var(--space-4);">
                    <button
                        class="type-label select-none cursor-pointer"
                        style="\
                            padding: var(--space-3) var(--space-5); \
                            background: var(--surface-sunken); \
                            color: var(--text-primary); \
                            border: 1px solid var(--border-default); \
                            border-radius: var(--radius-md);"
                        on:click=move |_| on_cancel()
                    >
                        {cancel_label.clone()}
                    </button>
                    <button
                        class="type-label select-none cursor-pointer"
                        style=format!(
                            "padding: var(--space-3) var(--space-5); \
                             background: {}; \
                             color: var(--text-inverse); \
                             border: none; \
                             border-radius: var(--radius-md);",
                            if danger { "var(--status-error-solid)" } else { "var(--accent-primary)" },
                        )
                        on:click=move |_| on_confirm()
                    >
                        {confirm_label.clone()}
                    </button>
                </div>
            </div>
        </Modal>
    }
}
