//! Background coordinator and run lifecycle orchestration.

use crate::routing::{
    advance_cursor, barrier_satisfied, compile_graph, select_messages_for_edge, should_stop_run,
};
use crate::storage::{SettingsState, StateStore, StorageError};
use crate::template::render_template;
use chatmux_common::{
    AdapterToBackground, ApprovalMode, BarrierPolicy, BindingId, CapabilitySnapshot,
    ContextStrategy, DeliveryCursor, DeliveryCursorId, DiagnosticEvent, DiagnosticLevel, Dispatch,
    DispatchOutcome, EdgePolicy, ExportFormat, Message, MessageRole, OrchestrationMode,
    ParticipantBinding, ProviderControlSnapshot, ProviderControlState, ProviderHealth, ProviderId,
    Round, RoundStatus, Run, RunLedger, RunStatus, Template, UiCommand, UiEvent, Workspace,
    WorkspaceSnapshot,
};
use chatmux_export as export_engine;
use chrono::Utc;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct BackgroundCoordinator<S> {
    store: S,
}

impl<S> BackgroundCoordinator<S>
where
    S: StateStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn snapshot_workspace(
        &self,
        workspace_id: chatmux_common::WorkspaceId,
    ) -> Result<WorkspaceSnapshot, StorageError> {
        let settings = self.store.load_settings().await?;
        let bindings = self.store.list_bindings(workspace_id).await?;
        Ok(WorkspaceSnapshot {
            workspace: self.store.get_workspace(workspace_id).await?,
            bindings: bindings.clone(),
            provider_controls: bindings
                .into_iter()
                .map(provider_control_snapshot_from_binding)
                .collect(),
            runs: self.store.list_runs(workspace_id).await?,
            recent_messages: self.store.list_messages(workspace_id).await?,
            diagnostics: self.store.list_diagnostics(workspace_id).await?,
            edge_policies: self.store.list_edge_policies(workspace_id).await?,
            delivery_cursors: self.store.list_cursors(workspace_id).await?,
            templates: self.store.list_templates(workspace_id).await?,
            export_profiles: self.store.list_export_profiles(workspace_id).await?,
            kill_switch_active: settings.kill_switch_active,
        })
    }

    pub async fn run_ledger(
        &self,
        run_id: chatmux_common::RunId,
    ) -> Result<RunLedger, StorageError> {
        let run = self.store.get_run(run_id).await?;
        let workspace_id = run.as_ref().map(|item| item.workspace_id);
        Ok(RunLedger {
            run,
            rounds: self.store.list_rounds(run_id).await?,
            dispatches: self.store.list_dispatches(run_id).await?,
            delivery_cursors: if let Some(workspace_id) = workspace_id {
                self.store.list_cursors(workspace_id).await?
            } else {
                Vec::new()
            },
        })
    }

    pub async fn handle_ui_command(
        &self,
        command: UiCommand,
    ) -> Result<Vec<UiEvent>, StorageError> {
        match command {
            UiCommand::RequestWorkspaceList => Ok(vec![UiEvent::WorkspaceList {
                workspaces: self.store.list_workspaces().await?,
            }]),
            UiCommand::CreateWorkspace { name } => {
                let workspace = Workspace {
                    id: chatmux_common::WorkspaceId::new(),
                    name,
                    archived: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    enabled_providers: BTreeSet::from([
                        ProviderId::Gpt,
                        ProviderId::Gemini,
                        ProviderId::Grok,
                        ProviderId::Claude,
                    ]),
                    default_mode: OrchestrationMode::Broadcast,
                    default_context_strategy: ContextStrategy::WorkspaceDefault,
                    default_template_id: None,
                    active_export_profile_ids: vec![],
                    tags: vec![],
                    notes: None,
                };
                self.store.save_workspace(workspace.clone()).await?;
                Ok(vec![UiEvent::WorkspaceList {
                    workspaces: self.store.list_workspaces().await?,
                }])
            }
            UiCommand::DeleteWorkspace { workspace_id }
            | UiCommand::ClearWorkspaceData { workspace_id } => {
                self.store.delete_workspace(workspace_id).await?;
                Ok(vec![UiEvent::WorkspaceList {
                    workspaces: self.store.list_workspaces().await?,
                }])
            }
            UiCommand::SetWorkspaceArchived {
                workspace_id,
                archived,
            } => {
                let Some(mut workspace) = self.store.get_workspace(workspace_id).await? else {
                    return Ok(vec![]);
                };
                workspace.archived = archived;
                workspace.updated_at = Utc::now();
                self.store.save_workspace(workspace).await?;
                Ok(vec![UiEvent::WorkspaceSnapshot {
                    snapshot: self.snapshot_workspace(workspace_id).await?,
                }])
            }
            UiCommand::RenameWorkspace { workspace_id, name } => {
                let Some(mut workspace) = self.store.get_workspace(workspace_id).await? else {
                    return Ok(vec![]);
                };
                workspace.name = name;
                workspace.updated_at = Utc::now();
                self.store.save_workspace(workspace).await?;
                Ok(vec![UiEvent::WorkspaceSnapshot {
                    snapshot: self.snapshot_workspace(workspace_id).await?,
                }])
            }
            UiCommand::OpenWorkspace { workspace_id }
            | UiCommand::RequestWorkspaceSnapshot { workspace_id } => {
                // TODO(frontend): The UI needs the canonical workspace snapshot whenever a
                // workspace is opened or refreshed. It will need the workspace metadata,
                // bindings, run list, recent messages, diagnostics, edge policies,
                // delivery cursors, templates, export profiles, and kill-switch state.
                Ok(vec![UiEvent::WorkspaceSnapshot {
                    snapshot: self.snapshot_workspace(workspace_id).await?,
                }])
            }
            UiCommand::PersistTemplate { template } => {
                let workspace_id = template.workspace_id;
                self.store.save_template(template).await?;
                Ok(vec![UiEvent::WorkspaceSnapshot {
                    snapshot: self.snapshot_workspace(workspace_id).await?,
                }])
            }
            UiCommand::PersistEdgePolicy { policy } => {
                let workspace_id = policy.workspace_id;
                self.store.save_edge_policy(policy).await?;
                Ok(vec![UiEvent::WorkspaceSnapshot {
                    snapshot: self.snapshot_workspace(workspace_id).await?,
                }])
            }
            UiCommand::PersistExportProfile { profile } => {
                let workspace_id = profile.workspace_id;
                self.store.save_export_profile(profile).await?;
                Ok(vec![UiEvent::WorkspaceSnapshot {
                    snapshot: self.snapshot_workspace(workspace_id).await?,
                }])
            }
            UiCommand::StartRun { workspace_id, mode } => {
                let snapshot = self.snapshot_workspace(workspace_id).await?;
                let Some(workspace) = snapshot.workspace.clone() else {
                    return Ok(vec![]);
                };
                let participants = workspace.enabled_providers.clone();
                let graph = compile_graph(mode, &participants);
                let run = Run {
                    id: chatmux_common::RunId::new(),
                    workspace_id,
                    mode,
                    graph_snapshot: graph,
                    participant_set: participants,
                    barrier_policy: BarrierPolicy::WaitForAll,
                    timing_policy: chatmux_common::TimingPolicy::default(),
                    stop_policy: chatmux_common::StopPolicy::default(),
                    status: RunStatus::Running,
                    started_at: Some(Utc::now()),
                    ended_at: None,
                };
                self.store.save_run(run.clone()).await?;
                let round = Round {
                    id: chatmux_common::RoundId::new(),
                    run_id: run.id,
                    round_number: 1,
                    started_at: Some(Utc::now()),
                    completed_at: None,
                    status: RoundStatus::Running,
                };
                self.store.save_round(round.clone()).await?;
                // TODO(frontend): The UI needs the run record and round list when a run starts.
                // It will need run status, graph snapshot, participant set, timing/barrier
                // policies, and the first round metadata so it can represent execution state.
                Ok(vec![UiEvent::RunUpdated {
                    run,
                    rounds: vec![round],
                }])
            }
            UiCommand::PauseRun { run_id }
            | UiCommand::ResumeRun { run_id }
            | UiCommand::StopRun { run_id }
            | UiCommand::AbortRun { run_id } => {
                let Some(mut run) = self.store.get_run(run_id).await? else {
                    return Ok(vec![]);
                };
                run.status = match command {
                    UiCommand::PauseRun { .. } => RunStatus::Paused,
                    UiCommand::ResumeRun { .. } => RunStatus::Running,
                    UiCommand::StopRun { .. } => RunStatus::Completed,
                    UiCommand::AbortRun { .. } => RunStatus::Aborted,
                    _ => unreachable!(),
                };
                if matches!(run.status, RunStatus::Aborted | RunStatus::Completed) {
                    run.ended_at = Some(Utc::now());
                }
                self.store.save_run(run.clone()).await?;
                Ok(vec![UiEvent::RunUpdated {
                    run: run.clone(),
                    rounds: self.store.list_rounds(run.id).await?,
                }])
            }
            UiCommand::StepRun { run_id } => {
                let Some(run) = self.store.get_run(run_id).await? else {
                    return Ok(vec![]);
                };
                let mut rounds = self.store.list_rounds(run.id).await?;
                let next_round_number = rounds
                    .iter()
                    .map(|round| round.round_number)
                    .max()
                    .unwrap_or(0)
                    + 1;
                let round = Round {
                    id: chatmux_common::RoundId::new(),
                    run_id,
                    round_number: next_round_number,
                    started_at: Some(Utc::now()),
                    completed_at: None,
                    status: RoundStatus::Running,
                };
                self.store.save_round(round.clone()).await?;
                rounds.push(round);
                Ok(vec![UiEvent::RunUpdated { run, rounds }])
            }
            UiCommand::SendManualMessage {
                workspace_id,
                targets,
                text,
                approval_mode,
            } => {
                let Some(workspace) = self.store.get_workspace(workspace_id).await? else {
                    return Ok(vec![]);
                };
                let user_message = Message {
                    id: chatmux_common::MessageId::new(),
                    workspace_id,
                    participant_id: ProviderId::User,
                    role: MessageRole::User,
                    round: None,
                    timestamp: Utc::now(),
                    body_text: text.clone(),
                    body_blocks: vec![chatmux_common::Block::Paragraph { text }],
                    source_binding_id: None,
                    dispatch_id: None,
                    raw_capture_ref: None,
                    tags: vec![],
                    capture_confidence: chatmux_common::CaptureConfidence::Certain,
                };
                self.store.save_message(user_message.clone()).await?;

                let templates = self.store.list_templates(workspace_id).await?;
                let template = templates.first().cloned().unwrap_or_else(|| Template {
                    id: chatmux_common::TemplateId::new(),
                    workspace_id,
                    kind: chatmux_common::TemplateKind::BuiltinWrappedXml,
                    name: "Built-in Wrapped".to_owned(),
                    version: "1.0.0".to_owned(),
                    body_template: "{{message_bundle}}".to_owned(),
                    preamble: None,
                    metadata_template: None,
                    filename_template: None,
                });

                let mut events = vec![UiEvent::MessageCaptured {
                    message: user_message.clone(),
                }];

                for target in targets {
                    let rendered = render_template(
                        &template,
                        target,
                        std::slice::from_ref(&user_message),
                        None,
                    );
                    let dispatch = Dispatch {
                        id: chatmux_common::DispatchId::new(),
                        run_id: chatmux_common::RunId::new(),
                        round_id: None,
                        round_number: 0,
                        target_participant_id: target,
                        source_message_ids: rendered.source_message_ids,
                        template_id: Some(template.id),
                        rendered_payload: rendered.body,
                        sent_at: Some(Utc::now()),
                        captured_at: None,
                        outcome: match approval_mode {
                            ApprovalMode::AutoSend => DispatchOutcome::Delivered,
                            ApprovalMode::RequireUserConfirmation
                            | ApprovalMode::DraftOnly
                            | ApprovalMode::CopyOnly
                            | ApprovalMode::ManualSend => DispatchOutcome::Skipped,
                        },
                        error_detail: None,
                        retry_count: 0,
                    };
                    self.store.save_dispatch(dispatch.clone()).await?;
                    events.push(UiEvent::DispatchUpdated { dispatch });
                }

                let _ = workspace;
                Ok(events)
            }
            UiCommand::SyncProviderConversation {
                workspace_id,
                provider,
            }
            | UiCommand::RequestProviderControlState {
                workspace_id,
                provider,
            }
            | UiCommand::CreateProviderProject {
                workspace_id,
                provider,
                ..
            }
            | UiCommand::SelectProviderProject {
                workspace_id,
                provider,
                ..
            }
            | UiCommand::CreateProviderConversation {
                workspace_id,
                provider,
                ..
            }
            | UiCommand::SelectProviderConversation {
                workspace_id,
                provider,
                ..
            }
            | UiCommand::SetProviderModel {
                workspace_id,
                provider,
                ..
            }
            | UiCommand::SetProviderReasoning {
                workspace_id,
                provider,
                ..
            }
            | UiCommand::SetProviderFeatureFlag {
                workspace_id,
                provider,
                ..
            } => {
                let binding = self
                    .upsert_binding_for_provider(workspace_id, provider, |_| {})
                    .await?;
                let snapshot = provider_control_snapshot_from_binding(binding);
                Ok(vec![UiEvent::ProviderControlUpdated {
                    workspace_id,
                    snapshot,
                }])
            }
            UiCommand::PersistProviderDefaults { provider, defaults } => {
                let mut settings = self.store.load_settings().await?;
                settings.provider_defaults.insert(provider, defaults.clone());
                self.store.save_settings(settings).await?;
                Ok(vec![UiEvent::ProviderDefaultsUpdated { provider, defaults }])
            }
            UiCommand::DeleteTemplate { template_id } => {
                self.store.delete_template(template_id).await?;
                Ok(vec![UiEvent::WorkspaceList {
                    workspaces: self.store.list_workspaces().await?,
                }])
            }
            UiCommand::ExportSelection {
                workspace_id,
                format,
                layout,
                profile_id,
            } => {
                let Some(workspace) = self.store.get_workspace(workspace_id).await? else {
                    return Ok(vec![]);
                };
                let messages = self.store.list_messages(workspace_id).await?;
                let all_runs = self.store.list_runs(workspace_id).await?;
                let export_profiles = self.store.list_export_profiles(workspace_id).await?;
                let selected_profile = profile_id.and_then(|selected_id| {
                    export_profiles
                        .iter()
                        .find(|profile| profile.id == selected_id)
                        .cloned()
                });
                let selected_runs = if let Some(run_id) = selected_profile
                    .as_ref()
                    .and_then(|profile| profile.filter_preset.run_id)
                {
                    all_runs
                        .iter()
                        .filter(|run| run.id == run_id)
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    all_runs.clone()
                };
                let mut dispatches = Vec::new();
                for run in &selected_runs {
                    dispatches.extend(self.store.list_dispatches(run.id).await?);
                }
                let diagnostics = self.store.list_diagnostics(workspace_id).await?;
                let templates = self.store.list_templates(workspace_id).await?;
                let template_name = dispatches
                    .iter()
                    .rev()
                    .find_map(|dispatch| dispatch.template_id)
                    .and_then(|template_id| {
                        templates
                            .iter()
                            .find(|template| template.id == template_id)
                            .map(|template| template.name.clone())
                    });
                let document = export_engine::build_export_document(
                    &workspace,
                    &messages,
                    &selected_runs,
                    &dispatches,
                    &diagnostics,
                    &export_engine::ExportBuildOptions {
                        template_name,
                        export_profile_name: selected_profile
                            .as_ref()
                            .map(|profile| profile.name.clone()),
                        browser_name: Some("browser-extension".to_owned()),
                        extension_version: Some(env!("CARGO_PKG_VERSION").to_owned()),
                        title: selected_profile
                            .as_ref()
                            .map(|profile| profile.name.clone())
                            .unwrap_or_else(|| workspace.name.clone()),
                    },
                );
                let body = export_engine::render_document(&document, format, layout, true)
                    .map_err(|error| StorageError::Invariant(error.to_string()))?;
                let filename = if let Some(profile) = &selected_profile {
                    export_engine::render_filename_template(
                        &profile.filename_template,
                        Some(&workspace),
                        format,
                    )
                } else {
                    export_engine::render_filename(
                        "{workspace}-{format}",
                        &std::collections::BTreeMap::from([
                            ("workspace", workspace.name.clone()),
                            ("format", format!("{format:?}")),
                        ]),
                    )
                    .map_err(|error| StorageError::Invariant(error.to_string()))?
                };
                // TODO(frontend): The UI needs the fully rendered export payload plus the
                // chosen format, MIME type, and suggested filename so it can drive either a
                // file download or a user-gesture clipboard copy flow.
                Ok(vec![UiEvent::ExportRendered {
                    format,
                    mime_type: mime_for_export(format).to_owned(),
                    filename,
                    body,
                }])
            }
            UiCommand::RequestMessageInspection { message_id } => {
                let message = self.store.get_message(message_id).await?;
                let dispatch = if let Some(dispatch_id) =
                    message.as_ref().and_then(|item| item.dispatch_id)
                {
                    self.find_dispatch(dispatch_id, message.as_ref().map(|item| item.workspace_id))
                        .await?
                } else {
                    None
                };
                Ok(vec![UiEvent::MessageInspection {
                    sent_payload: dispatch.as_ref().map(|item| item.rendered_payload.clone()),
                    raw_capture_ref: message
                        .as_ref()
                        .and_then(|item| item.raw_capture_ref.clone()),
                    message,
                    dispatch,
                }])
            }
            UiCommand::SetKillSwitch { active } => {
                let mut settings = self.store.load_settings().await?;
                settings.kill_switch_active = active;
                self.store.save_settings(settings).await?;
                Ok(vec![UiEvent::KillSwitchChanged { active }])
            }
            UiCommand::ToggleProvider {
                workspace_id,
                provider,
                enabled,
            } => {
                let Some(mut workspace) = self.store.get_workspace(workspace_id).await? else {
                    return Ok(vec![]);
                };
                if enabled {
                    workspace.enabled_providers.insert(provider);
                } else {
                    workspace.enabled_providers.remove(&provider);
                }
                self.store.save_workspace(workspace).await?;
                Ok(vec![UiEvent::WorkspaceSnapshot {
                    snapshot: self.snapshot_workspace(workspace_id).await?,
                }])
            }
        }
    }

    pub async fn ingest_adapter_event(
        &self,
        workspace_id: chatmux_common::WorkspaceId,
        event: AdapterToBackground,
    ) -> Result<Vec<UiEvent>, StorageError> {
        match event {
            AdapterToBackground::HealthReport { provider, health } => {
                Ok(vec![UiEvent::ProviderHealthChanged {
                    workspace_id,
                    provider,
                    health,
                    blocking_state: None,
                }])
            }
            AdapterToBackground::BlockingStateDetected {
                provider,
                blocking_state,
            } => {
                let diagnostic = DiagnosticEvent {
                    id: chatmux_common::DiagnosticEventId::new(),
                    workspace_id,
                    binding_id: None,
                    timestamp: Utc::now(),
                    level: DiagnosticLevel::Warning,
                    code: "blocking_state_detected".to_owned(),
                    detail: format!("{provider:?}: {blocking_state:?}"),
                    snapshot_ref: None,
                };
                self.store.save_diagnostic(diagnostic.clone()).await?;
                Ok(vec![
                    UiEvent::ProviderHealthChanged {
                        workspace_id,
                        provider,
                        health: ProviderHealth::Blocked,
                        blocking_state: Some(blocking_state),
                    },
                    UiEvent::DiagnosticRaised { diagnostic },
                ])
            }
            AdapterToBackground::MessagesCaptured {
                provider: _,
                messages,
            } => {
                let mut events = Vec::new();
                for mut message in messages {
                    message.workspace_id = workspace_id;
                    self.store.save_message(message.clone()).await?;
                    events.push(UiEvent::MessageCaptured { message });
                }
                Ok(events)
            }
            AdapterToBackground::StructuralProbePassed { provider } => {
                Ok(vec![UiEvent::ProviderHealthChanged {
                    workspace_id,
                    provider,
                    health: ProviderHealth::Ready,
                    blocking_state: None,
                }])
            }
            AdapterToBackground::StructuralProbeFailed { provider, detail } => {
                let diagnostic = DiagnosticEvent {
                    id: chatmux_common::DiagnosticEventId::new(),
                    workspace_id,
                    binding_id: None,
                    timestamp: Utc::now(),
                    level: DiagnosticLevel::Critical,
                    code: "dom_mismatch".to_owned(),
                    detail,
                    snapshot_ref: None,
                };
                self.store.save_diagnostic(diagnostic.clone()).await?;
                Ok(vec![
                    UiEvent::ProviderHealthChanged {
                        workspace_id,
                        provider,
                        health: ProviderHealth::DomMismatch,
                        blocking_state: None,
                    },
                    UiEvent::DiagnosticRaised { diagnostic },
                ])
            }
            AdapterToBackground::ConversationRefDiscovered {
                provider,
                conversation_ref,
            } => {
                let binding = self
                    .upsert_binding_for_provider(workspace_id, provider, |binding| {
                        binding.conversation_ref = conversation_ref.clone();
                        let provider_control = binding.provider_control.get_or_insert_with(
                            ProviderControlState::default,
                        );
                        provider_control.conversation_id = conversation_ref
                            .as_ref()
                            .and_then(|item| item.conversation_id.clone());
                        provider_control.conversation_title =
                            conversation_ref.as_ref().and_then(|item| item.title.clone());
                        provider_control.model_label =
                            conversation_ref.as_ref().and_then(|item| item.model_label.clone());
                        if binding.health_state == ProviderHealth::Disconnected {
                            binding.health_state = ProviderHealth::Ready;
                        }
                    })
                    .await?;

                Ok(vec![
                    UiEvent::ProviderHealthChanged {
                        workspace_id,
                        provider,
                        health: binding.health_state,
                        blocking_state: None,
                    },
                    UiEvent::ProviderControlUpdated {
                        workspace_id,
                        snapshot: provider_control_snapshot_from_binding(binding.clone()),
                    },
                    UiEvent::WorkspaceSnapshot {
                        snapshot: self.snapshot_workspace(workspace_id).await?,
                    },
                ])
            }
            AdapterToBackground::ProviderControlSnapshotCaptured { provider, snapshot } => {
                let binding = self
                    .upsert_binding_for_provider(workspace_id, provider, |binding| {
                        binding.provider_control = Some(snapshot.state.clone());
                        binding.conversation_ref = Some(chatmux_common::ConversationRef {
                            conversation_id: snapshot.state.conversation_id.clone(),
                            title: snapshot.state.conversation_title.clone(),
                            model_label: snapshot.state.model_label.clone(),
                        });
                        if binding.health_state == ProviderHealth::Disconnected {
                            binding.health_state = if snapshot.state.degraded {
                                ProviderHealth::DegradedManualOnly
                            } else {
                                ProviderHealth::Ready
                            };
                        }
                    })
                    .await?;
                let health = if snapshot.state.degraded {
                    ProviderHealth::DegradedManualOnly
                } else {
                    binding.health_state
                };

                Ok(vec![
                    UiEvent::ProviderHealthChanged {
                        workspace_id,
                        provider,
                        health,
                        blocking_state: None,
                    },
                    UiEvent::ProviderControlUpdated { workspace_id, snapshot },
                    UiEvent::WorkspaceSnapshot {
                        snapshot: self.snapshot_workspace(workspace_id).await?,
                    },
                ])
            }
            AdapterToBackground::CommandFailed {
                provider,
                level,
                detail,
            } => {
                let diagnostic = DiagnosticEvent {
                    id: chatmux_common::DiagnosticEventId::new(),
                    workspace_id,
                    binding_id: None,
                    timestamp: Utc::now(),
                    level,
                    code: "adapter_command_failed".to_owned(),
                    detail: detail.clone(),
                    snapshot_ref: None,
                };
                self.store.save_diagnostic(diagnostic.clone()).await?;
                Ok(vec![
                    UiEvent::ProviderHealthChanged {
                        workspace_id,
                        provider,
                        health: ProviderHealth::SendFailed,
                        blocking_state: None,
                    },
                    UiEvent::DiagnosticRaised { diagnostic },
                ])
            }
        }
    }

    pub async fn synthesize_dispatches(
        &self,
        run: &Run,
        policies: &[EdgePolicy],
        workspace_messages: &[Message],
        templates: &[Template],
    ) -> Result<Vec<Dispatch>, StorageError> {
        let mut dispatches = Vec::new();
        let mut responded = BTreeSet::new();
        let active = run.participant_set.clone();
        let cursors = self.store.list_cursors(run.workspace_id).await?;

        for edge in &run.graph_snapshot.edges {
            let Some(policy) = policies.iter().find(|policy| {
                policy.source_participant_id == edge.source
                    && policy.target_participant_id == edge.target
                    && policy.enabled
            }) else {
                continue;
            };

            let cursor = cursors.iter().find(|cursor| {
                cursor.source_participant_id == edge.source
                    && cursor.target_participant_id == edge.target
            });
            let selected_messages = select_messages_for_edge(workspace_messages, policy, cursor);
            let template = templates
                .iter()
                .find(|template| Some(template.id) == policy.template_id)
                .or_else(|| templates.first());
            let Some(template) = template else {
                continue;
            };

            let rendered = render_template(template, edge.target, &selected_messages, None);
            let dispatch = Dispatch {
                id: chatmux_common::DispatchId::new(),
                run_id: run.id,
                round_id: None,
                round_number: self.store.list_rounds(run.id).await?.len() as u32,
                target_participant_id: edge.target,
                source_message_ids: rendered.source_message_ids.clone(),
                template_id: Some(template.id),
                rendered_payload: rendered.body,
                sent_at: Some(Utc::now()),
                captured_at: None,
                outcome: DispatchOutcome::Delivered,
                error_detail: None,
                retry_count: 0,
            };
            self.store.save_dispatch(dispatch.clone()).await?;
            dispatches.push(dispatch);

            let cursor = cursor.cloned().unwrap_or(DeliveryCursor {
                id: DeliveryCursorId::new(),
                workspace_id: run.workspace_id,
                source_participant_id: edge.source,
                target_participant_id: edge.target,
                last_delivered_message_id: None,
                last_delivered_at: None,
                frozen: false,
            });
            let advanced = advance_cursor(&cursor, &selected_messages);
            self.store.save_cursor(advanced).await?;
            responded.insert(edge.target);
        }

        let completed_rounds = self
            .store
            .list_rounds(run.id)
            .await?
            .into_iter()
            .filter(|round| matches!(round.status, RoundStatus::Completed))
            .count() as u32;

        if barrier_satisfied(&run.barrier_policy, &responded, &active)
            && should_stop_run(&run.stop_policy, completed_rounds, 0, 0, &[])
        {
            let mut finished = run.clone();
            finished.status = RunStatus::Completed;
            finished.ended_at = Some(Utc::now());
            self.store.save_run(finished).await?;
        }

        Ok(dispatches)
    }

    pub async fn save_binding(&self, binding: ParticipantBinding) -> Result<(), StorageError> {
        self.store.save_binding(binding).await
    }

    pub async fn load_settings(&self) -> Result<SettingsState, StorageError> {
        self.store.load_settings().await
    }

    pub async fn save_settings(&self, settings: SettingsState) -> Result<(), StorageError> {
        self.store.save_settings(settings).await
    }

    async fn find_dispatch(
        &self,
        dispatch_id: chatmux_common::DispatchId,
        workspace_id: Option<chatmux_common::WorkspaceId>,
    ) -> Result<Option<Dispatch>, StorageError> {
        let Some(workspace_id) = workspace_id else {
            return Ok(None);
        };
        for run in self.store.list_runs(workspace_id).await? {
            if let Some(dispatch) = self
                .store
                .list_dispatches(run.id)
                .await?
                .into_iter()
                .find(|item| item.id == dispatch_id)
            {
                return Ok(Some(dispatch));
            }
        }
        Ok(None)
    }

    async fn upsert_binding_for_provider<F>(
        &self,
        workspace_id: chatmux_common::WorkspaceId,
        provider: ProviderId,
        update: F,
    ) -> Result<ParticipantBinding, StorageError>
    where
        F: FnOnce(&mut ParticipantBinding),
    {
        let existing = self
            .store
            .list_bindings(workspace_id)
            .await?
            .into_iter()
            .find(|binding| binding.provider_id == provider);

        let mut binding = existing.unwrap_or_else(|| ParticipantBinding {
            id: BindingId::new(),
            workspace_id,
            provider_id: provider,
            tab_id: None,
            window_id: None,
            origin: None,
            conversation_ref: None,
            provider_control: None,
            health_state: ProviderHealth::Ready,
            capability_snapshot: default_capability_snapshot(provider),
            last_seen_at: Some(Utc::now()),
        });

        update(&mut binding);
        binding.last_seen_at = Some(Utc::now());
        self.store.save_binding(binding.clone()).await?;
        Ok(binding)
    }
}

