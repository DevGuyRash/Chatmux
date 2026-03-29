//! Shared mounted screens used by both sidebar and full-tab shells.

use leptos::prelude::*;

use crate::bridge::messaging;
use crate::components::{
    composer::composer::{Composer, ComposerSubmission},
    composer::target_selector::Target,
    diagnostics::diagnostics_panel::DiagnosticsPanel,
    messages::message_log::MessageLog,
    routing::edge_policy_editor::EdgePolicyEditor,
    run::run_controls_bar::RunControlsBar,
    settings::settings_page::SettingsPage,
    templates::template_manager::TemplateManager,
    workspace::workspace_header::WorkspaceHeader,
    workspace::workspace_list::WorkspaceList,
};
use crate::components::provider::Provider;
use crate::layout::full_tab::{SidePanelContent, SidePanelCtx};
use crate::layout::sidebar::{SidebarNav, SidebarView};
use crate::models::{MessageId, ProviderId, WorkspaceId};
use crate::state::{
    app_state::AppState,
    binding_state::BindingState,
    controller::dispatch_command_result,
    diagnostics_state::DiagnosticsState,
    message_state::MessageState,
    run_state::ActiveRunState,
    workspace_state::WorkspaceListState,
};

#[component]
pub fn WorkspaceListScreen(on_select: impl Fn(WorkspaceId) + 'static + Copy + Send) -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();

    view! {
        <WorkspaceList
            workspaces=workspace_state.workspaces
            on_select=move |workspace_id| {
                on_select(workspace_id);
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
            }
            on_create=move || {
                leptos::task::spawn_local(async move {
                    let next_name = format!(
                        "Workspace {}",
                        workspace_state.workspaces.get_untracked().len() + 1
                    );
                    dispatch_command_result(
                        app_state,
                        workspace_state,
                        run_state,
                        binding_state,
                        message_state,
                        diagnostics_state,
                        messaging::create_workspace(next_name).await,
                    );
                });
            }
        />
    }
}

#[component]
pub fn ActiveWorkspaceScreen(on_back: impl Fn() + 'static + Copy + Send) -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();
    let side_panel_ctx = use_context::<SidePanelCtx>();
    let sidebar_nav = use_context::<SidebarNav>();

    let run_status = Signal::derive(move || run_state.state());
    let current_round = Signal::derive(move || {
        run_state
            .rounds
            .get()
            .iter()
            .map(|round| round.round_number)
            .max()
            .unwrap_or(0)
    });
    let max_rounds = Signal::derive(|| Some(20));
    let barrier_policy = Signal::derive(move || {
        run_state
            .run
            .get()
            .map(|run| run.barrier_policy)
            .unwrap_or(crate::models::BarrierPolicy::WaitForAll)
    });
    let targets = Signal::derive(move || {
        workspace_state
            .snapshot
            .get()
            .and_then(|snapshot| snapshot.workspace)
            .map(|workspace| {
                provider_targets(&workspace.enabled_providers)
            })
            .unwrap_or_default()
    });

    view! {
        {move || {
            let snapshot = workspace_state.snapshot.get();
            let Some(snapshot) = snapshot else {
                return view! {
                    <div class="flex items-center justify-center h-full p-6">
                        <p class="type-body text-secondary">"Select a workspace to view its conversation state."</p>
                    </div>
                }.into_any();
            };
            let Some(workspace) = snapshot.workspace.clone() else {
                return view! {
                    <div class="flex items-center justify-center h-full p-6">
                        <p class="type-body text-secondary">"This workspace has no loaded metadata."</p>
                    </div>
                }.into_any();
            };

            view! {
                <div class="flex flex-col h-full min-h-0">
                    <WorkspaceHeader
                        workspace=workspace.clone()
                        run=run_state.run.get()
                        on_back=on_back
                    />

                    <RunControlsBar
                        run_state=run_status
                        current_round=current_round
                        max_rounds=max_rounds
                        barrier_policy=barrier_policy
                        on_start=move || {
                            leptos::task::spawn_local(async move {
                                dispatch_command_result(
                                    app_state,
                                    workspace_state,
                                    run_state,
                                    binding_state,
                                    message_state,
                                    diagnostics_state,
                                    messaging::start_run(workspace.id, workspace.default_mode).await,
                                );
                            });
                        }
                        on_pause=move || {
                            if let Some(run) = run_state.run.get_untracked() {
                                leptos::task::spawn_local(async move {
                                    dispatch_command_result(
                                        app_state,
                                        workspace_state,
                                        run_state,
                                        binding_state,
                                        message_state,
                                        diagnostics_state,
                                        messaging::pause_run(run.id).await,
                                    );
                                });
                            }
                        }
                        on_resume=move || {
                            if let Some(run) = run_state.run.get_untracked() {
                                leptos::task::spawn_local(async move {
                                    dispatch_command_result(
                                        app_state,
                                        workspace_state,
                                        run_state,
                                        binding_state,
                                        message_state,
                                        diagnostics_state,
                                        messaging::resume_run(run.id).await,
                                    );
                                });
                            }
                        }
                        on_step=move || {
                            if let Some(run) = run_state.run.get_untracked() {
                                leptos::task::spawn_local(async move {
                                    dispatch_command_result(
                                        app_state,
                                        workspace_state,
                                        run_state,
                                        binding_state,
                                        message_state,
                                        diagnostics_state,
                                        messaging::step_run(run.id).await,
                                    );
                                });
                            }
                        }
                        on_stop=move || {
                            if let Some(run) = run_state.run.get_untracked() {
                                leptos::task::spawn_local(async move {
                                    dispatch_command_result(
                                        app_state,
                                        workspace_state,
                                        run_state,
                                        binding_state,
                                        message_state,
                                        diagnostics_state,
                                        messaging::stop_run(run.id).await,
                                    );
                                });
                            }
                        }
                        on_abort=move || {
                            if let Some(run) = run_state.run.get_untracked() {
                                leptos::task::spawn_local(async move {
                                    dispatch_command_result(
                                        app_state,
                                        workspace_state,
                                        run_state,
                                        binding_state,
                                        message_state,
                                        diagnostics_state,
                                        messaging::abort_run(run.id).await,
                                    );
                                });
                            }
                        }
                        on_new_run=move || {
                            leptos::task::spawn_local(async move {
                                dispatch_command_result(
                                    app_state,
                                    workspace_state,
                                    run_state,
                                    binding_state,
                                    message_state,
                                    diagnostics_state,
                                    messaging::start_run(workspace.id, workspace.default_mode).await,
                                );
                            });
                        }
                    />

                    <MessageLog
                        messages=message_state.messages
                        new_below_count=message_state.new_below_count
                        on_message_click=move |message_id: MessageId| {
                            if let Some(side_panel_ctx) = side_panel_ctx {
                                side_panel_ctx.open(SidePanelContent::MessageInspection { message_id });
                            }
                            if let Some(sidebar_nav) = sidebar_nav.clone() {
                                sidebar_nav.navigate(SidebarView::MessageInspection { message_id });
                            }
                            leptos::task::spawn_local(async move {
                                dispatch_command_result(
                                    app_state,
                                    workspace_state,
                                    run_state,
                                    binding_state,
                                    message_state,
                                    diagnostics_state,
                                    messaging::request_message_inspection(message_id).await,
                                );
                            });
                        }
                        on_scroll_to_bottom=move || {
                            message_state.set_new_below_count.set(0);
                        }
                    />

                    <Composer
                        targets=targets
                        on_send=move |submission: ComposerSubmission| {
                            let selected_targets = submission
                                .targets
                                .iter()
                                .copied()
                                .map(|provider| provider.to_provider_id())
                                .filter(|provider| workspace.enabled_providers.contains(provider))
                                .filter(|provider| *provider != ProviderId::User && *provider != ProviderId::System)
                                .collect::<Vec<_>>();
                            let approval_mode = submission.mode.approval_mode();
                            leptos::task::spawn_local(async move {
                                dispatch_command_result(
                                    app_state,
                                    workspace_state,
                                    run_state,
                                    binding_state,
                                    message_state,
                                    diagnostics_state,
                                    messaging::send_manual_message(
                                        workspace.id,
                                        selected_targets,
                                        submission.text,
                                        approval_mode,
                                    )
                                    .await,
                                );
                            });
                        }
                    />
                </div>
            }.into_any()
        }}
    }
}

