//! Full-tab layout (~1200px+).
//!
//! Persistent navigation model with a global header bar, a left nav rail,
//! and a content area that can include a collapsible side panel.

use leptos::prelude::*;

use super::global_header::GlobalHeader;
use super::nav_rail::{NavRail, NavDestination};

/// The content currently displayed in the collapsible side panel.
#[derive(Clone, Debug, PartialEq)]
pub enum SidePanelContent {
    /// Message inspection panel.
    MessageInspection { message_id: uuid::Uuid },
    /// Provider binding cards.
    ProviderBindings,
    /// Delivery cursor inspector.
    CursorInspector,
    /// Diagnostics (workspace-scoped).
    Diagnostics,
}

/// Context for controlling the side panel from anywhere.
#[derive(Clone, Copy)]
pub struct SidePanelCtx {
    pub content: ReadSignal<Option<SidePanelContent>>,
    pub set_content: WriteSignal<Option<SidePanelContent>>,
}

impl SidePanelCtx {
    /// Open the side panel with the given content.
    pub fn open(&self, content: SidePanelContent) {
        self.set_content.set(Some(content));
    }

    /// Close the side panel.
    pub fn close(&self) {
        self.set_content.set(None);
    }

    /// Toggle the side panel.
    pub fn toggle(&self, content: SidePanelContent) {
        self.set_content.update(|current| {
            if current.as_ref() == Some(&content) {
                *current = None;
            } else {
                *current = Some(content);
            }
        });
    }
}

/// Full-tab layout component.
#[component]
pub fn FullTabLayout() -> impl IntoView {
    let (active_nav, set_active_nav) = signal(NavDestination::Workspaces);
    let (panel_content, set_panel_content) = signal(None::<SidePanelContent>);

    let side_panel_ctx = SidePanelCtx {
        content: panel_content,
        set_content: set_panel_content,
    };
    provide_context(side_panel_ctx);
    provide_context((active_nav, set_active_nav));

    view! {
        <div class="full-tab-layout flex flex-col h-full surface-base">
            // Global header bar (56px)
            <GlobalHeader />

            // Main area: nav rail + content
            <div class="flex flex-row flex-1 min-h-0">
                // Nav rail (56px fixed width)
                <NavRail
                    active=active_nav
                    on_navigate=move |dest| set_active_nav.set(dest)
                />

                // Content area
                <div class="flex-1 flex flex-row min-w-0">
                    // Main content
                    <div class="flex-1 flex flex-col min-w-0">
                        {move || match active_nav.get() {
                            NavDestination::Workspaces => view! {
                                <div class="flex flex-col h-full items-center justify-center p-8">
                                    <p class="type-display text-primary">"Chatmux"</p>
                                    <p class="type-body text-secondary" style="margin-top: var(--space-4);">
                                        "Full-tab layout — workspace list placeholder"
                                    </p>
                                </div>
                            }.into_any(),
                            NavDestination::ActiveWorkspace => view! {
                                <div class="flex flex-col h-full items-center justify-center">
                                    <p class="type-body text-secondary">"Active workspace placeholder"</p>
                                </div>
                            }.into_any(),
                            NavDestination::Routing => view! {
                                <div class="flex flex-col h-full items-center justify-center">
                                    <p class="type-body text-secondary">"Routing editor placeholder"</p>
                                </div>
                            }.into_any(),
                            NavDestination::Templates => view! {
                                <div class="flex flex-col h-full items-center justify-center">
                                    <p class="type-body text-secondary">"Template manager placeholder"</p>
                                </div>
                            }.into_any(),
                            NavDestination::Diagnostics => view! {
                                <div class="flex flex-col h-full items-center justify-center">
                                    <p class="type-body text-secondary">"Diagnostics placeholder"</p>
                                </div>
                            }.into_any(),
                            NavDestination::Settings => view! {
                                <div class="flex flex-col h-full items-center justify-center">
                                    <p class="type-body text-secondary">"Settings placeholder"</p>
                                </div>
                            }.into_any(),
                        }}
                    </div>

                    // Side panel (collapsible, 300-600px)
                    {move || {
                        panel_content.get().map(|_content| {
                            view! {
                                <div class="side-panel surface-raised"
                                     style="width: 360px; min-width: 300px; max-width: 600px; \
                                            border-left: 1px solid var(--border-subtle);">
                                    <div class="flex items-center justify-between p-5"
                                         style="border-bottom: 1px solid var(--border-subtle);">
                                        <span class="type-title text-primary">"Panel"</span>
                                        <button
                                            class="type-caption text-secondary cursor-pointer"
                                            on:click=move |_| set_panel_content.set(None)
                                        >
                                            "Close"
                                        </button>
                                    </div>
                                    <div class="p-5">
                                        <p class="type-body text-secondary">"Side panel content placeholder"</p>
                                    </div>
                                </div>
                            }
                        })
                    }}
                </div>
            </div>
        </div>
    }
}
