//! Full-tab layout (~1200px+).
//!
//! Persistent navigation model with a global header bar, a left nav rail,
//! and a content area that can include a collapsible side panel.

use leptos::prelude::*;

use crate::components::inspection::inspection_panel::InspectionPanel;
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::icon::{Icon, IconKind};
use crate::layout::screens::{
    ActiveWorkspaceScreen, DiagnosticsScreen, ProviderBindingsScreen, RoutingScreen,
    SettingsScreen, TemplatesScreen, WorkspaceListScreen,
};
use crate::models::MessageId;
use crate::state::app_state::AppState;
use crate::state::diagnostics_state::DiagnosticsState;
use crate::state::workspace_state::WorkspaceListState;

use super::global_header::GlobalHeader;
use super::nav_rail::{NavRail, NavDestination};

/// The content currently displayed in the collapsible side panel.
#[derive(Clone, Debug, PartialEq)]
pub enum SidePanelContent {
    /// Message inspection panel.
    MessageInspection { message_id: MessageId },
    /// Provider binding cards.
    ProviderBindings,
    /// Delivery cursor inspector.
    CursorInspector,
    /// Diagnostics (workspace-scoped).
    Diagnostics,
}

impl SidePanelContent {
    /// Display title for the side panel header.
    fn title(&self) -> &'static str {
        match self {
            Self::MessageInspection { .. } => "Message Inspection",
            Self::ProviderBindings => "Provider Settings",
            Self::CursorInspector => "Delivery Cursors",
            Self::Diagnostics => "Diagnostics",
        }
    }
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

    // Derive workspace name for the breadcrumb.
    let workspace_name = Signal::derive(move || {
        let workspace_state = expect_context::<WorkspaceListState>();
        workspace_state
            .snapshot
            .get()
            .and_then(|snapshot| snapshot.workspace.map(|ws| ws.name))
    });

    // Derive diagnostics count for the badge.
    let diagnostics_count = Signal::derive(move || {
        let diagnostics_state = expect_context::<DiagnosticsState>();
        diagnostics_state.events.get().len()
    });

    view! {
        <div class="full-tab-layout flex flex-col h-full surface-base">
            // Global header bar (56px)
            <GlobalHeader
                active_workspace_name=workspace_name
                diagnostics_count=diagnostics_count
                on_diagnostics=move || set_active_nav.set(NavDestination::Diagnostics)
                on_settings=move || set_active_nav.set(NavDestination::Settings)
                on_kill=move || {
                    // TODO(backend): Wire kill switch toggle via messaging::toggle_kill_switch()
                }
            />

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
                                <WorkspaceListScreen on_select=move |_workspace_id| {
                                    set_active_nav.set(NavDestination::ActiveWorkspace);
                                } />
                            }.into_any(),
                            NavDestination::ActiveWorkspace => view! {
                                <ActiveWorkspaceScreen on_back=move || set_active_nav.set(NavDestination::Workspaces) />
                            }.into_any(),
                            NavDestination::Routing => view! {
                                <RoutingScreen />
                            }.into_any(),
                            NavDestination::Templates => view! {
                                <TemplatesScreen />
                            }.into_any(),
                            NavDestination::Diagnostics => view! {
                                <DiagnosticsScreen />
                            }.into_any(),
                            NavDestination::Settings => view! {
                                <SettingsScreen />
                            }.into_any(),
                        }}
                    </div>

                    // Side panel (collapsible, 360px)
                    {move || {
                        let app_state = expect_context::<AppState>();
                        panel_content.get().map(|content| {
                            let panel_title = content.title();
                            view! {
                                <div class="side-panel surface-raised flex flex-col"
                                     style="width: 360px; min-width: 300px; max-width: 600px; \
                                            border-left: 1px solid var(--border-subtle);">
                                    // Panel header
                                    <div class="flex items-center justify-between"
                                         style="padding: var(--space-4) var(--space-5); \
                                                border-bottom: 1px solid var(--border-subtle);">
                                        <span class="type-title text-primary">{panel_title}</span>
                                        <Button
                                            variant=ButtonVariant::Icon
                                            size=ButtonSize::Small
                                            aria_label="Close panel".to_string()
                                            on_click=Box::new(move |_| set_panel_content.set(None))
                                        >
                                            <Icon kind=IconKind::Close size=16 />
                                        </Button>
                                    </div>

                                    // Panel content
                                    <div class="flex-1 overflow-y-auto p-5">
                                        {move || match panel_content.get() {
                                            Some(SidePanelContent::ProviderBindings) => view! {
                                                <ProviderBindingsScreen
                                                    on_close=move || set_panel_content.set(None)
                                                    show_header=false
                                                />
                                            }.into_any(),
                                            _ => match app_state.inspection.get() {
                                                Some(inspection) => inspection.message.map(|message| {
                                                    view! {
                                                        <InspectionPanel
                                                            message=message
                                                            sent_payload=inspection.sent_payload.clone()
                                                            raw_response=inspection.raw_response_text.clone()
                                                            network_capture=inspection.network_capture.clone()
                                                            on_close=move || set_panel_content.set(None)
                                                        />
                                                    }
                                                }).map(IntoAny::into_any).unwrap_or_else(|| view! {
                                                    <p class="type-body text-secondary">"Inspection data unavailable."</p>
                                                }.into_any()),
                                                None => view! {
                                                    <p class="type-body text-secondary">"No side panel content is loaded."</p>
                                                }.into_any(),
                                            }
                                        }}
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