#[component]
pub fn RoutingScreen() -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();

    let edges = Signal::derive(move || {
        workspace_state
            .snapshot
            .get()
            .map(|snapshot| snapshot.edge_policies)
            .unwrap_or_default()
    });

    view! {
        <EdgePolicyEditor
            edges=edges
            on_update=move |policy| {
                leptos::task::spawn_local(async move {
                    dispatch_command_result(
                        app_state,
                        workspace_state,
                        run_state,
                        binding_state,
                        message_state,
                        diagnostics_state,
                        messaging::persist_edge_policy(policy).await,
                    );
                });
            }
        />
    }
}

#[component]
pub fn TemplatesScreen() -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();

    let templates = Signal::derive(move || {
        workspace_state
            .snapshot
            .get()
            .map(|snapshot| snapshot.templates)
            .unwrap_or_default()
    });

    view! {
        <TemplateManager
            templates=templates
            on_save=move |template| {
                leptos::task::spawn_local(async move {
                    dispatch_command_result(
                        app_state,
                        workspace_state,
                        run_state,
                        binding_state,
                        message_state,
                        diagnostics_state,
                        messaging::persist_template(template).await,
                    );
                });
            }
        />
    }
}

#[component]
pub fn DiagnosticsScreen() -> impl IntoView {
    let diagnostics_state = expect_context::<DiagnosticsState>();

    view! {
        <DiagnosticsPanel events=diagnostics_state.events />
    }
}

#[component]
pub fn SettingsScreen() -> impl IntoView {
    view! { <SettingsPage /> }
}

fn provider_targets(enabled_providers: &std::collections::BTreeSet<ProviderId>) -> Vec<Target> {
    [
        (ProviderId::Gpt, Provider::Gpt),
        (ProviderId::Gemini, Provider::Gemini),
        (ProviderId::Grok, Provider::Grok),
        (ProviderId::Claude, Provider::Claude),
    ]
    .into_iter()
    .map(|(provider_id, provider)| Target {
        provider,
        bound: enabled_providers.contains(&provider_id),
    })
    .collect()
}
