//! Shared mounted screens used by both sidebar and full-tab shells.

use leptos::prelude::*;
use std::collections::BTreeSet;

use crate::bridge::messaging;
use crate::components::composer::availability::TargetAvailability;
use crate::components::provider::HealthState;
use crate::components::provider::Provider;
use crate::components::{
    binding::binding_card::BindingCard,
    composer::composer::{Composer, ComposerSubmission},
    composer::target_selector::Target,
    diagnostics::diagnostics_panel::DiagnosticsPanel,
    export::export_dialog::ExportDialog,
    messages::message_log::MessageLog,
    primitives::button::{Button, ButtonSize, ButtonVariant},
    primitives::icon::{Icon, IconKind},
    primitives::text_input::TextInput,
    routing::cursor_inspector::CursorInspector,
    routing::edge_policy_editor::EdgePolicyEditor,
    run::between_rounds_review::BetweenRoundsReview,
    run::run_config_sheet::RunConfigSheet,
    run::run_controls_bar::RunControlsBar,
    search_filter_bar::SearchFilterBar,
    settings::settings_page::SettingsPage,
    summaries::pinned_summary_manager::{PinnedSummary, PinnedSummaryManager},
    templates::template_manager::TemplateManager,
    workspace::workspace_header::WorkspaceHeader,
    workspace::workspace_list::WorkspaceList,
};
use crate::layout::full_tab::{SidePanelContent, SidePanelCtx};
use crate::layout::responsive::LayoutMode;
use crate::layout::sidebar::{SidebarNav, SidebarView};
use crate::models::{
    BarrierPolicy, MessageId, NextRoundPackage, OrchestrationMode, ProviderControlSnapshot,
    ProviderId, ProviderStrategy, RunConfiguration, UiEvent, WorkspaceId,
};
use crate::state::{
    app_state::AppState, binding_state::BindingState, controller::dispatch_command_result,
    diagnostics_state::DiagnosticsState, message_state::MessageState, run_state::ActiveRunState,
    search_state::SearchState, workspace_state::WorkspaceListState,
};
use crate::time::{format_local_datetime, format_local_title_timestamp};

