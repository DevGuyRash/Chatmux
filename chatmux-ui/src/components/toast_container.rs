//! Toast container — manages the toast notification stack (§3.21).
//!
//! Positioned at top-right of the extension viewport.
//! Maximum 3 visible toasts. Older toasts dismissed when 4th arrives.
//! Toasts stack vertically with space-2 gaps.

use leptos::prelude::*;

use super::primitives::toast::{Toast, ToastData, ToastKind};

/// Maximum visible toasts at once.
const MAX_VISIBLE_TOASTS: usize = 3;

/// Toast auto-dismiss durations in milliseconds.
fn auto_dismiss_ms(kind: &ToastKind) -> Option<u32> {
    match kind {
        ToastKind::Success | ToastKind::Info | ToastKind::Provider { .. } => Some(5000),
        ToastKind::Warning => Some(8000),
        ToastKind::Error => None, // Manual dismiss only
    }
}

/// Context for pushing toasts from anywhere in the app.
#[derive(Clone, Copy)]
pub struct ToastCtx {
    pub push: WriteSignal<Vec<ToastData>>,
    next_id: ReadSignal<u32>,
    set_next_id: WriteSignal<u32>,
}

impl ToastCtx {
    /// Push a new toast notification.
    pub fn toast(&self, kind: ToastKind, message: impl Into<String>) {
        let id = self.next_id.get_untracked();
        self.set_next_id.set(id + 1);

        let msg_text: String = message.into();

        // Announce to screen readers (§8.4)
        crate::a11y::announce(&msg_text);

        let data = ToastData {
            id,
            kind: kind.clone(),
            message: msg_text,
        };

        self.push.update(|toasts| {
            toasts.push(data);
            // Enforce max visible
            while toasts.len() > MAX_VISIBLE_TOASTS {
                toasts.remove(0);
            }
        });

        // Auto-dismiss timer
        if let Some(ms) = auto_dismiss_ms(&kind) {
            let push = self.push;
            gloo_timers::callback::Timeout::new(ms, move || {
                push.update(|toasts| {
                    toasts.retain(|t| t.id != id);
                });
            })
            .forget();
        }
    }
}

/// Toast container component — renders at the top-right of the viewport.
#[component]
pub fn ToastContainer() -> impl IntoView {
    let (toasts, set_toasts) = signal(Vec::<ToastData>::new());
    let (next_id, set_next_id) = signal(0u32);

    let ctx = ToastCtx {
        push: set_toasts,
        next_id,
        set_next_id,
    };
    provide_context(ctx);

    view! {
        <div
            class="toast-container fixed"
            style="top: var(--space-4); right: var(--space-4); \
                   z-index: var(--z-toast); \
                   display: flex; flex-direction: column; gap: var(--space-2); \
                   pointer-events: none;"
            aria-live="polite"
            aria-label="Notifications"
        >
            {move || toasts.get().into_iter().map(|data| {
                view! {
                    <div class="toast--entering" style="pointer-events: auto;">
                        <Toast
                            data=data
                            on_dismiss=move |id| {
                                set_toasts.update(|t| t.retain(|toast| toast.id != id));
                            }
                        />
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
