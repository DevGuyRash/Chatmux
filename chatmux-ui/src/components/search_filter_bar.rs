//! Search and filter bar component (§3.22).
//!
//! Appears at the top of the message log. Toggled by search icon.
//! Search input with in-place highlighting, result counter, navigation.
//! Filter row with participant, role, run, round range, status, and tag filters.

use leptos::prelude::*;

// Button imports reserved for filter chip actions (Phase 9 polish).

/// Search and filter bar component.
#[component]
pub fn SearchFilterBar(
    /// Current search query.
    query: ReadSignal<String>,
    /// Set the search query.
    set_query: WriteSignal<String>,
    /// Whether the search bar is visible.
    is_active: ReadSignal<bool>,
    /// Whether the filter row is expanded.
    show_filters: ReadSignal<bool>,
    /// Toggle filter row visibility.
    set_show_filters: WriteSignal<bool>,
    /// Total result count.
    result_count: ReadSignal<u32>,
    /// Current result index (1-based).
    current_result: ReadSignal<u32>,
    /// Navigate to next result.
    on_next: impl Fn() + 'static + Copy + Send,
    /// Navigate to previous result.
    on_prev: impl Fn() + 'static + Copy + Send,
    /// Close the search bar.
    on_close: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        {move || is_active.get().then(|| view! {
            <div class="search-filter-bar"
                 style="border-bottom: 1px solid var(--border-subtle); \
                        background: var(--surface-raised);">

                // Search input row
                <div class="flex items-center gap-2 px-4 py-3">
                    // Search icon
                    <span class="text-secondary" style="font-size: 14px; flex-shrink: 0;">"🔍"</span>

                    // Input
                    <input
                        class="type-body flex-1"
                        type="text"
                        placeholder="Search messages…"
                        style="\
                            background: var(--surface-sunken); \
                            border: 1px solid var(--border-default); \
                            border-radius: var(--radius-md); \
                            padding: var(--space-3) var(--space-4); \
                            color: var(--text-primary); \
                            min-width: 0;"
                        prop:value=move || query.get()
                        on:input=move |ev| set_query.set(event_target_value(&ev))
                    />

                    // Result counter
                    {move || {
                        let count = result_count.get();
                        (count > 0).then(|| {
                            let current = current_result.get();
                            view! {
                                <span class="type-caption text-secondary" style="white-space: nowrap;">
                                    {format!("{current} of {count}")}
                                </span>
                            }
                        })
                    }}

                    // Navigation arrows
                    <button
                        class="cursor-pointer"
                        style="background: none; border: none; color: var(--text-secondary); font-size: 12px;"
                        aria-label="Previous result"
                        on:click=move |_| on_prev()
                    >
                        "↑"
                    </button>
                    <button
                        class="cursor-pointer"
                        style="background: none; border: none; color: var(--text-secondary); font-size: 12px;"
                        aria-label="Next result"
                        on:click=move |_| on_next()
                    >
                        "↓"
                    </button>

                    // Filter toggle
                    <button
                        class="cursor-pointer relative"
                        style="background: none; border: none; color: var(--text-secondary); font-size: 14px;"
                        title="Toggle filters"
                        aria-label="Toggle filters"
                        on:click=move |_| set_show_filters.update(|v| *v = !*v)
                    >
                        "▽"
                    </button>

                    // Close
                    <button
                        class="cursor-pointer"
                        style="background: none; border: none; color: var(--text-secondary); font-size: 14px;"
                        aria-label="Close search"
                        on:click=move |_| on_close()
                    >
                        "✕"
                    </button>
                </div>

                // Filter row (expanded)
                {move || show_filters.get().then(|| view! {
                    <div class="flex items-center gap-3 px-4 py-2 flex-wrap"
                         style="border-top: 1px solid var(--border-subtle);">
                        <span class="type-caption text-secondary">"Filters:"</span>
                        // Participant, Role, Run, Status, Tags filter chips
                        // These will be fully implemented with the filter state
                        <span class="type-caption text-tertiary">"(filter controls placeholder)"</span>

                        <button
                            class="type-caption cursor-pointer"
                            style="margin-left: auto; color: var(--text-link); \
                                   background: none; border: none;"
                        >
                            "Clear Filters"
                        </button>
                    </div>
                })}
            </div>
        })}
    }
}