#[component]
pub fn WorkspaceListScreen(
    on_select: impl Fn(WorkspaceId) + 'static + Copy + Send,
) -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();
    let layout_mode = expect_context::<LayoutMode>();

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
                    let result = messaging::create_workspace(next_name).await;
                    let workspace_id = workspace_id_from_result(&result);
                    dispatch_command_result(
                        app_state,
                        workspace_state,
                        run_state,
                        binding_state,
                        message_state,
                        diagnostics_state,
                        result,
                    );
                    if let Some(workspace_id) = workspace_id {
                        on_select(workspace_id);
                        if layout_mode == LayoutMode::Sidebar {
                            let url = extension_workspace_url(workspace_id);
                            let _ = messaging::open_tab(&url).await;
                        }
                    }
                });
            }
            on_delete=move |workspace_id| {
                leptos::task::spawn_local(async move {
                    // Deleting a workspace removes its history as part of the
                    // same operation; there is no partial variant to branch on.
                    let result = messaging::delete_workspace(workspace_id).await;
                    dispatch_command_result(
                        app_state,
                        workspace_state,
                        run_state,
                        binding_state,
                        message_state,
                        diagnostics_state,
                        result,
                    );
                });
            }
            on_rename=move |workspace_id, name| {
                leptos::task::spawn_local(async move {
                    dispatch_command_result(
                        app_state, workspace_state, run_state, binding_state,
                        message_state, diagnostics_state,
                        messaging::rename_workspace(workspace_id, name).await,
                    );
                });
            }
            on_duplicate=move |workspace_id| {
                leptos::task::spawn_local(async move {
                    let result = messaging::duplicate_workspace(workspace_id).await;
                    dispatch_command_result(
                        app_state, workspace_state, run_state, binding_state,
                        message_state, diagnostics_state, result,
                    );
                });
            }
            on_archive=move |workspace_id, archived| {
                leptos::task::spawn_local(async move {
                    dispatch_command_result(
                        app_state, workspace_state, run_state, binding_state,
                        message_state, diagnostics_state,
                        messaging::set_workspace_archived(workspace_id, archived).await,
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
    let search_state = expect_context::<SearchState>();
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
    let max_rounds = Signal::derive(move || {
        run_state
            .run
            .get()
            .and_then(|run| run.timing_policy.max_rounds)
    });
    let barrier_policy = Signal::derive(move || {
        run_state
            .run
            .get()
            .map(|run| run.barrier_policy)
            .unwrap_or(crate::models::BarrierPolicy::WaitForAll)
    });
    let (composer_draft, set_composer_draft) = signal(String::new());
    let (composer_submitting, set_composer_submitting) = signal(false);
    let (context_selection_mode, set_context_selection_mode) = signal(false);
    let (selected_context_ids, set_selected_context_ids) = signal(BTreeSet::<MessageId>::new());
    let (export_open, set_export_open) = signal(false);
    let (run_config_open, set_run_config_open) = signal(false);
    let (review_open, set_review_open) = signal(false);
    let (review_round, set_review_round) = signal(0u32);
    let (review_packages, set_review_packages) = signal(Vec::<NextRoundPackage>::new());
    let targets = Signal::derive(move || {
        workspace_state
            .snapshot
            .get()
            .and_then(|snapshot| snapshot.workspace)
            .map(|workspace| {
                provider_targets(
                    &workspace.enabled_providers,
                    &binding_state.bindings.get(),
                    &app_state.provider_health.get(),
                )
            })
            .unwrap_or_default()
    });
    let filtered_messages = Signal::derive(move || {
        let query = search_state.query.get().to_lowercase();
        let provider = search_state.provider_filter.get();
        let role = search_state.role_filter.get();
        let round_min = search_state.round_min.get();
        let round_max = search_state.round_max.get();
        let tag_query = search_state.tag_query.get().to_lowercase();
        message_state
            .messages
            .get()
            .into_iter()
            .filter(|message| {
                (query.is_empty() || message.body_text.to_lowercase().contains(&query))
                    && provider.is_none_or(|provider| message.participant_id == provider)
                    && role.is_none_or(|role| message.role == role)
                    && round_min
                        .is_none_or(|minimum| message.round.is_some_and(|round| round >= minimum))
                    && round_max
                        .is_none_or(|maximum| message.round.is_some_and(|round| round <= maximum))
                    && (tag_query.is_empty()
                        || message
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&tag_query)))
            })
            .collect::<Vec<_>>()
    });
    Effect::new(move |_| {
        let count = filtered_messages.get().len() as u32;
        search_state.set_result_count.set(count);
        search_state.set_current_result.update(|current| {
            if count == 0 {
                *current = 0;
            } else if *current == 0 || *current > count {
                *current = 1;
            }
        });
        search_state.set_current_result.update(|current| {
            *current = if count == 0 {
                0
            } else {
                (*current).clamp(1, count)
            };
        });
    });
    let active_workspace = Memo::new(move |_| {
        workspace_state
            .snapshot
            .get()
            .and_then(|snapshot| snapshot.workspace)
    });

    view! {
        {move || {
            let Some(workspace) = active_workspace.get() else {
                let has_error = app_state.last_error.get().is_some();
                let error_msg = app_state.last_error.get().unwrap_or_default();

                return view! {
                    <div class="flex flex-col h-full">
                        // Back button so user isn't trapped
                        <div class="flex items-center gap-3 border-b"
                             style="padding: var(--space-5) var(--space-6); \
                                    background: var(--surface-raised);">
                            <Button
                                variant=ButtonVariant::Icon
                                size=ButtonSize::Small
                                aria_label="Back to workspaces".to_string()
                                on_click=Box::new(move |_| on_back())
                            >
                                <Icon kind=IconKind::ArrowLeft size=18 />
                            </Button>
                            <span class="type-title text-primary">
                                {if has_error { "Connection error" } else { "Loading..." }}
                            </span>
                        </div>

                        {if has_error {
                            // Error state: show message + retry
                            view! {
                                <div class="flex-1 flex flex-col items-center justify-center gap-4 p-6">
                                    <Icon kind=IconKind::ExclamationCircle size=40 color="var(--status-error-text)".to_string() />
                                    <p class="type-body text-secondary text-center" style="max-width: 280px;">
                                        "Could not connect to the background service. The extension may need to be reloaded."
                                    </p>
                                    <p class="type-caption text-tertiary text-center" style="max-width: 280px;">
                                        {error_msg}
                                    </p>
                                    <Button
                                        variant=ButtonVariant::Secondary
                                        on_click=Box::new(move |_| on_back())
                                    >
                                        "Back to Workspaces"
                                    </Button>
                                </div>
                            }.into_any()
                        } else {
                            // Loading state: skeleton shimmer
                            view! {
                                <div class="flex-1 flex flex-col gap-4 p-6">
                                    <div class="skeleton rounded-md" style="height: 48px;" />
                                    <div class="skeleton rounded-md" style="height: 120px;" />
                                    <div class="skeleton rounded-md" style="height: 80px;" />
                                    <div class="skeleton rounded-md" style="height: 80px;" />
                                </div>
                            }.into_any()
                        }}
                    </div>
                }.into_any();
            };
            let active_workspace_id = workspace.id;
            let composer_enabled_providers = workspace.enabled_providers.clone();

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
                        on_export=move || set_export_open.set(true)
                    />

                    <RunControlsBar
                        run_state=run_status
                        current_round=current_round
                        max_rounds=max_rounds
                        barrier_policy=barrier_policy
                        on_start=move || {
                            set_run_config_open.set(true);
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
                        on_edit_packages=move || {
                            if let Some(run) = run_state.run.get_untracked() {
                                leptos::task::spawn_local(async move {
                                    let result = messaging::preview_next_round(run.id).await;
                                    if let Ok(events) = &result
                                        && let Some((round_number, packages)) = events.iter().find_map(|event| match event {
                                            UiEvent::NextRoundPreview { round_number, packages, .. } => {
                                                Some((*round_number, packages.clone()))
                                            }
                                            _ => None,
                                        })
                                    {
                                        set_review_round.set(round_number);
                                        set_review_packages.set(packages);
                                        set_review_open.set(true);
                                    }
                                    dispatch_command_result(
                                        app_state,
                                        workspace_state,
                                        run_state,
                                        binding_state,
                                        message_state,
                                        diagnostics_state,
                                        result,
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
                            set_run_config_open.set(true);
                        }
                    />

                    <div class="flex items-center justify-end border-b px-4 py-2">
                        <Show when=move || !search_state.is_active.get()>
                            <Button
                                variant=ButtonVariant::Ghost
                                size=ButtonSize::Small
                                aria_label="Search and filter messages".to_owned()
                                on_click=Box::new(move |_| search_state.set_is_active.set(true))
                            >
                                <Icon kind=IconKind::Search size=14 />
                                "Search"
                            </Button>
                        </Show>
                    </div>
                    <SearchFilterBar
                        query=search_state.query
                        set_query=search_state.set_query
                        is_active=search_state.is_active
                        show_filters=search_state.show_filters
                        set_show_filters=search_state.set_show_filters
                        result_count=search_state.result_count
                        current_result=search_state.current_result
                        provider_filter=search_state.provider_filter
                        set_provider_filter=search_state.set_provider_filter
                        role_filter=search_state.role_filter
                        set_role_filter=search_state.set_role_filter
                        round_min=search_state.round_min
                        set_round_min=search_state.set_round_min
                        round_max=search_state.round_max
                        set_round_max=search_state.set_round_max
                        tag_query=search_state.tag_query
                        set_tag_query=search_state.set_tag_query
                        on_next=move || {
                            let count = search_state.result_count.get_untracked();
                            if count > 0 {
                                search_state.set_current_result.update(|current| {
                                    *current = if *current >= count { 1 } else { current.saturating_add(1) };
                                });
                            }
                        }
                        on_prev=move || {
                            let count = search_state.result_count.get_untracked();
                            if count > 0 {
                                search_state.set_current_result.update(|current| {
                                    *current = if *current <= 1 { count } else { current.saturating_sub(1) };
                                });
                            }
                        }
                        on_close=move || {
                            search_state.set_is_active.set(false);
                            search_state.set_query.set(String::new());
                        }
                    />
                    <MessageLog
                        messages=filtered_messages
                        new_below_count=message_state.new_below_count
                        branch_parent_id=message_state.branch_parent_id
                        context_selection_mode=context_selection_mode
                        selected_context_ids=selected_context_ids
                        on_message_click=move |message_id: MessageId| {
                            if let Some(side_panel_ctx) = side_panel_ctx {
                                side_panel_ctx.open(SidePanelContent::MessageInspection { message_id });
                            }
                            if let Some(sidebar_nav) = sidebar_nav {
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
                        on_branch_from_message=move |message_id: MessageId| {
                            message_state.set_branch_parent_id.set(Some(message_id));
                        }
                        on_toggle_context=move |message_id: MessageId| {
                            set_selected_context_ids.update(|selected| {
                                if !selected.insert(message_id) {
                                    selected.remove(&message_id);
                                }
                            });
                        }
                        on_context_done=move || set_context_selection_mode.set(false)
                        on_context_clear=move || set_selected_context_ids.set(BTreeSet::new())
                        on_scroll_to_bottom=move || {
                            message_state.set_new_below_count.set(0);
                        }
                    />

                    <Composer
                        workspace_id=active_workspace_id
                        targets=targets
                        draft=composer_draft
                        set_draft=set_composer_draft
                        submitting=composer_submitting
                        kill_switch_active=app_state.kill_switch_active
                        context_selection_mode=context_selection_mode
                        set_context_selection_mode=set_context_selection_mode
                        selected_context_ids=selected_context_ids
                        set_selected_context_ids=set_selected_context_ids
                        branch_parent_id=message_state.branch_parent_id
                        on_clear_branch_parent=move || {
                            message_state.set_branch_parent_id.set(None);
                        }
                        on_send=move |submission: ComposerSubmission| {
                            let selected_targets = submission
                                .targets
                                .iter()
                                .copied()
                                .map(|provider| provider.to_provider_id())
                                .filter(|provider| composer_enabled_providers.contains(provider))
                                .filter(|provider| *provider != ProviderId::User && *provider != ProviderId::System)
                                .collect::<Vec<_>>();
                            let approval_mode = submission.mode.approval_mode();
                            let success_message = match submission.mode {
                                crate::components::composer::mode_selector::ComposerMode::Send => {
                                    "Message queued for delivery."
                                }
                                crate::components::composer::mode_selector::ComposerMode::DraftOnly => {
                                    "Draft saved."
                                }
                                crate::components::composer::mode_selector::ComposerMode::CopyOnly => {
                                    "Outbound text prepared."
                                }
                            };
                            let failure_message = match submission.mode {
                                crate::components::composer::mode_selector::ComposerMode::Send => {
                                    "Couldn't queue the message:"
                                }
                                crate::components::composer::mode_selector::ComposerMode::DraftOnly => {
                                    "Couldn't save the draft:"
                                }
                                crate::components::composer::mode_selector::ComposerMode::CopyOnly => {
                                    "Couldn't prepare the outbound text:"
                                }
                            };
                            set_composer_submitting.set(true);
                            leptos::task::spawn_local(async move {
                                let result = messaging::send_manual_message(
                                    active_workspace_id,
                                    selected_targets,
                                    submission.text,
                                    approval_mode,
                                    submission.selected_message_ids,
                                    submission.pinned_note,
                                    submission.target_notes,
                                    submission.include_target_prior_turns,
                                    submission.payload_overrides,
                                    submission.parent_message_id,
                                )
                                .await;
                                let copy_payload = (submission.mode
                                    == crate::components::composer::mode_selector::ComposerMode::CopyOnly)
                                    .then(|| single_rendered_payload(&result))
                                    .flatten();
                                let confirmed = crate::state::controller::dispatch_user_command_result(
                                    app_state,
                                    workspace_state,
                                    run_state,
                                    binding_state,
                                    message_state,
                                    diagnostics_state,
                                    success_message,
                                    failure_message,
                                    result,
                                );
                                let completed = if confirmed
                                    && submission.mode
                                        == crate::components::composer::mode_selector::ComposerMode::CopyOnly
                                {
                                    if let Some(payload) = copy_payload {
                                        let copied = crate::bridge::clipboard::write_clipboard(&payload).await;
                                        crate::state::controller::publish_user_outcome(
                                            app_state,
                                            if copied {
                                                crate::state::command_state::CommandOutcomeKind::Success
                                            } else {
                                                crate::state::command_state::CommandOutcomeKind::Error
                                            },
                                            if copied {
                                                "Exact rendered payload copied to the clipboard."
                                            } else {
                                                "The rendered payload was prepared, but clipboard access failed."
                                            },
                                        );
                                        copied
                                    } else {
                                        crate::state::controller::publish_user_outcome(
                                            app_state,
                                            crate::state::command_state::CommandOutcomeKind::Error,
                                            "No single rendered payload was returned to copy.",
                                        );
                                        false
                                    }
                                } else {
                                    confirmed
                                };
                                if completed {
                                    set_composer_draft.set(String::new());
                                    message_state.set_branch_parent_id.set(None);
                                    set_selected_context_ids.set(BTreeSet::new());
                                    set_context_selection_mode.set(false);
                                }
                                set_composer_submitting.set(false);
                            });
                        }
                    />
                </div>
            }.into_any()
        }}
        // Dialog components live outside the reactive workspace body. Run/message updates may
        // rebuild that body, but an in-progress form must retain its local selection and edits.
        {move || active_workspace.get().map(|workspace| {
            let active_workspace_id = workspace.id;
            let initial_run_mode = workspace.default_mode;
            let run_participants = workspace.enabled_providers.clone();
            view! {
                <ExportDialog
                    open=export_open
                    workspace_id=active_workspace_id
                    on_close=move || set_export_open.set(false)
                />
                <BetweenRoundsReview
                    open=review_open
                    round_number=review_round
                    packages=review_packages
                    on_close=move || set_review_open.set(false)
                    on_resume=move |payload_overrides, skipped_targets, injected_user_message| {
                        let Some(run) = run_state.run.get_untracked() else {
                            return;
                        };
                        set_review_open.set(false);
                        leptos::task::spawn_local(async move {
                            dispatch_command_result(
                                app_state,
                                workspace_state,
                                run_state,
                                binding_state,
                                message_state,
                                diagnostics_state,
                                messaging::resume_run_with_overrides(
                                    run.id,
                                    payload_overrides,
                                    skipped_targets,
                                    injected_user_message,
                                )
                                .await,
                            );
                        });
                    }
                />
                <RunConfigSheet
                    open=run_config_open
                    initial_mode=initial_run_mode
                    available_participants=run_participants
                    on_cancel=move || set_run_config_open.set(false)
                    on_start=move |configuration| {
                        set_run_config_open.set(false);
                        leptos::task::spawn_local(async move {
                            dispatch_command_result(
                                app_state,
                                workspace_state,
                                run_state,
                                binding_state,
                                message_state,
                                diagnostics_state,
                                messaging::start_configured_run(active_workspace_id, configuration).await,
                            );
                        });
                    }
                />
            }
        })}
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
    let cursors = Signal::derive(move || {
        workspace_state
            .snapshot
            .get()
            .map(|snapshot| snapshot.delivery_cursors)
            .unwrap_or_default()
    });
    let summaries = Signal::derive(move || {
        message_state
            .messages
            .get()
            .into_iter()
            .filter(|message| message.tags.iter().any(|tag| tag == "pinned-summary"))
            .map(|message| {
                let name = message
                    .tags
                    .iter()
                    .find_map(|tag| tag.strip_prefix("summary-name:"))
                    .unwrap_or("Pinned summary")
                    .to_owned();
                (message.id, name)
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class="flex flex-col h-full min-h-0">
            <AdvancedRoutingTools edges=edges />
            <div class="flex-1 min-h-0">
                <EdgePolicyEditor
                    edges=edges
                    summaries=summaries
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
            </div>
            <div class="border-t p-5" style="max-height: 38%; overflow-y: auto;">
                <CursorInspector
                    cursors=cursors
                    on_reset=move |cursor_id| leptos::task::spawn_local(async move {
                        dispatch_command_result(
                            app_state, workspace_state, run_state, binding_state,
                            message_state, diagnostics_state,
                            messaging::reset_delivery_cursor(cursor_id).await,
                        );
                    })
                    on_toggle_freeze=move |cursor_id| {
                        let frozen = cursors.get_untracked().into_iter()
                            .find(|cursor| cursor.id == cursor_id)
                            .map(|cursor| !cursor.frozen)
                            .unwrap_or(true);
                        leptos::task::spawn_local(async move {
                            dispatch_command_result(
                                app_state, workspace_state, run_state, binding_state,
                                message_state, diagnostics_state,
                                messaging::set_delivery_cursor_frozen(cursor_id, frozen).await,
                            );
                        });
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn AdvancedRoutingTools(edges: Signal<Vec<crate::models::EdgePolicy>>) -> impl IntoView {
    use crate::bridge::storage::{OrchestrationRecipe, RecipePhase, SavedRoutePreset};

    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();
    let (preset_name, set_preset_name) = signal(String::new());
    let (selected_preset, set_selected_preset) = signal(String::new());
    let (recipe_name, set_recipe_name) = signal(String::new());
    let (selected_recipe, set_selected_recipe) = signal(String::new());
    let (recipe_phase, set_recipe_phase) = signal(0usize);

    view! {
        <section class="surface-raised border-b p-4 flex flex-col gap-3" aria-label="Routing presets and recipes">
            <div class="flex items-center gap-3 flex-wrap">
                <span class="type-caption-strong text-secondary">"Route presets"</span>
                <select
                    class="type-caption text-primary surface-sunken border rounded-md p-2"
                    aria-label="Saved route preset"
                    prop:value=move || selected_preset.get()
                    on:change=move |event| set_selected_preset.set(event_target_value(&event))
                >
                    <option value="">"Choose preset"</option>
                    {move || {
                        let Some(workspace_id) = app_state.active_workspace_id.get() else { return Vec::new().into_iter().collect_view(); };
                        app_state.ui_settings.get().saved_route_presets
                            .get(&workspace_id).cloned().unwrap_or_default().into_iter()
                            .map(|preset| view! { <option value=preset.id.to_string()>{preset.name}</option> })
                            .collect_view()
                    }}
                </select>
                <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| {
                    let selected = selected_preset.get_untracked();
                    let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else { return; };
                    let preset = app_state.ui_settings.get_untracked().saved_route_presets
                        .get(&workspace_id).and_then(|presets| presets.iter().find(|preset| preset.id.to_string() == selected)).cloned();
                    if let Some(preset) = preset {
                        leptos::task::spawn_local(async move {
                            for policy in preset.policies {
                                dispatch_command_result(
                                    app_state, workspace_state, run_state, binding_state,
                                    message_state, diagnostics_state,
                                    messaging::persist_edge_policy(policy).await,
                                );
                            }
                        });
                    }
                })>"Apply"</Button>
                <input
                    class="type-caption text-primary surface-sunken border rounded-md p-2"
                    aria-label="Route preset name"
                    placeholder="Preset name"
                    prop:value=move || preset_name.get()
                    on:input=move |event| set_preset_name.set(event_target_value(&event))
                />
                <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| {
                    let name = preset_name.get_untracked().trim().to_owned();
                    let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else { return; };
                    if name.is_empty() || edges.get_untracked().is_empty() { return; }
                    app_state.set_ui_settings.update(|settings| {
                        settings.saved_route_presets.entry(workspace_id).or_default().push(SavedRoutePreset {
                            id: uuid::Uuid::new_v4(), name, policies: edges.get_untracked(),
                        });
                    });
                    set_preset_name.set(String::new());
                })>"Save current graph"</Button>
                <Button variant=ButtonVariant::Ghost on_click=Box::new(move |_| {
                    let selected = selected_preset.get_untracked();
                    let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else { return; };
                    app_state.set_ui_settings.update(|settings| {
                        if let Some(presets) = settings.saved_route_presets.get_mut(&workspace_id) {
                            presets.retain(|preset| preset.id.to_string() != selected);
                        }
                    });
                    set_selected_preset.set(String::new());
                })>"Delete"</Button>
            </div>

            <div class="flex items-center gap-3 flex-wrap">
                <span class="type-caption-strong text-secondary">"Multi-phase recipes"</span>
                <select
                    class="type-caption text-primary surface-sunken border rounded-md p-2"
                    aria-label="Orchestration recipe"
                    prop:value=move || selected_recipe.get()
                    on:change=move |event| {
                        set_selected_recipe.set(event_target_value(&event));
                        set_recipe_phase.set(0);
                    }
                >
                    <option value="">"Choose recipe"</option>
                    {move || {
                        let Some(workspace_id) = app_state.active_workspace_id.get() else { return Vec::new().into_iter().collect_view(); };
                        app_state.ui_settings.get().orchestration_recipes
                            .get(&workspace_id).cloned().unwrap_or_default().into_iter()
                            .map(|recipe| view! { <option value=recipe.id.to_string()>{recipe.name}</option> })
                            .collect_view()
                    }}
                </select>
                <input
                    class="type-caption text-primary surface-sunken border rounded-md p-2"
                    aria-label="Recipe name"
                    placeholder="Recipe name"
                    prop:value=move || recipe_name.get()
                    on:input=move |event| set_recipe_name.set(event_target_value(&event))
                />
                <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| {
                    let name = recipe_name.get_untracked().trim().to_owned();
                    let Some(workspace) = workspace_state.snapshot.get_untracked().and_then(|snapshot| snapshot.workspace) else { return; };
                    if name.is_empty() || workspace.enabled_providers.is_empty() { return; }
                    let mut one_round = app_state.ui_settings.get_untracked().timing;
                    one_round.max_rounds = Some(1);
                    let moderator = workspace.enabled_providers.iter().next().copied();
                    let phase = |name: &str, mode, moderator| RecipePhase {
                        name: name.to_owned(),
                        configuration: RunConfiguration {
                            mode,
                            participants: workspace.enabled_providers.clone(),
                            moderator,
                            relay_order: workspace.enabled_providers.iter().copied().collect(),
                            barrier_policy: BarrierPolicy::WaitForAll,
                            timing_policy: one_round.clone(),
                            stop_policy: crate::models::StopPolicy::default(),
                            require_review_between_rounds: true,
                        },
                    };
                    let recipe = OrchestrationRecipe {
                        id: uuid::Uuid::new_v4(),
                        name,
                        phases: vec![
                            phase("Independent drafts", OrchestrationMode::Broadcast, None),
                            phase("Conflict map", OrchestrationMode::Roundtable, None),
                            phase("Targeted synthesis", OrchestrationMode::ModeratorJury, moderator),
                            phase("Final verification", OrchestrationMode::Roundtable, None),
                        ],
                    };
                    app_state.set_ui_settings.update(|settings| {
                        settings.orchestration_recipes.entry(workspace.id).or_default().push(recipe);
                    });
                    set_recipe_name.set(String::new());
                })>"Save 4-phase recipe"</Button>
                <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                    let selected = selected_recipe.get_untracked();
                    let phase_index = recipe_phase.get_untracked();
                    let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else { return; };
                    let phase = app_state.ui_settings.get_untracked().orchestration_recipes
                        .get(&workspace_id)
                        .and_then(|recipes| recipes.iter().find(|recipe| recipe.id.to_string() == selected))
                        .and_then(|recipe| recipe.phases.get(phase_index))
                        .cloned();
                    if let Some(phase) = phase {
                        leptos::task::spawn_local(async move {
                            dispatch_command_result(
                                app_state, workspace_state, run_state, binding_state,
                                message_state, diagnostics_state,
                                messaging::start_configured_run(workspace_id, phase.configuration).await,
                            );
                        });
                    }
                })>
                    {move || {
                        let selected = selected_recipe.get();
                        let phase_index = recipe_phase.get();
                        let Some(workspace_id) = app_state.active_workspace_id.get() else { return "Run phase".to_owned(); };
                        app_state.ui_settings.get().orchestration_recipes.get(&workspace_id)
                            .and_then(|recipes| recipes.iter().find(|recipe| recipe.id.to_string() == selected))
                            .and_then(|recipe| recipe.phases.get(phase_index))
                            .map(|phase| format!("Run phase {}: {}", phase_index + 1, phase.name))
                            .unwrap_or_else(|| "Run phase".to_owned())
                    }}
                </Button>
                <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| {
                    let selected = selected_recipe.get_untracked();
                    let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else { return; };
                    let count = app_state.ui_settings.get_untracked().orchestration_recipes
                        .get(&workspace_id)
                        .and_then(|recipes| recipes.iter().find(|recipe| recipe.id.to_string() == selected))
                        .map(|recipe| recipe.phases.len()).unwrap_or(0);
                    if count > 0 { set_recipe_phase.update(|phase| *phase = (*phase + 1).min(count - 1)); }
                })>"Next phase"</Button>
            </div>
        </section>
    }
}

#[component]
pub fn SummariesScreen() -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();

    let summaries = Signal::derive(move || {
        let policies = workspace_state
            .snapshot
            .get()
            .map(|snapshot| snapshot.edge_policies)
            .unwrap_or_default();
        message_state
            .messages
            .get()
            .into_iter()
            .filter(|message| message.tags.iter().any(|tag| tag == "pinned-summary"))
            .map(|message| {
                let in_use = policies.iter().any(|policy| {
                    matches!(
                        policy.catch_up_policy,
                        crate::models::CatchUpPolicy::PinnedSummary {
                            summary_message_id: Some(id)
                        } if id == message.id
                    ) || matches!(
                        policy.truncation_policy,
                        crate::models::TruncationPolicy::SwapForSummary {
                            summary_message_id: Some(id), ..
                        } if id == message.id
                    )
                });
                let name = message
                    .tags
                    .iter()
                    .find_map(|tag| tag.strip_prefix("summary-name:"))
                    .unwrap_or("Pinned summary")
                    .to_owned();
                PinnedSummary {
                    id: message.id,
                    name,
                    body: message.body_text,
                    created_at: format_local_datetime(message.timestamp),
                    in_use,
                }
            })
            .collect::<Vec<_>>()
    });

    view! {
        <PinnedSummaryManager
            summaries=summaries
            on_save=move |summary| {
                let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else {
                    return;
                };
                leptos::task::spawn_local(async move {
                    dispatch_command_result(
                        app_state,
                        workspace_state,
                        run_state,
                        binding_state,
                        message_state,
                        diagnostics_state,
                        messaging::persist_pinned_summary(
                            workspace_id,
                            Some(summary.id),
                            summary.name,
                            summary.body,
                        )
                        .await,
                    );
                });
            }
            on_delete=move |summary_message_id| {
                let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else {
                    return;
                };
                leptos::task::spawn_local(async move {
                    dispatch_command_result(
                        app_state,
                        workspace_state,
                        run_state,
                        binding_state,
                        message_state,
                        diagnostics_state,
                        messaging::delete_pinned_summary(workspace_id, summary_message_id).await,
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
    let workspace_id = Signal::derive(move || {
        workspace_state
            .snapshot
            .get()
            .and_then(|snapshot| snapshot.workspace.map(|workspace| workspace.id))
    });

    view! {
        <TemplateManager
            workspace_id=workspace_id
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
            on_delete=move |template_id| {
                leptos::task::spawn_local(async move {
                    dispatch_command_result(
                        app_state,
                        workspace_state,
                        run_state,
                        binding_state,
                        message_state,
                        diagnostics_state,
                        messaging::delete_template(template_id).await,
                    );
                });
            }
        />
    }
}

#[component]
pub fn DiagnosticsScreen() -> impl IntoView {
    let _diagnostics_state = expect_context::<DiagnosticsState>();

    view! { <DiagnosticsPanel /> }
}

#[component]
pub fn ProviderBindingsScreen(
    on_close: impl Fn() + 'static + Copy + Send,
    #[prop(default = true)] show_header: bool,
) -> impl IntoView {
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
            .or_else(|| app_state.active_workspace_id.get())
    });
    let (permission_grants, set_permission_grants) =
        signal(std::collections::BTreeMap::<ProviderId, bool>::new());
    let (permission_check_started, set_permission_check_started) = signal(false);

    Effect::new(move |_| {
        if permission_check_started.get() {
            return;
        }
        set_permission_check_started.set(true);
        for provider_id in provider_ids() {
            leptos::task::spawn_local(async move {
                let granted = crate::bridge::permissions::check_host_permissions(
                    provider_permission_origins(provider_id),
                )
                .await;
                set_permission_grants.update(|grants| {
                    grants.insert(provider_id, granted);
                });
            });
        }
    });

    view! {
        <div class="flex flex-col gap-5">
            {show_header.then(|| view! {
                <div class="flex items-center justify-between">
                    <div>
                        <h2 class="type-title text-primary">"Provider Settings"</h2>
                        <p class="type-caption text-secondary">
                            "Grant access, detect a tab, then bind the exact provider conversation Chatmux may use."
                        </p>
                    </div>
                    <div class="flex items-center gap-2">
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Small
                            on_click=Box::new(move |_| {
                                let Some(workspace_id) = active_workspace_id.get_untracked() else {
                                    crate::state::controller::publish_user_outcome(
                                        app_state,
                                        crate::state::command_state::CommandOutcomeKind::Error,
                                        "Open a workspace before detecting provider tabs.",
                                    );
                                    return;
                                };
                                leptos::task::spawn_local(async move {
                                    let mut all_ok = true;
                                    for provider_id in provider_ids() {
                                        all_ok &= dispatch_command_result(
                                            app_state,
                                            workspace_state,
                                            run_state,
                                            binding_state,
                                            message_state,
                                            diagnostics_state,
                                            messaging::request_provider_tab_candidates(
                                                workspace_id,
                                                provider_id,
                                            )
                                            .await,
                                        );
                                    }
                                    if all_ok {
                                        crate::state::controller::publish_user_outcome(
                                            app_state,
                                            crate::state::command_state::CommandOutcomeKind::Success,
                                            "Provider tab detection finished.",
                                        );
                                    }
                                });
                            })
                        >
                            "Detect tabs"
                        </Button>
                        <Button
                            variant=ButtonVariant::Ghost
                            size=ButtonSize::Small
                            on_click=Box::new(move |_| on_close())
                        >
                            "Close"
                        </Button>
                    </div>
                </div>
            })}

            <div class="flex flex-col gap-4">
                {move || provider_ids()
                    .into_iter()
                    .map(|provider_id| {
                        let provider = Provider::from_provider_id(provider_id);
                        let binding = binding_state
                            .bindings
                            .get()
                            .into_iter()
                            .find(|binding| binding.provider_id == provider_id);
                        let binding_for_health = binding.clone();
                        let binding_for_bound = binding.clone();
                        let binding_for_tab = binding.clone();
                        let binding_for_activity = binding.clone();
                        let binding_for_panel = binding.clone();
                        let workspace_id = active_workspace_id.get();
                        let permission_missing = Signal::derive(move || {
                            !permission_grants
                                .get()
                                .get(&provider_id)
                                .copied()
                                .unwrap_or(false)
                        });
                        let health = Signal::derive(move || {
                            if permission_missing.get() {
                                return HealthState::PermissionMissing;
                            }
                            let provider_health = app_state
                                .provider_health
                                .get()
                                .get(&provider_id)
                                .map(|state| state.health)
                                .or_else(|| {
                                    binding_for_health
                                        .as_ref()
                                        .map(|binding| binding.health_state)
                                })
                                .unwrap_or(crate::models::ProviderHealth::Disconnected);
                            map_health(provider_health)
                        });
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
                                    health=health
                                    bound=Signal::derive(move || {
                                        binding_for_bound
                                            .as_ref()
                                            .is_some_and(|binding| binding.tab_id.is_some())
                                    })
                                    permission_missing=permission_missing
                                    tab_info=Signal::derive({
                                        move || binding_for_tab.as_ref().and_then(|binding| {
                                            binding.tab_id.map(|id| {
                                                let title = binding
                                                    .tab_title
                                                    .clone()
                                                    .unwrap_or_else(|| "Bound browser tab".to_owned());
                                                let pin_suffix = if binding.pinned { " · pinned" } else { "" };
                                                format!("{title} · Tab #{id}{pin_suffix}")
                                            })
                                        })
                                    })
                                    last_activity=Signal::derive({
                                        move || binding_for_activity
                                            .as_ref()
                                            .and_then(|binding| binding.last_seen_at.map(format_local_datetime))
                                    })
                                    on_detect=move || {
                                        if let Some(workspace_id) = workspace_id {
                                            leptos::task::spawn_local(async move {
                                                crate::state::controller::dispatch_user_command_result(
                                                    app_state,
                                                    workspace_state,
                                                    run_state,
                                                    binding_state,
                                                    message_state,
                                                    diagnostics_state,
                                                    "Provider tabs detected.",
                                                    "Couldn't detect provider tabs:",
                                                    messaging::request_provider_tab_candidates(workspace_id, provider_id).await,
                                                );
                                            });
                                        }
                                    }
                                    on_grant_permission=move || {
                                        leptos::task::spawn_local(async move {
                                            let granted = crate::bridge::permissions::request_host_permissions(
                                                provider_permission_origins(provider_id),
                                            )
                                            .await;
                                            set_permission_grants.update(|grants| {
                                                grants.insert(provider_id, granted);
                                            });
                                            if !granted {
                                                crate::state::controller::publish_user_outcome(
                                                    app_state,
                                                    crate::state::command_state::CommandOutcomeKind::Error,
                                                    format!("Access to {} was not granted.", provider.label()),
                                                );
                                                return;
                                            }
                                            if let Some(workspace_id) = workspace_id {
                                                crate::state::controller::dispatch_user_command_result(
                                                    app_state,
                                                    workspace_state,
                                                    run_state,
                                                    binding_state,
                                                    message_state,
                                                    diagnostics_state,
                                                    "Access granted; provider tabs detected.",
                                                    "Couldn't grant access:",
                                                    messaging::request_provider_tab_candidates(
                                                        workspace_id,
                                                        provider_id,
                                                    )
                                                    .await,
                                                );
                                            }
                                        });
                                    }
                                    on_open_tab=move || {
                                        if let Some(workspace_id) = workspace_id {
                                            leptos::task::spawn_local(async move {
                                                crate::state::controller::dispatch_user_command_result(
                                                    app_state,
                                                    workspace_state,
                                                    run_state,
                                                    binding_state,
                                                    message_state,
                                                    diagnostics_state,
                                                    "Provider tab opened.",
                                                    "Couldn't open the provider tab:",
                                                    messaging::open_provider_tab(workspace_id, provider_id, true).await,
                                                );
                                            });
                                        }
                                    }
                                />
                                {binding_for_panel.map(|binding| view! {
                                    <ProviderControlPanel
                                        workspace_id=workspace_id
                                        provider_id=provider_id
                                        binding=binding
                                        snapshot=snapshot
                                        app_state=app_state
                                        workspace_state=workspace_state
                                        run_state=run_state
                                        binding_state=binding_state
                                        message_state=message_state
                                        diagnostics_state=diagnostics_state
                                    />
                                })}
                                <ProviderCandidateList
                                    workspace_id=workspace_id
                                    provider_id=provider_id
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

#[allow(clippy::too_many_arguments)]
#[component]
fn ProviderCandidateList(
    workspace_id: Option<WorkspaceId>,
    provider_id: ProviderId,
    app_state: AppState,
    workspace_state: WorkspaceListState,
    run_state: ActiveRunState,
    binding_state: BindingState,
    message_state: MessageState,
    diagnostics_state: DiagnosticsState,
) -> impl IntoView {
    let candidates = Signal::derive(move || {
        app_state
            .provider_controls
            .get()
            .tab_candidates
            .get(&provider_id)
            .cloned()
            .unwrap_or_default()
    });

    view! {
        {move || (!candidates.get().is_empty()).then(|| view! {
            <div class="flex flex-col gap-2 mt-4">
                <span class="type-caption-strong text-primary">"Detected tabs"</span>
                <div class="flex flex-col gap-2">
                    {candidates
                        .get()
                        .into_iter()
                        .map(move |candidate| {
                            let candidate_for_bind = candidate.clone();
                            let label = candidate
                                .conversation_title
                                .clone()
                                .or(candidate.title.clone())
                                .unwrap_or_else(|| format!("Tab #{}", candidate.tab_id));
                            let subtitle = candidate
                                .conversation_id
                                .clone()
                                .or(candidate.url.clone())
                                .unwrap_or_else(|| "No conversation metadata detected".to_owned());
                            view! {
                                <Button
                                    variant=ButtonVariant::Secondary
                                    full_width=true
                                    aria_pressed=candidate.is_bound
                                    on_click=Box::new(move |_| {
                                        let Some(workspace_id) = workspace_id else {
                                            return;
                                        };
                                        let candidate = candidate_for_bind.clone();
                                        leptos::task::spawn_local(async move {
                                            crate::state::controller::dispatch_user_command_result(
                                                app_state,
                                                workspace_state,
                                                run_state,
                                                binding_state,
                                                message_state,
                                                diagnostics_state,
                                                "Provider tab bound.",
                                                "Couldn't bind the provider tab:",
                                                messaging::bind_provider_tab(
                                                    workspace_id,
                                                    provider_id,
                                                    candidate.tab_id,
                                                    candidate.window_id,
                                                    candidate.url.as_deref().and_then(url_origin),
                                                    candidate.title.clone(),
                                                    candidate.url.clone(),
                                                    candidate.conversation_id.clone(),
                                                    candidate.conversation_title.clone(),
                                                    candidate.url.clone(),
                                                    true,
                                                )
                                                .await,
                                            );
                                        });
                                    })
                                >
                                    <span class="flex flex-col gap-1">
                                        <span class="type-caption-strong text-primary">{label}</span>
                                        <span class="type-caption text-tertiary break-words">{subtitle}</span>
                                    </span>
                                </Button>
                            }
                        })
                        .collect_view()}
                </div>
                <p class="type-caption text-tertiary">
                    "Choose the exact conversation tab Chatmux may read and send through."
                </p>
            </div>
        })}
    }
}

#[component]
fn ProviderControlPanel(
    workspace_id: Option<WorkspaceId>,
    provider_id: ProviderId,
    binding: chatmux_common::ParticipantBinding,
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
    let bound_ref = binding.bound_conversation_ref.clone();
    let current_ref = binding.conversation_ref.clone();
    let chat_mismatch = binding.has_bound_target() && !binding.matches_bound_target();
    let controls_locked = chat_mismatch;
    let bound_ref_for_recover = bound_ref.clone();

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
    let strategy = state.last_strategy.map(strategy_label).unwrap_or("Unknown");
    let strategy_detail = state
        .last_strategy
        .map(strategy_detail_label)
        .unwrap_or("Control state unavailable.");
    let recover_bound_chat = move || {
        if let Some(workspace_id) = workspace_id {
            if let Some(conversation_id) = bound_ref_for_recover
                .as_ref()
                .and_then(|item| item.conversation_id.clone())
            {
                leptos::task::spawn_local(async move {
                    dispatch(
                        messaging::select_provider_conversation(
                            workspace_id,
                            provider_id,
                            conversation_id,
                        )
                        .await,
                    );
                });
                return;
            }

            leptos::task::spawn_local(async move {
                dispatch(messaging::open_provider_tab(workspace_id, provider_id, true).await);
            });
            return;
        }

        if let Some(url) = bound_ref_for_recover
            .as_ref()
            .and_then(|item| item.url.clone())
        {
            leptos::task::spawn_local(async move {
                let _ = messaging::open_tab(&url).await;
            });
        }
    };

    view! {
        <div class="flex flex-col gap-3 mt-4">
            <div class="flex flex-col gap-1">
                <span class="type-caption text-secondary">
                    {bound_ref
                        .as_ref()
                        .and_then(|item| item.title.clone())
                        .or_else(|| bound_ref.as_ref().and_then(|item| item.conversation_id.clone()))
                        .unwrap_or_else(|| "No bound chat target".to_owned())}
                </span>
                {bound_ref
                    .as_ref()
                    .and_then(|item| item.conversation_id.clone())
                    .map(|conversation_id| view! {
                        <span class="type-caption text-tertiary">
                            {format!("Bound chat ID: {conversation_id}")}
                        </span>
                    })}
                {bound_ref
                    .as_ref()
                    .and_then(|item| item.url.clone())
                    .map(|url| view! {
                        <span class="type-caption text-tertiary break-words">
                            {format!("Bound URL: {url}")}
                        </span>
                    })}
                <span class="type-caption text-secondary" style="margin-top: var(--space-2);">
                    {current_ref
                        .as_ref()
                        .and_then(|item| item.title.clone())
                        .or_else(|| current_ref.as_ref().and_then(|item| item.conversation_id.clone()))
                        .or_else(|| binding.tab_title.clone())
                        .unwrap_or_else(|| "Current chat not detected yet".to_owned())}
                </span>
                {current_ref
                    .as_ref()
                    .and_then(|item| item.conversation_id.clone())
                    .map(|conversation_id| view! {
                        <span class="type-caption text-tertiary">
                            {format!("Current chat ID: {conversation_id}")}
                        </span>
                    })}
                {current_ref
                    .as_ref()
                    .and_then(|item| item.url.clone())
                    .or_else(|| binding.tab_url.clone())
                    .map(|url| view! {
                        <span class="type-caption text-tertiary break-words">
                            {format!("Current URL: {url}")}
                        </span>
                    })}
            </div>

            {chat_mismatch.then(|| view! {
                <div
                    class="flex flex-col gap-3 rounded-md border p-4"
                    style="background: var(--status-warning-muted); border-color: var(--status-warning-border);"
                >
                    <div class="flex flex-col gap-1">
                        <span class="type-body-strong" style="color: var(--status-warning-text);">
                            "Bound chat mismatch"
                        </span>
                        <span class="type-caption text-secondary">
                            "This tab is on a different chat. Refresh stays available, but sync and provider actions are blocked until you switch back."
                        </span>
                    </div>
                    <div class="flex items-center gap-2 flex-wrap">
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Small
                            on_click=Box::new(move |_| recover_bound_chat())
                        >
                            "Switch to Bound Chat"
                        </Button>
                    </div>
                </div>
            })}

            <div class="flex items-center gap-2 flex-wrap">
                <span class="type-caption text-secondary">
                    {format!("Strategy: {}", strategy)}
                </span>
                {state.degraded.then(|| view! {
                    <span class="type-caption" style="color: var(--status-warning-solid);">
                        "Limited controls on this page"
                    </span>
                })}
            </div>
            <p class="type-caption text-tertiary">{strategy_detail}</p>

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
                    "Refresh controls"
                </Button>
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Small
                    disabled=controls_locked
                    on_click=Box::new(move |_| {
                        if let Some(workspace_id) = workspace_id {
                            leptos::task::spawn_local(async move {
                                dispatch(messaging::sync_provider_conversation(workspace_id, provider_id).await);
                            });
                        }
                    })
                >
                    "Sync transcript"
                </Button>
            </div>
            <p class="type-caption text-tertiary">
                "Refresh Controls rereads projects, conversations, models, and reasoning from the current page. Sync Transcript also refreshes chat metadata and imports the visible conversation history."
            </p>

            {snapshot.capabilities.supports_projects.then(|| view! {
                <div class="flex flex-col gap-2">
                    <label class="type-caption text-secondary">"Projects"</label>
                    <div class="flex gap-2">
                        <div class="flex-1">
                            <TextInput
                                value=project_title
                                on_input=move |val| set_project_title.set(val)
                                placeholder="Create project"
                            />
                        </div>
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Small
                            disabled=controls_locked
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
                                    class="control-pill type-caption cursor-pointer"
                                    disabled=controls_locked
                                    aria-pressed=move || if project.is_active { "true" } else { "false" }
                                    on:click=move |_| {
                                        if controls_locked {
                                            return;
                                        }
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
                        <div class="flex-1">
                            <TextInput
                                value=conversation_title
                                on_input=move |val| set_conversation_title.set(val)
                                placeholder="Create timestamped chat"
                            />
                        </div>
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Small
                            disabled=controls_locked
                            on_click=Box::new(move |_| {
                                if let Some(workspace_id) = workspace_id {
                                    let title = conversation_title.get_untracked();
                                    let title = if title.trim().is_empty() {
                                        format!(
                                            "Chatmux {}",
                                            format_local_title_timestamp(chrono::Utc::now())
                                        )
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
                            "New chat"
                        </Button>
                    </div>
                    <div class="flex flex-col gap-2">
                        {snapshot.conversations.clone().into_iter().take(10).map(move |conversation| {
                            let conversation_id = conversation.id.clone();
                            view! {
                                <button
                                    class="control-pill control-pill--row type-caption text-left cursor-pointer"
                                    disabled=controls_locked
                                    aria-pressed=move || if conversation.is_active { "true" } else { "false" }
                                    on:click=move |_| {
                                        if controls_locked {
                                            return;
                                        }
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
                                    class="control-pill type-caption cursor-pointer"
                                    disabled=controls_locked
                                    aria-pressed=move || if model.is_active { "true" } else { "false" }
                                    on:click=move |_| {
                                        if controls_locked {
                                            return;
                                        }
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
                                    class="control-pill type-caption cursor-pointer"
                                    disabled=controls_locked
                                    aria-pressed=move || if is_active { "true" } else { "false" }
                                    on:click=move |_| {
                                        if controls_locked {
                                            return;
                                        }
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

fn provider_ids() -> [ProviderId; 4] {
    [
        ProviderId::Gpt,
        ProviderId::Gemini,
        ProviderId::Grok,
        ProviderId::Claude,
    ]
}

fn single_rendered_payload(result: &Result<Vec<crate::models::UiEvent>, String>) -> Option<String> {
    let mut payloads = result
        .as_ref()
        .ok()?
        .iter()
        .filter_map(|event| match event {
            crate::models::UiEvent::DispatchUpdated { dispatch } => {
                Some(dispatch.rendered_payload.clone())
            }
            _ => None,
        });
    let payload = payloads.next()?;
    payloads.next().is_none().then_some(payload)
}

fn provider_permission_origins(provider_id: ProviderId) -> &'static [&'static str] {
    match provider_id {
        ProviderId::Gpt => &["https://chat.openai.com/*", "https://chatgpt.com/*"],
        ProviderId::Gemini => &["https://gemini.google.com/*"],
        ProviderId::Grok => &["https://grok.com/*", "https://x.com/*"],
        ProviderId::Claude => &["https://claude.ai/*"],
        ProviderId::User | ProviderId::System => &[],
    }
}

fn provider_targets(
    enabled_providers: &std::collections::BTreeSet<ProviderId>,
    bindings: &[crate::models::ParticipantBinding],
    runtime_health: &std::collections::BTreeMap<
        ProviderId,
        crate::state::app_state::ProviderRuntimeState,
    >,
) -> Vec<Target> {
    [
        (ProviderId::Gpt, Provider::Gpt),
        (ProviderId::Gemini, Provider::Gemini),
        (ProviderId::Grok, Provider::Grok),
        (ProviderId::Claude, Provider::Claude),
    ]
    .into_iter()
    .map(|(provider_id, provider)| Target {
        provider,
        availability: provider_target_availability(
            provider_id,
            enabled_providers,
            bindings,
            runtime_health,
        ),
    })
    .collect()
}

fn provider_target_availability(
    provider_id: ProviderId,
    enabled_providers: &std::collections::BTreeSet<ProviderId>,
    bindings: &[crate::models::ParticipantBinding],
    runtime_health: &std::collections::BTreeMap<
        ProviderId,
        crate::state::app_state::ProviderRuntimeState,
    >,
) -> TargetAvailability {
    if !enabled_providers.contains(&provider_id) {
        return TargetAvailability::WorkspaceDisabled;
    }
    let Some(binding) = bindings
        .iter()
        .find(|binding| binding.provider_id == provider_id)
    else {
        return TargetAvailability::Unbound;
    };
    if binding.tab_id.is_none() {
        return TargetAvailability::Unbound;
    }
    if binding.stale {
        return TargetAvailability::StaleBinding;
    }
    if !binding.matches_bound_target() {
        return TargetAvailability::ConversationChanged;
    }
    let health = runtime_health
        .get(&provider_id)
        .map(|state| state.health)
        .unwrap_or(binding.health_state);
    match health {
        crate::models::ProviderHealth::PermissionMissing => TargetAvailability::PermissionMissing,
        crate::models::ProviderHealth::Ready | crate::models::ProviderHealth::Completed => {
            if binding.capability_snapshot.can_auto_send {
                TargetAvailability::Available
            } else {
                TargetAvailability::Unsupported
            }
        }
        crate::models::ProviderHealth::Composing
        | crate::models::ProviderHealth::Sending
        | crate::models::ProviderHealth::Generating
            if binding.capability_snapshot.can_auto_send
                && binding
                    .capability_snapshot
                    .supports_follow_up_while_generating =>
        {
            TargetAvailability::Available
        }
        crate::models::ProviderHealth::Disconnected
        | crate::models::ProviderHealth::Composing
        | crate::models::ProviderHealth::Sending
        | crate::models::ProviderHealth::Generating
        | crate::models::ProviderHealth::LoginRequired
        | crate::models::ProviderHealth::DomMismatch
        | crate::models::ProviderHealth::Blocked
        | crate::models::ProviderHealth::RateLimited
        | crate::models::ProviderHealth::SendFailed
        | crate::models::ProviderHealth::CaptureUncertain
        | crate::models::ProviderHealth::DegradedManualOnly => TargetAvailability::Unhealthy,
    }
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
        ProviderStrategy::Dom => "DOM / page controls",
        ProviderStrategy::Manual => "Manual",
    }
}

fn strategy_detail_label(strategy: ProviderStrategy) -> &'static str {
    match strategy {
        ProviderStrategy::PublicApi => "Using a provider API integration.",
        ProviderStrategy::Network => {
            "Using provider network responses discovered from the page session."
        }
        ProviderStrategy::Dom => {
            "Using controls and metadata read directly from the open provider page."
        }
        ProviderStrategy::Manual => {
            "Manual-only mode. Chatmux can inspect state but cannot drive provider actions automatically."
        }
    }
}

fn url_origin(value: &str) -> Option<String> {
    let (scheme, rest) = value.split_once("://")?;
    let host = rest.split('/').next()?;
    Some(format!("{scheme}://{host}"))
}

fn extension_workspace_url(workspace_id: WorkspaceId) -> String {
    let window = web_sys::window().expect("no window");
    let location = window.location();
    let href = location
        .href()
        .unwrap_or_else(|_| "ui/index.html".to_owned());
    let base = href.split('?').next().unwrap_or(href.as_str());
    format!("{base}?workspace={}", workspace_id.0)
}

fn workspace_id_from_result(
    result: &Result<Vec<crate::models::UiEvent>, String>,
) -> Option<WorkspaceId> {
    result.as_ref().ok().and_then(|events| {
        events.iter().find_map(|event| match event {
            crate::models::UiEvent::WorkspaceSnapshot { snapshot } => {
                snapshot.workspace.as_ref().map(|workspace| workspace.id)
            }
            _ => None,
        })
    })
}
