//! Sidebar layout (~360px).
//!
//! Stack-based navigation model. The user navigates deeper into views
//! and uses a back action to return. All content is in a single
//! vertical column.
//!
//! Navigation stack:
//! 1. Workspace list (top-level)
//! 2. Active workspace (message log + composer)
//! 3. Sub-views push as full-width overlays

use leptos::prelude::*;

/// The views that can be pushed onto the sidebar navigation stack.
#[derive(Clone, Debug, PartialEq)]
pub enum SidebarView {
    /// Top-level workspace list.
    WorkspaceList,
    /// Active workspace with message log and composer.
    ActiveWorkspace {
        workspace_id: uuid::Uuid,
    },
    /// Message inspection overlay.
    MessageInspection {
        message_id: uuid::Uuid,
    },
    /// Provider bindings view.
    ProviderBindings,
    /// Edge policy / routing editor.
    Routing,
    /// Delivery cursor inspector.
    CursorInspector,
    /// Template manager.
    Templates,
    /// Pinned summary manager.
    Summaries,
    /// Export dialog.
    Export,
    /// Diagnostics panel.
    Diagnostics,
    /// Run configuration sheet.
    RunConfig,
    /// Between-rounds review.
    BetweenRoundsReview,
    /// Settings page.
    Settings,
    /// Keyboard shortcut reference.
    KeyboardShortcuts,
}

/// Context type for the sidebar navigation stack.
#[derive(Clone)]
pub struct SidebarNav {
    /// The navigation stack.
    pub stack: ReadSignal<Vec<SidebarView>>,
    /// Push a view onto the stack.
    pub push: WriteSignal<Vec<SidebarView>>,
}

impl SidebarNav {
    /// Push a new view onto the navigation stack.
    pub fn navigate(&self, view: SidebarView) {
        self.push.update(|stack| {
            stack.push(view);
        });
    }

    /// Pop the top view from the stack (go back).
    pub fn back(&self) {
        self.push.update(|stack| {
            if stack.len() > 1 {
                stack.pop();
            }
        });
    }

    /// Get the current (topmost) view.
    pub fn current(&self) -> SidebarView {
        self.stack.get_untracked().last().cloned()
            .unwrap_or(SidebarView::WorkspaceList)
    }
}

/// Sidebar layout component.
#[component]
pub fn SidebarLayout() -> impl IntoView {
    let (stack, set_stack) = signal(vec![SidebarView::WorkspaceList]);

    let nav = SidebarNav {
        stack,
        push: set_stack,
    };
    provide_context(nav.clone());

    view! {
        <div class="sidebar-layout flex flex-col h-full surface-base"
             style="max-width: 500px; width: 100%;">
            {move || {
                let current = stack.get().last().cloned()
                    .unwrap_or(SidebarView::WorkspaceList);
                match current {
                    SidebarView::WorkspaceList => view! {
                        <div class="flex flex-col h-full items-center justify-center p-6">
                            <p class="type-title text-primary">"Chatmux"</p>
                            <p class="type-body text-secondary" style="margin-top: var(--space-4);">
                                "Sidebar layout — workspace list placeholder"
                            </p>
                        </div>
                    }.into_any(),
                    SidebarView::Settings => view! {
                        <div class="flex flex-col h-full p-6">
                            <p class="type-title text-primary">"Settings"</p>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <div class="flex flex-col h-full items-center justify-center p-6">
                            <p class="type-body text-secondary">"View placeholder"</p>
                        </div>
                    }.into_any(),
                }
            }}
        </div>
    }
}