fn default_capability_snapshot(provider: ProviderId) -> CapabilitySnapshot {
    match provider {
        ProviderId::Gpt => CapabilitySnapshot {
            supports_follow_up_while_generating: false,
            can_auto_send: true,
            can_capture_full_history: true,
            can_capture_delta: true,
        },
        ProviderId::Gemini | ProviderId::Grok | ProviderId::Claude => CapabilitySnapshot {
            supports_follow_up_while_generating: false,
            can_auto_send: true,
            can_capture_full_history: false,
            can_capture_delta: false,
        },
        ProviderId::User | ProviderId::System => CapabilitySnapshot {
            supports_follow_up_while_generating: false,
            can_auto_send: false,
            can_capture_full_history: false,
            can_capture_delta: false,
        },
    }
}

fn provider_control_snapshot_from_binding(binding: ParticipantBinding) -> ProviderControlSnapshot {
    let mut state = binding.provider_control.unwrap_or_default();
    if state.conversation_id.is_none() {
        state.conversation_id = binding
            .conversation_ref
            .as_ref()
            .and_then(|item| item.conversation_id.clone());
    }
    if state.conversation_title.is_none() {
        state.conversation_title = binding.conversation_ref.as_ref().and_then(|item| item.title.clone());
    }
    if state.model_label.is_none() {
        state.model_label = binding
            .conversation_ref
            .as_ref()
            .and_then(|item| item.model_label.clone());
    }

    ProviderControlSnapshot {
        provider: binding.provider_id,
        capabilities: default_provider_control_capabilities(binding.provider_id),
        state,
        projects: Vec::new(),
        conversations: Vec::new(),
        models: Vec::new(),
        reasoning_options: Vec::new(),
        feature_flags: Vec::new(),
    }
}

