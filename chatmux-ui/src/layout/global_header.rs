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
    /// Whether global orchestration is currently halted.
    #[prop(into)]
    kill_switch_active: Signal<bool>,
    /// Called when diagnostics button is clicked.
    on_diagnostics: impl Fn() + 'static + Copy + Send,
    /// Called when settings button is clicked.
    on_settings: impl Fn() + 'static + Copy + Send,
    /// Called when kill switch is clicked.
    on_kill: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <header
            class="global-header flex items-center justify-between px-6 select-none border-b"
        >
            // Left: channel glyph + wordmark + workspace breadcrumb
            <div class="flex items-center gap-4 min-w-0">
                <div class="brand flex items-center gap-3">
                    // Four bars, one per provider channel, converging into the
                    // accent rail. The mark states what the product does.
                    <span class="channel-glyph" aria-hidden="true">
                        <i class="channel-glyph__bar channel-glyph__bar--gpt" />
                        <i class="channel-glyph__bar channel-glyph__bar--gemini" />
                        <i class="channel-glyph__bar channel-glyph__bar--grok" />
                        <i class="channel-glyph__bar channel-glyph__bar--claude" />
                    </span>
                    <span class="wordmark">"Chatmux"</span>
                </div>

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
                        size=ButtonSize::Medium
                        title="Diagnostics".to_string()
                        aria_label="Diagnostics".to_string()
                        on_click=Box::new(move |_| on_diagnostics())
                    >
                        <Icon kind=IconKind::Shield size=20 />
                    </Button>

                    // Badge overlay when there are unread events
                    {move || (diagnostics_count.get() > 0).then(|| {
                        let count = diagnostics_count.get();
                        let label = if count > 99 { "99+".to_string() } else { count.to_string() };
                        view! {
                            <span class="count-badge" aria-hidden="true">{label}</span>
                        }
                    })}
                </div>

                // Settings gear
                <Button
                    variant=ButtonVariant::Icon
                    size=ButtonSize::Medium
                    title="Settings".to_string()
                    aria_label="Settings".to_string()
                    on_click=Box::new(move |_| on_settings())
                >
                    <Icon kind=IconKind::Gear size=20 />
                </Button>

                // Kill switch
                <Button
                    variant=ButtonVariant::Icon
                    size=ButtonSize::Medium
                    title="Kill switch — halt all orchestration".to_string()
                    aria_label="Toggle global kill switch".to_string()
                    aria_pressed=Signal::derive(move || Some(kill_switch_active.get()))
                    danger_active=kill_switch_active
                    on_click=Box::new(move |_| on_kill())
                >
                    <Icon kind=IconKind::StopOctagon size=20 />
                </Button>
                {move || kill_switch_active.get().then(|| view! {
                    <span class="halt-pill" role="status">
                        <i class="halt-pill__dot" aria-hidden="true" />
                        "Halted"
                    </span>
                })}
            </div>
        </header>
    }
}
