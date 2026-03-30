//! Full-tab layout (~1200px+).
//!
//! Persistent navigation model with a global header bar, a left nav rail,
//! and a content area that can include a collapsible side panel.

use leptos::prelude::*;

use crate::bridge::messaging;
use crate::components::inspection::inspection_panel::InspectionPanel;
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::icon::{Icon, IconKind};
use crate::layout::screens::{
    ActiveWorkspaceScreen, DiagnosticsScreen, ProviderBindingsScreen, RoutingScreen,
    SettingsScreen, TemplatesScreen, WorkspaceListScreen,
};
use crate::models::{DiagnosticLevel, MessageId};
use crate::state::app_state::AppState;
use crate::state::binding_state::BindingState;
use crate::state::controller::dispatch_command_result;
use crate::state::diagnostics_state::DiagnosticsState;
use crate::state::message_state::MessageState;
use crate::state::run_state::ActiveRunState;
use crate::state::workspace_state::WorkspaceListState;

use super::global_header::GlobalHeader;
use super::nav_rail::{NavDestination, NavRail};

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
    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();
    let (active_nav, set_active_nav) = signal(NavDestination::Workspaces);
    let (panel_content, set_panel_content) = signal(None::<SidePanelContent>);
    let (handled_workspace_query, set_handled_workspace_query) = signal(false);

    let side_panel_ctx = SidePanelCtx {
        content: panel_content,
        set_content: set_panel_content,
    };
    provide_context(side_panel_ctx);
    provide_context((active_nav, set_active_nav));

    let navigate_to_active_workspace = move || {
        leptos::task::spawn_local(async move {
            if ensure_active_workspace_loaded(
                app_state,
                workspace_state,
                run_state,
                binding_state,
                message_state,
                diagnostics_state,
            )
            .await
            .is_some()
            {
                set_active_nav.set(NavDestination::ActiveWorkspace);
            } else {
                set_active_nav.set(NavDestination::Workspaces);
            }
        });
    };

    Effect::new(move |_| {
        if handled_workspace_query.get() {
            return;
        }

        let Some(workspace_id) = workspace_query_workspace_id() else {
            set_handled_workspace_query.set(true);
            return;
        };

        set_handled_workspace_query.set(true);
        set_active_nav.set(NavDestination::ActiveWorkspace);
        leptos::task::spawn_local(async move {
            dispatch_command_result(
                app_state,
                workspace_state,
                run_state,
                binding_state,
                message_state,
                diagnostics_state,
                messaging::open_workspace(workspace_id).await,
            );
        });
    });

    // Close side panel whenever the user navigates to a different page.
    Effect::new(move |_| {
        let _ = active_nav.get();
        set_panel_content.set(None);
    });

    // Derive workspace name for the breadcrumb.
    let workspace_name = Signal::derive(move || {
        let workspace_state = expect_context::<WorkspaceListState>();
        workspace_state
            .snapshot
            .get()
            .and_then(|snapshot| snapshot.workspace.map(|ws| ws.name))
    });

    // Derive diagnostics count for the badge — respects level filters.
    let diagnostics_count = Signal::derive(move || {
        let ds = expect_context::<DiagnosticsState>();
        ds.events
            .get()
            .iter()
            .filter(|e| match e.level {
                DiagnosticLevel::Critical => ds.filter_critical.get(),
                DiagnosticLevel::Warning => ds.filter_warning.get(),
                DiagnosticLevel::Info => ds.filter_info.get(),
                DiagnosticLevel::Debug => ds.filter_debug.get(),
            })
            .count()
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
                    on_navigate=move |dest| match dest {
                        NavDestination::ActiveWorkspace => navigate_to_active_workspace(),
                        _ => set_active_nav.set(dest),
                    }
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
                                    <div class="flex items-center justify-between border-b"
                                         style="padding: var(--space-4) var(--space-5);">
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
                                                            show_header=false
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

async fn ensure_active_workspace_loaded(
    app_state: AppState,
    workspace_state: WorkspaceListState,
    run_state: ActiveRunState,
    binding_state: BindingState,
    message_state: MessageState,
    diagnostics_state: DiagnosticsState,
) -> Option<crate::models::WorkspaceId> {
    if let Some(workspace_id) = workspace_state
        .snapshot
        .get_untracked()
        .and_then(|snapshot| snapshot.workspace.map(|workspace| workspace.id))
    {
        return Some(workspace_id);
    }

    if let Some(workspace_id) = app_state
        .ui_settings
        .get_untracked()
        .last_active_workspace_id
        .filter(|workspace_id| {
            workspace_state
                .workspaces
                .get_untracked()
                .iter()
                .any(|workspace| workspace.id == *workspace_id)
        })
    {
        dispatch_command_result(
            app_state,
            workspace_state,
            run_state,
            binding_state,
            message_state,
            diagnostics_state,
            messaging::open_workspace(workspace_id).await,
        );
        return Some(workspace_id);
    }

    let next_name = format!(
        "Workspace {}",
        workspace_state.workspaces.get_untracked().len() + 1
    );
    let result = messaging::create_workspace(next_name).await;
    let workspace_id = workspace_id_from_events(&result);
    dispatch_command_result(
        app_state,
        workspace_state,
        run_state,
        binding_state,
        message_state,
        diagnostics_state,
        result,
    );
    workspace_id
}

fn workspace_id_from_events(
    result: &Result<Vec<crate::models::UiEvent>, String>,
) -> Option<crate::models::WorkspaceId> {
    result
        .as_ref()
        .ok()
        .and_then(|events| {
            events.iter().find_map(|event| match event {
                crate::models::UiEvent::WorkspaceSnapshot { snapshot } => {
                    snapshot.workspace.as_ref().map(|workspace| workspace.id)
                }
                _ => None,
            })
        })
}

fn workspace_query_workspace_id() -> Option<crate::models::WorkspaceId> {
    let window = web_sys::window()?;
    let href = window.location().href().ok()?;
    let url = web_sys::Url::new(&href).ok()?;
    let raw_id = url.search_params().get("workspace")?;
    let parsed = uuid::Uuid::parse_str(&raw_id).ok()?;
    Some(crate::models::WorkspaceId(parsed))
}
