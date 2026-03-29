//! Shared mounted screens used by both sidebar and full-tab shells.

use leptos::prelude::*;

use crate::bridge::messaging;
use crate::components::{
    binding::binding_card::BindingCard,
    composer::composer::{Composer, ComposerSubmission},
    composer::target_selector::Target,
    diagnostics::diagnostics_panel::DiagnosticsPanel,
    messages::message_log::MessageLog,
    primitives::button::{Button, ButtonSize, ButtonVariant},
    primitives::text_input::TextInput,
    routing::edge_policy_editor::EdgePolicyEditor,
    run::run_controls_bar::RunControlsBar,
    settings::settings_page::SettingsPage,
    templates::template_manager::TemplateManager,
    workspace::workspace_header::WorkspaceHeader,
    workspace::workspace_list::WorkspaceList,
};
use crate::components::provider::Provider;
use crate::components::provider::HealthState;
use crate::layout::full_tab::{SidePanelContent, SidePanelCtx};
use crate::layout::sidebar::{SidebarNav, SidebarView};
use crate::models::{
    MessageId, ProviderControlSnapshot, ProviderId, ProviderStrategy, WorkspaceId,
};
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
                        on_manage_providers=move || {
                            if let Some(side_panel_ctx) = side_panel_ctx {
                                side_panel_ctx.open(SidePanelContent::ProviderBindings);
                            } else if let Some(sidebar_nav) = sidebar_nav {
                                sidebar_nav.navigate(SidebarView::ProviderBindings);
                            }
                        }
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
pub fn ProviderBindingsScreen(on_close: impl Fn() + 'static + Copy + Send) -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();

    let active_workspace_id = Signal::derive(move || {
        workspace_state
            .snapshot
            .get()
            .and_then(|snapshot| snapshot.workspace.map(|workspace| workspace.id))
    });
    let bindings = Signal::derive(move || binding_state.bindings.get());

    view! {
        <div class="flex flex-col gap-5">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="type-title text-primary">"Provider Settings"</h2>
                    <p class="type-caption text-secondary">
                        "Manage provider-specific chats, models, reasoning, and sync."
                    </p>
                </div>
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Small
                    on_click=Box::new(move |_| on_close())
                >
                    "Close"
                </Button>
            </div>

            <div class="flex flex-col gap-4">
                {move || bindings
                    .get()
                    .into_iter()
                    .filter(|binding| matches!(binding.provider_id, ProviderId::Gpt | ProviderId::Gemini | ProviderId::Grok | ProviderId::Claude))
                    .map(|binding| {
                        let provider = Provider::from_provider_id(binding.provider_id);
                        let provider_id = binding.provider_id;
                        let workspace_id = active_workspace_id.get();
                        let snapshot = app_state
                            .provider_controls
                            .get()
                            .snapshots
                            .get(&provider_id)
                            .cloned()
                            .unwrap_or_else(|| ProviderControlSnapshot {
                                provider: provider_id,
                                capabilities: chatmux_common::ProviderControlCapabilities::default(),
                                state: chatmux_common::ProviderControlState::default(),
                                projects: Vec::new(),
                                conversations: Vec::new(),
                                models: Vec::new(),
                                reasoning_options: Vec::new(),
                                feature_flags: Vec::new(),
                            });
                        view! {
                            <div class="surface-raised rounded-md" style="border: 1px solid var(--border-default); padding: var(--space-4);">
                                <BindingCard
                                    provider=provider
                                    health=Signal::derive(move || map_health(binding.health_state))
                                    tab_info=Signal::derive({
                                        let binding = binding.clone();
                                        move || binding.tab_id.map(|id| format!("Tab #{} bound", id))
                                    })
                                    last_activity=Signal::derive({
                                        let binding = binding.clone();
                                        move || binding.last_seen_at.map(|timestamp| timestamp.to_rfc3339())
                                    })
                                    on_rebind=move || {}
                                    on_open_tab=move || {}
                                />
                                <ProviderControlPanel
                                    workspace_id=workspace_id
                                    provider_id=provider_id
                                    snapshot=snapshot
                                    app_state=app_state
                                    workspace_state=workspace_state
                                    run_state=run_state
                                    binding_state=binding_state
                                    message_state=message_state
                                    diagnostics_state=diagnostics_state
                                />
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

#[component]
fn ProviderControlPanel(
    workspace_id: Option<WorkspaceId>,
    provider_id: ProviderId,
    snapshot: ProviderControlSnapshot,
    app_state: AppState,
    workspace_state: WorkspaceListState,
    run_state: ActiveRunState,
    binding_state: BindingState,
    message_state: MessageState,
    diagnostics_state: DiagnosticsState,
) -> impl IntoView {
    let (project_title, set_project_title) = signal(String::new());
    let (conversation_title, set_conversation_title) = signal(String::new());

    let dispatch = move |result| {
        dispatch_command_result(
            app_state,
            workspace_state,
            run_state,
            binding_state,
            message_state,
            diagnostics_state,
            result,
        );
    };

    let state = snapshot.state.clone();
    let strategy = state
        .last_strategy
        .map(strategy_label)
        .unwrap_or("Unknown");

    view! {
        <div class="flex flex-col gap-3" style="margin-top: var(--space-4);">
            <div class="flex items-center gap-2 flex-wrap">
                <span class="type-caption text-secondary">
                    {format!("Strategy: {}", strategy)}
                </span>
                {state.degraded.then(|| view! {
                    <span class="type-caption" style="color: var(--status-warning-solid);">
                        "DOM fallback / degraded"
                    </span>
                })}
            </div>

            <div class="flex items-center gap-2 flex-wrap">
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Small
                    on_click=Box::new(move |_| {
                        if let Some(workspace_id) = workspace_id {
                            leptos::task::spawn_local(async move {
                                dispatch(messaging::request_provider_control_state(workspace_id, provider_id).await);
                            });
                        }
                    })
                >
                    "Refresh"
                </Button>
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Small
                    on_click=Box::new(move |_| {
                        if let Some(workspace_id) = workspace_id {
                            leptos::task::spawn_local(async move {
                                dispatch(messaging::sync_provider_conversation(workspace_id, provider_id).await);
                            });
                        }
                    })
                >
                    "Sync Conversation"
                </Button>
            </div>

            {snapshot.capabilities.supports_projects.then(|| view! {
                <div class="flex flex-col gap-2">
                    <label class="type-caption text-secondary">"Projects"</label>
                    <div class="flex gap-2">
                        <div style="flex: 1;">
                            <TextInput
                                value=project_title
                                on_input=move |val| set_project_title.set(val)
                                placeholder="Create project"
                            />
                        </div>
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Small
                            on_click=Box::new(move |_| {
                                if let Some(workspace_id) = workspace_id {
                                    let title = project_title.get_untracked();
                                    if !title.trim().is_empty() {
                                        set_project_title.set(String::new());
                                        leptos::task::spawn_local(async move {
                                            dispatch(messaging::create_provider_project(workspace_id, provider_id, title).await);
                                        });
                                    }
                                }
                            })
                        >
                            "Create"
                        </Button>
                    </div>
                    <div class="flex flex-wrap gap-2">
                        {snapshot.projects.clone().into_iter().map(move |project| {
                            let project_id = project.id.clone();
                            view! {
                                <button
                                    class="type-caption cursor-pointer"
                                    style=move || format!(
                                        "padding: var(--space-3) var(--space-5); border-radius: var(--radius-full); border: 1px solid var(--border-default); background: {};",
                                        if project.is_active { "var(--surface-sunken)" } else { "transparent" }
                                    )
                                    on:click=move |_| {
                                        if let Some(workspace_id) = workspace_id {
                                            let project_id = project_id.clone();
                                            leptos::task::spawn_local(async move {
                                                dispatch(messaging::select_provider_project(workspace_id, provider_id, project_id).await);
                                            });
                                        }
                                    }
                                >
                                    {project.title}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>
            })}

            {snapshot.capabilities.supports_conversations.then(|| view! {
                <div class="flex flex-col gap-2">
                    <label class="type-caption text-secondary">"Conversations"</label>
                    <div class="flex gap-2">
                        <div style="flex: 1;">
                            <TextInput
                                value=conversation_title
                                on_input=move |val| set_conversation_title.set(val)
                                placeholder="Create timestamped chat"
                            />
                        </div>
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Small
                            on_click=Box::new(move |_| {
                                if let Some(workspace_id) = workspace_id {
                                    let title = conversation_title.get_untracked();
                                    let title = if title.trim().is_empty() {
                                        format!("Chatmux {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))
                                    } else {
                                        title
                                    };
                                    set_conversation_title.set(String::new());
                                    let project_id = snapshot.state.project_id.clone();
                                    leptos::task::spawn_local(async move {
                                        dispatch(
                                            messaging::create_provider_conversation(workspace_id, provider_id, project_id, title).await
                                        );
                                    });
                                }
                            })
                        >
                            "New Chat"
                        </Button>
                    </div>
                    <div class="flex flex-col gap-2">
                        {snapshot.conversations.clone().into_iter().take(10).map(move |conversation| {
                            let conversation_id = conversation.id.clone();
                            view! {
                                <button
                                    class="type-caption text-left cursor-pointer"
                                    style=move || format!(
                                        "padding: var(--space-4) var(--space-5); border-radius: var(--radius-md); border: 1px solid var(--border-default); background: {};",
                                        if conversation.is_active { "var(--surface-sunken)" } else { "transparent" }
                                    )
                                    on:click=move |_| {
                                        if let Some(workspace_id) = workspace_id {
                                            let conversation_id = conversation_id.clone();
                                            leptos::task::spawn_local(async move {
                                                dispatch(messaging::select_provider_conversation(workspace_id, provider_id, conversation_id).await);
                                            });
                                        }
                                    }
                                >
                                    {conversation.title}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>
            })}

            {(!snapshot.models.is_empty()).then(|| view! {
                <div class="flex flex-col gap-2">
                    <label class="type-caption text-secondary">"Models"</label>
                    <div class="flex flex-wrap gap-2">
                        {snapshot.models.clone().into_iter().map(move |model| {
                            let model_id = model.id.clone();
                            view! {
                                <button
                                    class="type-caption cursor-pointer"
                                    style=move || format!(
                                        "padding: var(--space-3) var(--space-5); border-radius: var(--radius-full); border: 1px solid var(--border-default); background: {};",
                                        if model.is_active { "var(--surface-sunken)" } else { "transparent" }
                                    )
                                    on:click=move |_| {
                                        if let Some(workspace_id) = workspace_id {
                                            let model_id = model_id.clone();
                                            leptos::task::spawn_local(async move {
                                                dispatch(messaging::set_provider_model(workspace_id, provider_id, model_id).await);
                                            });
                                        }
                                    }
                                >
                                    {model.label}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>
            })}

            {(!snapshot.reasoning_options.is_empty()).then(|| view! {
                <div class="flex flex-col gap-2">
                    <label class="type-caption text-secondary">"Reasoning"</label>
                    <div class="flex flex-wrap gap-2">
                        {snapshot.reasoning_options.clone().into_iter().map(move |option| {
                            let option_id = option.id.clone();
                            let is_active = snapshot.state.reasoning_id.as_deref() == Some(option.id.as_str());
                            view! {
                                <button
                                    class="type-caption cursor-pointer"
                                    style=move || format!(
                                        "padding: var(--space-3) var(--space-5); border-radius: var(--radius-full); border: 1px solid var(--border-default); background: {};",
                                        if is_active { "var(--surface-sunken)" } else { "transparent" }
                                    )
                                    on:click=move |_| {
                                        if let Some(workspace_id) = workspace_id {
                                            let option_id = option_id.clone();
                                            leptos::task::spawn_local(async move {
                                                dispatch(messaging::set_provider_reasoning(workspace_id, provider_id, option_id).await);
                                            });
                                        }
                                    }
                                >
                                    {option.label}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>
            })}
        </div>
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

fn map_health(health: chatmux_common::ProviderHealth) -> HealthState {
    match health {
        chatmux_common::ProviderHealth::Disconnected => HealthState::Disconnected,
        chatmux_common::ProviderHealth::Ready => HealthState::Ready,
        chatmux_common::ProviderHealth::Composing => HealthState::Composing,
        chatmux_common::ProviderHealth::Sending => HealthState::Sending,
        chatmux_common::ProviderHealth::Generating => HealthState::Generating,
        chatmux_common::ProviderHealth::Completed => HealthState::Completed,
        chatmux_common::ProviderHealth::PermissionMissing => HealthState::PermissionMissing,
        chatmux_common::ProviderHealth::LoginRequired => HealthState::LoginRequired,
        chatmux_common::ProviderHealth::DomMismatch => HealthState::DomMismatch,
        chatmux_common::ProviderHealth::Blocked => HealthState::Blocked,
        chatmux_common::ProviderHealth::RateLimited => HealthState::RateLimited,
        chatmux_common::ProviderHealth::SendFailed => HealthState::SendFailed,
        chatmux_common::ProviderHealth::CaptureUncertain => HealthState::CaptureUncertain,
        chatmux_common::ProviderHealth::DegradedManualOnly => HealthState::DegradedManualOnly,
    }
}

fn strategy_label(strategy: ProviderStrategy) -> &'static str {
    match strategy {
        ProviderStrategy::PublicApi => "Public API",
        ProviderStrategy::Network => "Network",
        ProviderStrategy::Dom => "DOM",
        ProviderStrategy::Manual => "Manual",
    }
}
