//! Section header component — standard header bar with optional back button.
//!
//! Replaces the repeated pattern of padding + border-bottom + title + trailing actions.

use leptos::prelude::*;

/// Standard section/screen header bar.
#[component]
pub fn SectionHeader(
    /// Header title text.
    title: String,
    /// If provided, renders a back arrow button.
    #[prop(optional)]
    on_back: Option<Box<dyn Fn() + Send>>,
    /// Use compact padding (space-4 instead of space-5).
    #[prop(default = false)]
    compact: bool,
    /// Optional trailing actions slot.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let header_class = if compact {
        "section-header-compact flex items-center gap-3"
    } else {
        "section-header flex items-center gap-3"
    };

    view! {
        <div class=header_class>
            {on_back.map(|handler| view! {
                <button
                    class="type-body text-secondary cursor-pointer select-none"
                    aria-label="Go back"
                    on:click=move |_| handler()
                >
                    "←"
                </button>
            })}
            <h2 class="type-title text-primary">{title}</h2>
            <span class="flex-1"></span>
            {children.map(|c| c())}
        </div>
    }
}
