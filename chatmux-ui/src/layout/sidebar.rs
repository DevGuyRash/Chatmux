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
//!
//! Structure:
//! - SidebarHeader: branding + icon buttons (only on WorkspaceList)
//! - Main content area (flex-1, stack-based routing)
//! - SidebarToolbar: always-visible bottom strip with nav icons

use leptos::prelude::*;

use crate::components::inspection::inspection_panel::InspectionPanel;
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::empty_state::EmptyState;
use crate::components::primitives::icon::{Icon, IconKind};
use crate::layout::screens::{
    ActiveWorkspaceScreen, DiagnosticsScreen, ProviderBindingsScreen, RoutingScreen,
    SettingsScreen, TemplatesScreen, WorkspaceListScreen,
};
use crate::models::{MessageId, WorkspaceId};
use crate::state::app_state::AppState;

/// The views that can be pushed onto the sidebar navigation stack.
#[derive(Clone, Debug, PartialEq)]
pub enum SidebarView {
    /// Top-level workspace list.
    WorkspaceList,
    /// Active workspace with message log and composer.
    ActiveWorkspace { workspace_id: WorkspaceId },
    /// Message inspection overlay.
    MessageInspection { message_id: MessageId },
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
#[derive(Clone, Copy)]
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
        self.stack
            .get_untracked()
            .last()
            .cloned()
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

    // Show header only on the top-level workspace list view.
    let show_header =
        Signal::derive(move || stack.get().last().cloned() == Some(SidebarView::WorkspaceList));

    view! {
        <div class="sidebar-layout flex flex-col h-full surface-base"
             style="max-width: 500px; width: 100%;">

            // Sidebar header (only on WorkspaceList)
            {move || show_header.get().then(|| view! {
                <SidebarHeader nav=nav />
            })}

            // Main content area
            <div class="flex-1 min-h-0 overflow-hidden">
                {move || {
                    let app_state = expect_context::<AppState>();
                    let current = stack.get().last().cloned()
                        .unwrap_or(SidebarView::WorkspaceList);
                    match current {
                        SidebarView::WorkspaceList => view! {
                            <WorkspaceListScreen on_select=move |workspace_id| {
                                nav.navigate(SidebarView::ActiveWorkspace { workspace_id });
                            } />
                        }.into_any(),
                        SidebarView::ActiveWorkspace { .. } => view! {
                            <ActiveWorkspaceScreen on_back=move || nav.back() />
                        }.into_any(),
                        SidebarView::Settings => view! {
                            <SettingsScreen />
                        }.into_any(),
                        SidebarView::Routing => view! {
                            <RoutingScreen />
                        }.into_any(),
                        SidebarView::Templates => view! {
                            <TemplatesScreen />
                        }.into_any(),
                        SidebarView::Diagnostics => view! {
                            <DiagnosticsScreen />
                        }.into_any(),
                        SidebarView::ProviderBindings => view! {
                            <ProviderBindingsScreen on_close=move || nav.back() />
                        }.into_any(),
                        SidebarView::MessageInspection { .. } => view! {
                            {move || {
                                match app_state.inspection.get() {
                                    Some(inspection) => inspection.message.map(|message| {
                                        view! {
                                            <InspectionPanel
                                                message=message
                                                sent_payload=inspection.sent_payload.clone()
                                                raw_response=inspection.raw_response_text.clone()
                                                network_capture=inspection.network_capture.clone()
                                                on_close=move || nav.back()
                                            />
                                        }
                                    }).map(IntoAny::into_any).unwrap_or_else(|| view! {
                                        <div class="flex items-center justify-center h-full p-6">
                                            <p class="type-body text-secondary">"Inspection data unavailable."</p>
                                        </div>
                                    }.into_any()),
                                    None => view! {
                                        <div class="flex items-center justify-center h-full p-6">
                                            <p class="type-body text-secondary">"Select a message to inspect it."</p>
                                        </div>
                                    }.into_any(),
                                }
                            }}
                        }.into_any(),
                        SidebarView::CursorInspector => view! {
                            <EmptyState
                                icon=IconKind::Crosshair
                                heading="Delivery Cursors"
                                description="Cursor inspector is available in full-tab mode. Expand the sidebar to access it."
                            />
                        }.into_any(),
                        SidebarView::Summaries => view! {
                            <EmptyState
                                icon=IconKind::Pin
                                heading="Pinned Summaries"
                                description="Summary management is available from the active workspace view."
                            />
                        }.into_any(),
                        SidebarView::Export => view! {
                            <EmptyState
                                icon=IconKind::Download
                                heading="Export"
                                description="Export options are available from the active workspace overflow menu."
                            />
                        }.into_any(),
                        SidebarView::RunConfig => view! {
                            <EmptyState
                                icon=IconKind::Gear
                                heading="Run Configuration"
                                description="Configure run options before starting from the active workspace."
                            />
                        }.into_any(),
                        SidebarView::BetweenRoundsReview => view! {
                            <EmptyState
                                icon=IconKind::Pause
                                heading="Between-Rounds Review"
                                description="Review options appear automatically between orchestration rounds."
                            />
                        }.into_any(),
                        SidebarView::KeyboardShortcuts => view! {
                            <KeyboardShortcutsView on_back=move || nav.back() />
                        }.into_any(),
                    }
                }}
            </div>

            // Bottom toolbar (always visible)
            <SidebarToolbar nav=nav />
        </div>
    }
}

