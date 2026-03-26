//! Global header bar for the full-tab layout.
//!
//! 56px tall. Contains:
//! - Left: Chatmux logo/wordmark + workspace name breadcrumb
//! - Right: Global diagnostics indicator + settings gear + kill switch

use leptos::prelude::*;

/// Global header bar component.
#[component]
pub fn GlobalHeader() -> impl IntoView {
    // TODO(backend): Subscribe to the active workspace name to display in breadcrumb.
    // TODO(backend): Subscribe to unread diagnostic event count for the badge.
    // TODO(backend): Subscribe to kill switch state (active/inactive).

    view! {
        <header
            class="global-header flex items-center justify-between px-6 select-none"
            style="height: 56px; min-height: 56px; \
                   background: var(--surface-raised); \
                   border-bottom: 1px solid var(--border-subtle);"
        >
            // Left: Logo + workspace breadcrumb
            <div class="flex items-center gap-4">
                <span
                    class="type-subtitle"
                    style="color: var(--accent-primary); font-weight: 700;"
                >
                    "Chatmux"
                </span>
                // Workspace breadcrumb will appear here when a workspace is active
            </div>

            // Right: Diagnostics + Settings + Kill switch
            <div class="flex items-center gap-3">
                // Diagnostics indicator
                <button
                    class="flex items-center justify-center cursor-pointer transition-colors"
                    style="width: 28px; height: 28px; border-radius: var(--radius-md); \
                           color: var(--text-secondary);"
                    title="Diagnostics"
                    aria-label="Diagnostics"
                >
                    "🛡"
                </button>

                // Settings gear
                <button
                    class="flex items-center justify-center cursor-pointer transition-colors"
                    style="width: 28px; height: 28px; border-radius: var(--radius-md); \
                           color: var(--text-secondary);"
                    title="Settings"
                    aria-label="Settings"
                >
                    "⚙"
                </button>

                // Kill switch
                <button
                    class="flex items-center justify-center cursor-pointer transition-colors"
                    style="width: 28px; height: 28px; border-radius: var(--radius-md); \
                           color: var(--text-secondary);"
                    title="Kill switch — halt all orchestration"
                    aria-label="Kill switch"
                >
                    "⏹"
                </button>
            </div>
        </header>
    }
}