fn default_provider_control_capabilities(
    provider: ProviderId,
) -> chatmux_common::ProviderControlCapabilities {
    match provider {
        ProviderId::Gpt => chatmux_common::ProviderControlCapabilities {
            supports_projects: true,
            supports_project_creation: true,
            supports_conversations: true,
            supports_conversation_creation: true,
            supports_model_selection: true,
            supports_reasoning_selection: true,
            supports_feature_flags: true,
            supports_sync: true,
        },
        ProviderId::Gemini | ProviderId::Grok | ProviderId::Claude => {
            chatmux_common::ProviderControlCapabilities {
                supports_projects: false,
                supports_project_creation: false,
                supports_conversations: true,
                supports_conversation_creation: true,
                supports_model_selection: false,
                supports_reasoning_selection: false,
                supports_feature_flags: false,
                supports_sync: true,
            }
        }
        ProviderId::User | ProviderId::System => chatmux_common::ProviderControlCapabilities {
            supports_projects: false,
            supports_project_creation: false,
            supports_conversations: false,
            supports_conversation_creation: false,
            supports_model_selection: false,
            supports_reasoning_selection: false,
            supports_feature_flags: false,
            supports_sync: false,
        },
    }
}

fn mime_for_export(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Markdown => "text/markdown",
        ExportFormat::Json => "application/json",
        ExportFormat::Toml => "application/toml",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStateStore;
    use chatmux_common::{
        Block, CaptureConfidence, ExportFilterPreset, ExportLayout, ExportProfile, ExportProfileId,
        ExportScopePreset, MessageId, MessageRole, MetadataIncludeFlags, RouteEdge, RoutingGraph,
        StopPolicy, TemplateId, TemplateKind, WorkspaceId,
    };
    use futures::executor::block_on;

    #[test]
    fn export_selection_uses_requested_profile_and_selected_run() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();

            let workspace = Workspace {
                id: workspace_id,
                name: "Workspace".to_owned(),
                archived: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                enabled_providers: BTreeSet::from([ProviderId::Gpt, ProviderId::Gemini]),
                default_mode: OrchestrationMode::Broadcast,
                default_context_strategy: ContextStrategy::WorkspaceDefault,
                default_template_id: None,
                active_export_profile_ids: Vec::new(),
                tags: Vec::new(),
                notes: None,
            };
            store
                .save_workspace(workspace)
                .await
                .expect("workspace saves");

            let older_run = Run {
                id: chatmux_common::RunId::new(),
                workspace_id,
                mode: OrchestrationMode::Broadcast,
                graph_snapshot: RoutingGraph {
                    nodes: BTreeSet::from([ProviderId::Gpt, ProviderId::Gemini]),
                    edges: Vec::new(),
                },
                participant_set: BTreeSet::from([ProviderId::Gpt, ProviderId::Gemini]),
                barrier_policy: BarrierPolicy::WaitForAll,
                timing_policy: chatmux_common::TimingPolicy::default(),
                stop_policy: StopPolicy::default(),
                status: RunStatus::Completed,
                started_at: Some(Utc::now()),
                ended_at: Some(Utc::now()),
            };
            let selected_run = Run {
                id: chatmux_common::RunId::new(),
                workspace_id,
                mode: OrchestrationMode::Roundtable,
                graph_snapshot: RoutingGraph {
                    nodes: BTreeSet::from([ProviderId::Gpt, ProviderId::Gemini]),
                    edges: Vec::new(),
                },
                participant_set: BTreeSet::from([ProviderId::Gpt, ProviderId::Gemini]),
                barrier_policy: BarrierPolicy::WaitForAll,
                timing_policy: chatmux_common::TimingPolicy::default(),
                stop_policy: StopPolicy::default(),
                status: RunStatus::Completed,
                started_at: Some(Utc::now()),
                ended_at: Some(Utc::now()),
            };
            store
                .save_run(older_run.clone())
                .await
                .expect("older run saves");
            store
                .save_run(selected_run.clone())
                .await
                .expect("selected run saves");

            let older_template = Template {
                id: TemplateId::new(),
                workspace_id,
                kind: TemplateKind::Custom,
                name: "Older Template".to_owned(),
                version: "1.0.0".to_owned(),
                body_template: "{{message_bundle}}".to_owned(),
                preamble: None,
                metadata_template: None,
                filename_template: None,
            };
            let selected_template = Template {
                id: TemplateId::new(),
                workspace_id,
                kind: TemplateKind::Custom,
                name: "Selected Template".to_owned(),
                version: "1.0.0".to_owned(),
                body_template: "{{message_bundle}}".to_owned(),
                preamble: None,
                metadata_template: None,
                filename_template: None,
            };
            store
                .save_template(older_template.clone())
                .await
                .expect("older template saves");
            store
                .save_template(selected_template.clone())
                .await
                .expect("selected template saves");

            let export_profile = ExportProfile {
                id: ExportProfileId::new(),
                workspace_id,
                name: "Focused Export".to_owned(),
                scope_preset: ExportScopePreset::SingleRun,
                filter_preset: ExportFilterPreset {
                    run_id: Some(selected_run.id),
                    ..ExportFilterPreset::default()
                },
                format: ExportFormat::Json,
                layout: ExportLayout::Chronological,
                include_flags: MetadataIncludeFlags::default(),
                filename_template: "focused-{workspace}-{format}".to_owned(),
                metadata_template: None,
                prefer_copy: false,
            };
            store
                .save_export_profile(export_profile.clone())
                .await
                .expect("profile saves");

            store
                .save_dispatch(Dispatch {
                    id: chatmux_common::DispatchId::new(),
                    run_id: older_run.id,
                    round_id: None,
                    round_number: 1,
                    target_participant_id: ProviderId::Gemini,
                    source_message_ids: Vec::new(),
                    template_id: Some(older_template.id),
                    rendered_payload: "older".to_owned(),
                    sent_at: Some(Utc::now()),
                    captured_at: None,
                    outcome: DispatchOutcome::Delivered,
                    error_detail: None,
                    retry_count: 0,
                })
                .await
                .expect("older dispatch saves");
            store
                .save_dispatch(Dispatch {
                    id: chatmux_common::DispatchId::new(),
                    run_id: selected_run.id,
                    round_id: None,
                    round_number: 1,
                    target_participant_id: ProviderId::Gemini,
                    source_message_ids: Vec::new(),
                    template_id: Some(selected_template.id),
                    rendered_payload: "selected".to_owned(),
                    sent_at: Some(Utc::now()),
                    captured_at: None,
                    outcome: DispatchOutcome::Delivered,
                    error_detail: None,
                    retry_count: 0,
                })
                .await
                .expect("selected dispatch saves");

            let events = coordinator
                .handle_ui_command(UiCommand::ExportSelection {
                    workspace_id,
                    format: ExportFormat::Json,
                    layout: ExportLayout::Chronological,
                    profile_id: Some(export_profile.id),
                })
                .await
                .expect("export succeeds");

            let UiEvent::ExportRendered { filename, body, .. } = &events[0] else {
                panic!("expected export rendered event");
            };
            assert_eq!(filename, "focused-workspace-json.json");

            let rendered: serde_json::Value =
                serde_json::from_str(body).expect("export body should be valid JSON");
            assert_eq!(
                rendered["metadata"]["export_profile_name"],
                export_profile.name
            );
            assert_eq!(
                rendered["metadata"]["run_id"],
                selected_run.id.0.to_string()
            );
            assert_eq!(
                rendered["metadata"]["template_name"],
                selected_template.name
            );
            assert_eq!(rendered["dispatches"].as_array().map(Vec::len), Some(1));
            assert_eq!(rendered["dispatches"][0]["rendered_payload"], "selected");
        });
    }

    #[test]
    fn synthesize_dispatches_stops_by_completed_rounds_not_dispatch_count() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            let run = Run {
                id: chatmux_common::RunId::new(),
                workspace_id,
                mode: OrchestrationMode::Roundtable,
                graph_snapshot: RoutingGraph {
                    nodes: BTreeSet::from([
                        ProviderId::Gpt,
                        ProviderId::Gemini,
                        ProviderId::Claude,
                    ]),
                    edges: vec![
                        RouteEdge {
                            source: ProviderId::Gpt,
                            target: ProviderId::Gemini,
                            policy_id: None,
                        },
                        RouteEdge {
                            source: ProviderId::Gpt,
                            target: ProviderId::Claude,
                            policy_id: None,
                        },
                    ],
                },
                participant_set: BTreeSet::from([
                    ProviderId::Gpt,
                    ProviderId::Gemini,
                    ProviderId::Claude,
                ]),
                barrier_policy: BarrierPolicy::FirstFinisher,
                timing_policy: chatmux_common::TimingPolicy::default(),
                stop_policy: StopPolicy {
                    stop_on_max_rounds: true,
                    stop_on_manual_pause: false,
                    stop_on_sentinel_phrase: None,
                    repeated_provider_failure_limit: None,
                    repeated_timeout_limit: None,
                    stagnation_window: Some(2),
                    require_approval_between_rounds: true,
                },
                status: RunStatus::Running,
                started_at: Some(Utc::now()),
                ended_at: None,
            };
            store.save_run(run.clone()).await.expect("run saves");
            store
                .save_round(Round {
                    id: chatmux_common::RoundId::new(),
                    run_id: run.id,
                    round_number: 1,
                    started_at: Some(Utc::now()),
                    completed_at: Some(Utc::now()),
                    status: RoundStatus::Completed,
                })
                .await
                .expect("completed round saves");

            let template = Template {
                id: TemplateId::new(),
                workspace_id,
                kind: TemplateKind::Custom,
                name: "Template".to_owned(),
                version: "1.0.0".to_owned(),
                body_template: "{{message_bundle}}".to_owned(),
                preamble: None,
                metadata_template: None,
                filename_template: None,
            };
            let policies = vec![
                EdgePolicy {
                    id: chatmux_common::EdgePolicyId::new(),
                    workspace_id,
                    source_participant_id: ProviderId::Gpt,
                    target_participant_id: ProviderId::Gemini,
                    enabled: true,
                    catch_up_policy: chatmux_common::CatchUpPolicy::FullHistory,
                    incremental_policy: chatmux_common::IncrementalPolicy::FullHistoryEveryTime,
                    self_exclusion: true,
                    include_user_turns: true,
                    include_system_notes: false,
                    include_pinned_summaries: false,
                    include_moderator_annotations: false,
                    include_target_prior_turns: false,
                    truncation_policy: chatmux_common::TruncationPolicy::None,
                    priority: 0,
                    approval_mode: ApprovalMode::AutoSend,
                    template_id: Some(template.id),
                },
                EdgePolicy {
                    id: chatmux_common::EdgePolicyId::new(),
                    workspace_id,
                    source_participant_id: ProviderId::Gpt,
                    target_participant_id: ProviderId::Claude,
                    enabled: true,
                    catch_up_policy: chatmux_common::CatchUpPolicy::FullHistory,
                    incremental_policy: chatmux_common::IncrementalPolicy::FullHistoryEveryTime,
                    self_exclusion: true,
                    include_user_turns: true,
                    include_system_notes: false,
                    include_pinned_summaries: false,
                    include_moderator_annotations: false,
                    include_target_prior_turns: false,
                    truncation_policy: chatmux_common::TruncationPolicy::None,
                    priority: 0,
                    approval_mode: ApprovalMode::AutoSend,
                    template_id: Some(template.id),
                },
            ];
            let workspace_messages = vec![Message {
                id: MessageId::new(),
                workspace_id,
                participant_id: ProviderId::Gpt,
                role: MessageRole::Assistant,
                round: Some(1),
                timestamp: Utc::now(),
                body_text: "source".to_owned(),
                body_blocks: vec![Block::Paragraph {
                    text: "source".to_owned(),
                }],
                source_binding_id: None,
                dispatch_id: None,
                raw_capture_ref: None,
                tags: Vec::new(),
                capture_confidence: CaptureConfidence::Certain,
            }];

            coordinator
                .synthesize_dispatches(&run, &policies, &workspace_messages, &[template])
                .await
                .expect("dispatch synthesis succeeds");

            let ledger = coordinator.run_ledger(run.id).await.expect("ledger loads");
            assert_eq!(
                ledger.run.expect("run should exist").status,
                RunStatus::Running
            );
        });
    }
}
