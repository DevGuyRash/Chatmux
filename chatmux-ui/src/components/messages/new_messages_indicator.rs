//! New messages indicator (§3.27).
//!
//! Floating pill at bottom of message log when user is scrolled up.
//! accent-primary background, text-inverse, radius-xl, shadow-md.

use leptos::prelude::*;

/// New messages indicator pill.
#[component]
pub fn NewMessagesIndicator(
    /// Number of new messages below viewport.
    count: ReadSignal<u32>,
    /// Called when clicked (scroll to bottom).
    on_click: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        {move || (count.get() > 0).then(|| view! {
            <button
                class="new-messages-indicator type-caption-strong select-none cursor-pointer"
                style="\
                    position: absolute; bottom: var(--space-4); \
                    left: 50%; transform: translateX(-50%); \
                    background: var(--accent-primary); \
                    color: var(--text-inverse); \
                    padding: var(--space-2) var(--space-5); \
                    border-radius: var(--radius-xl); \
                    box-shadow: var(--shadow-md); \
                    border: none; \
                    z-index: var(--z-raised); \
                    display: flex; align-items: center; gap: var(--space-2);"
                on:click=move |_| on_click()
            >
                <span style="font-size: 12px;">"↓"</span>
                {move || format!("{} new messages", count.get())}
            </button>
        })}
    }
}
