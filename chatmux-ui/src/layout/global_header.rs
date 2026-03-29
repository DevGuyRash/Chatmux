//! Global header bar for the full-tab layout.
//!
//! 56px tall. Contains:
//! - Left: Chatmux logo/wordmark + workspace name breadcrumb
//! - Right: Global diagnostics indicator + settings gear + kill switch

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::icon::{Icon, IconKind};

/// Global header bar component.
#[component]
pub fn GlobalHeader(
    /// Name of the active workspace, if any.
    #[prop(into)]
    active_workspace_name: Signal<Option<String>>,
    /// Number of unread diagnostic events.
    #[prop(into)]
    diagnostics_count: Signal<usize>,
    /// Called when diagnostics button is clicked.
    on_diagnostics: impl Fn() + 'static + Copy + Send,
    /// Called when settings button is clicked.
    on_settings: impl Fn() + 'static + Copy + Send,
    /// Called when kill switch is clicked.
    on_kill: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <header
            class="global-header flex items-center justify-between px-6 select-none"
            style="height: 56px; min-height: 56px; \
                   background: var(--surface-raised); \
                   border-bottom: 1px solid var(--border-subtle);"
        >
            // Left: Logo + workspace breadcrumb
            <div class="flex items-center gap-3">
                <span
                    class="type-subtitle"
                    style="color: var(--accent-primary); font-weight: 700;"
                >
                    "Chatmux"
                </span>

                // Breadcrumb separator + workspace name
                {move || active_workspace_name.get().map(|name| view! {
                    <Icon kind=IconKind::ChevronRight size=14 color="var(--text-tertiary)".to_string() />
                    <span class="type-body text-primary truncate" style="max-width: 240px;">
                        {name}
                    </span>
                })}
            </div>

            // Right: Diagnostics + Settings + Kill switch
            <div class="flex items-center gap-2">
                // Diagnostics indicator with badge
                <div class="relative">
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        title="Diagnostics".to_string()
                        aria_label="Diagnostics".to_string()
                        on_click=Box::new(move |_| on_diagnostics())
                    >
                        <Icon kind=IconKind::Shield size=18 />
                    </Button>

                    // Badge overlay when there are unread events
                    {move || (diagnostics_count.get() > 0).then(|| {
                        let count = diagnostics_count.get();
                        let label = if count > 99 { "99+".to_string() } else { count.to_string() };
                        view! {
                            <span
                                class="type-label"
                                style="position: absolute; top: -2px; right: -2px; \
                                       min-width: 16px; height: 16px; \
                                       padding: 0 var(--space-1); \
                                       background: var(--status-warning-solid); \
                                       color: var(--text-inverse); \
                                       border-radius: var(--radius-full); \
                                       display: flex; align-items: center; justify-content: center; \
                                       font-size: 10px; font-weight: 700; \
                                       pointer-events: none;"
                            >
                                {label}
                            </span>
                        }
                    })}
                </div>

                // Settings gear
                <Button
                    variant=ButtonVariant::Icon
                    size=ButtonSize::Small
                    title="Settings".to_string()
                    aria_label="Settings".to_string()
                    on_click=Box::new(move |_| on_settings())
                >
                    <Icon kind=IconKind::Gear size=18 />
                </Button>

                // Kill switch
                <Button
                    variant=ButtonVariant::Icon
                    size=ButtonSize::Small
                    title="Kill switch — halt all orchestration".to_string()
                    aria_label="Kill switch".to_string()
                    on_click=Box::new(move |_| on_kill())
                >
                    <Icon kind=IconKind::StopOctagon size=18 />
                </Button>
            </div>
        </header>
    }
}
