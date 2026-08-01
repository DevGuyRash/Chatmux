//! Modal dialog container (§3.20).
//!
//! Centered modal with shadow-lg, surface-raised, radius-lg.
//! Max width: 440px. Focus trap and backdrop scrim.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use super::button::{Button, ButtonVariant};

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
    #[prop(into)]
    open: Signal<bool>,
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
    let (was_open, set_was_open) = signal(false);

    // Focus management: capture the trigger, focus the dialog, and restore the trigger on close.
    Effect::new(move |_| {
        let is_open = open.get();
        let previously_open = was_open.get_untracked();
        if is_open && !previously_open {
            if let Some(active) = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.active_element())
            {
                let _ = active.set_attribute("data-chatmux-modal-return-focus", "true");
            }
            gloo_timers::callback::Timeout::new(50, || {
                crate::a11y::focus_element(".modal-dialog button, .modal-dialog input, .modal-dialog textarea, .modal-dialog select");
            }).forget();
        } else if !is_open
            && previously_open
            && let Some(document) = web_sys::window().and_then(|window| window.document())
            && let Ok(Some(element)) =
                document.query_selector("[data-chatmux-modal-return-focus='true']")
        {
            let _ = element.remove_attribute("data-chatmux-modal-return-focus");
            if let Ok(element) = element.dyn_into::<web_sys::HtmlElement>() {
                let _ = element.focus();
            }
        }
        set_was_open.set(is_open);
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
            // Only geometry is set inline; appearance belongs to
            // `.modal-dialog` in components.css.
            style=move || format!(
                "max-width: {max_width}px; display: {};",
                if open.get() { "block" } else { "none" },
            )
            on:keydown=move |ev| {
                if ev.key() == "Escape" {
                    on_close();
                } else {
                    crate::a11y::trap_focus(".modal-dialog", &ev);
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
                    <Button
                        variant=ButtonVariant::Secondary
                        on_click=Box::new(move |_| on_cancel())
                    >
                        {cancel_label.clone()}
                    </Button>
                    <Button
                        variant=if danger { ButtonVariant::Danger } else { ButtonVariant::Primary }
                        on_click=Box::new(move |_| on_confirm())
                    >
                        {confirm_label.clone()}
                    </Button>
                </div>
            </div>
        </Modal>
    }
}