/// Sidebar header: branding wordmark + utility buttons.
/// Visible only on the top-level WorkspaceList view.
#[component]
fn SidebarHeader(#[allow(unused)] nav: SidebarNav) -> impl IntoView {
    view! {
        <header
            class="flex items-center justify-between select-none border-b"
            style="padding: var(--space-4) var(--space-6); \
                   min-height: 48px; \
                   background: var(--surface-raised);"
        >
            <span
                class="type-subtitle"
                style="color: var(--accent-primary); font-weight: 700;"
            >
                "Chatmux"
            </span>

            // Open in full tab
            <Button
                variant=ButtonVariant::Icon
                size=ButtonSize::Small
                title="Open in full tab".to_string()
                aria_label="Open Chatmux in a full browser tab".to_string()
                on_click=Box::new(move |_| {
                    leptos::task::spawn_local(async move {
                        let extension_url = get_extension_ui_url();
                        let _ = crate::bridge::messaging::open_tab(&extension_url).await;
                    });
                })
            >
                <Icon kind=IconKind::ExternalLink size=16 />
            </Button>
        </header>
    }
}

/// Get the full URL to the extension's UI page (for opening in a tab).
fn get_extension_ui_url() -> String {
    let window = web_sys::window().expect("no window");
    let location = window.location();
    // In extension context, location.href is chrome-extension://<id>/ui/index.html
    // Just use that directly
    location
        .href()
        .unwrap_or_else(|_| "ui/index.html".to_string())
}

/// Sidebar bottom toolbar with navigation icons.
/// Always visible, pinned to the bottom of the sidebar.
#[component]
fn SidebarToolbar(nav: SidebarNav) -> impl IntoView {
    view! {
        <nav
            class="flex items-center justify-around select-none border-t"
            style="min-height: 44px; \
                   background: var(--surface-raised);"
            role="navigation"
            aria-label="Sidebar navigation"
        >
            <Button
                variant=ButtonVariant::Icon
                size=ButtonSize::Small
                title="Routing".to_string()
                aria_label="Routing".to_string()
                on_click=Box::new(move |_| nav.navigate(SidebarView::Routing))
            >
                <Icon kind=IconKind::GitBranch size=18 />
            </Button>

            <Button
                variant=ButtonVariant::Icon
                size=ButtonSize::Small
                title="Templates".to_string()
                aria_label="Templates".to_string()
                on_click=Box::new(move |_| nav.navigate(SidebarView::Templates))
            >
                <Icon kind=IconKind::Document size=18 />
            </Button>

            <Button
                variant=ButtonVariant::Icon
                size=ButtonSize::Small
                title="Diagnostics".to_string()
                aria_label="Diagnostics".to_string()
                on_click=Box::new(move |_| nav.navigate(SidebarView::Diagnostics))
            >
                <Icon kind=IconKind::Shield size=18 />
            </Button>

            <Button
                variant=ButtonVariant::Icon
                size=ButtonSize::Small
                title="Settings".to_string()
                aria_label="Settings".to_string()
                on_click=Box::new(move |_| nav.navigate(SidebarView::Settings))
            >
                <Icon kind=IconKind::Gear size=18 />
            </Button>
        </nav>
    }
}

/// Simple keyboard shortcut reference view.
#[component]
fn KeyboardShortcutsView(on_back: impl Fn() + 'static + Copy + Send) -> impl IntoView {
    let shortcuts = [
        ("Ctrl+Enter", "Send message"),
        ("Ctrl+Shift+Enter", "Send to all providers"),
        ("Escape", "Close panel / Go back"),
        ("Ctrl+/", "Toggle keyboard shortcuts"),
        ("Ctrl+K", "Quick command palette"),
        ("Ctrl+.", "Kill switch"),
    ];

    view! {
        <div class="flex flex-col h-full">
            <div class="flex items-center gap-3 border-b"
                 style="padding: var(--space-5) var(--space-6); \
                        background: var(--surface-raised);">
                <Button
                    variant=ButtonVariant::Icon
                    size=ButtonSize::Small
                    aria_label="Back".to_string()
                    on_click=Box::new(move |_| on_back())
                >
                    <Icon kind=IconKind::ArrowLeft size=18 />
                </Button>
                <span class="type-title text-primary">"Keyboard Shortcuts"</span>
            </div>

            <div class="flex flex-col" style="padding: var(--space-5) var(--space-6);">
                {shortcuts.into_iter().map(|(key, action)| {
                    view! {
                        <div class="flex items-center justify-between border-b"
                             style="padding: var(--space-3) 0;">
                            <span class="type-body text-primary">{action}</span>
                            <kbd
                                class="type-code"
                                style="padding: var(--space-1) var(--space-3); \
                                       background: var(--surface-sunken); \
                                       border: 1px solid var(--border-default); \
                                       border-radius: var(--radius-sm); \
                                       color: var(--text-secondary);"
                            >
                                {key}
                            </kbd>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
