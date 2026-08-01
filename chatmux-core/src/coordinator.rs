//! Background coordinator and run lifecycle orchestration.

use crate::dispatch::{mark_captured, mark_delivered, mark_failed, prepared_outcome};
use crate::routing::{
    advance_cursor, barrier_satisfied, compile_configured_graph, compile_graph,
    select_messages_for_edge, should_stop_run,
};
use crate::storage::{SettingsState, StateStore, StorageError};
use crate::template::{builtin_templates, render_template};
use chatmux_common::{
    AdapterToBackground, BarrierPolicy, BindingId, CapabilitySnapshot, CatchUpPolicy,
    ContextStrategy, DeliveryCursor, DeliveryCursorId, DiagnosticEvent, DiagnosticLevel,
    DiagnosticScope, DiagnosticSource, DiagnosticsQuery, DiagnosticsSnapshot, Dispatch, EdgePolicy,
    ExportFormat, ExportRequest, ExportScopePreset, Message, MessageRole, MetadataIncludeFlags,
    NextRoundPackage, OrchestrationMode, ParticipantBinding, ProviderControlSnapshot,
    ProviderControlState, ProviderHealth, ProviderId, Round, RoundStatus, Run, RunConfiguration,
    RunLedger, RunStatus, Template, TruncationPolicy, UiCommand, UiEvent, Workspace,
    WorkspaceArchive, WorkspaceDiagnosticsSummary, WorkspaceSnapshot,
};
use chatmux_export as export_engine;
use chrono::Utc;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct BackgroundCoordinator<S> {
    store: S,
}

struct ManualMessagePreparation {
    workspace_mode: OrchestrationMode,
    parent_message: Option<Message>,
    user_message: Message,
    packages: Vec<NextRoundPackage>,
    templates_to_persist: Vec<Template>,
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
        let diagnostics = self.store.list_diagnostics(workspace_id).await?;
        Ok(WorkspaceSnapshot {
            workspace: self.store.get_workspace(workspace_id).await?,
            bindings: bindings.clone(),
            provider_controls: bindings
                .into_iter()
                .map(provider_control_snapshot_from_binding)
                .collect(),
            runs: self.store.list_runs(workspace_id).await?,
            recent_messages: self.store.list_messages(workspace_id).await?,
            diagnostics_summary: summarize_diagnostics(Some(workspace_id), &diagnostics),
            diagnostics,
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

    async fn workspace_archive(
        &self,
        workspace_id: chatmux_common::WorkspaceId,
    ) -> Result<WorkspaceArchive, StorageError> {
        let workspace = self
            .store
            .get_workspace(workspace_id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("workspace {}", workspace_id.0)))?;
        let runs = self.store.list_runs(workspace_id).await?;
        let mut rounds = Vec::new();
        let mut dispatches = Vec::new();
        for run in &runs {
            rounds.extend(self.store.list_rounds(run.id).await?);
            dispatches.extend(self.store.list_dispatches(run.id).await?);
        }
        Ok(WorkspaceArchive {
            schema_version: 1,
            workspace,
            messages: self.store.list_messages(workspace_id).await?,
            runs,
            rounds,
            dispatches,
            delivery_cursors: self.store.list_cursors(workspace_id).await?,
            edge_policies: self.store.list_edge_policies(workspace_id).await?,
            templates: self.store.list_templates(workspace_id).await?,
            export_profiles: self.store.list_export_profiles(workspace_id).await?,
        })
    }

    async fn import_workspace_archive(
        &self,
        body: &str,
    ) -> Result<chatmux_common::WorkspaceId, StorageError> {
        let mut archive: WorkspaceArchive = serde_json::from_str(body).map_err(|error| {
            StorageError::Invariant(format!("invalid workspace archive: {error}"))
        })?;
        if archive.schema_version != 1 {
            return Err(StorageError::Invariant(format!(
                "unsupported workspace archive schema {}",
                archive.schema_version
            )));
        }

        let workspace_id = chatmux_common::WorkspaceId::new();
        let template_ids = archive
            .templates
            .iter()
            .map(|item| (item.id, chatmux_common::TemplateId::new()))
            .collect::<BTreeMap<_, _>>();
        let message_ids = archive
            .messages
            .iter()
            .map(|item| (item.id, chatmux_common::MessageId::new()))
            .collect::<BTreeMap<_, _>>();
        let run_ids = archive
            .runs
            .iter()
            .map(|item| (item.id, chatmux_common::RunId::new()))
            .collect::<BTreeMap<_, _>>();
        let round_ids = archive
            .rounds
            .iter()
            .map(|item| (item.id, chatmux_common::RoundId::new()))
            .collect::<BTreeMap<_, _>>();
        let dispatch_ids = archive
            .dispatches
            .iter()
            .map(|item| (item.id, chatmux_common::DispatchId::new()))
            .collect::<BTreeMap<_, _>>();
        let policy_ids = archive
            .edge_policies
            .iter()
            .map(|item| (item.id, chatmux_common::EdgePolicyId::new()))
            .collect::<BTreeMap<_, _>>();

        archive.workspace.id = workspace_id;
        archive.workspace.name = format!("{} (Imported)", archive.workspace.name);
        archive.workspace.archived = false;
        archive.workspace.created_at = Utc::now();
        archive.workspace.updated_at = Utc::now();
        archive.workspace.default_template_id = archive
            .workspace
            .default_template_id
            .and_then(|id| template_ids.get(&id).copied());
        archive.workspace.active_export_profile_ids.clear();
        self.store.save_workspace(archive.workspace).await?;

        for mut template in archive.templates {
            template.id = template_ids[&template.id];
            template.workspace_id = workspace_id;
            self.store.save_template(template).await?;
        }
        for mut policy in archive.edge_policies {
            policy.id = policy_ids[&policy.id];
            policy.workspace_id = workspace_id;
            policy.template_id = policy
                .template_id
                .and_then(|id| template_ids.get(&id).copied());
            if let CatchUpPolicy::PinnedSummary { summary_message_id } = &mut policy.catch_up_policy
            {
                *summary_message_id =
                    summary_message_id.and_then(|id| message_ids.get(&id).copied());
            }
            if let TruncationPolicy::SwapForSummary {
                summary_message_id, ..
            } = &mut policy.truncation_policy
            {
                *summary_message_id =
                    summary_message_id.and_then(|id| message_ids.get(&id).copied());
            }
            self.store.save_edge_policy(policy).await?;
        }
        for mut run in archive.runs {
            run.id = run_ids[&run.id];
            run.workspace_id = workspace_id;
            for edge in &mut run.graph_snapshot.edges {
                edge.policy_id = edge.policy_id.and_then(|id| policy_ids.get(&id).copied());
            }
            run.status = if matches!(run.status, RunStatus::Running) {
                RunStatus::Paused
            } else {
                run.status
            };
            self.store.save_run(run).await?;
        }
        for mut round in archive.rounds {
            round.id = round_ids[&round.id];
            round.run_id = run_ids.get(&round.run_id).copied().ok_or_else(|| {
                StorageError::Invariant("archive round references a missing run".to_owned())
            })?;
            self.store.save_round(round).await?;
        }
        for mut dispatch in archive.dispatches {
            dispatch.id = dispatch_ids[&dispatch.id];
            dispatch.run_id = run_ids.get(&dispatch.run_id).copied().ok_or_else(|| {
                StorageError::Invariant("archive dispatch references a missing run".to_owned())
            })?;
            dispatch.round_id = dispatch.round_id.and_then(|id| round_ids.get(&id).copied());
            dispatch.source_message_ids = dispatch
                .source_message_ids
                .into_iter()
                .filter_map(|id| message_ids.get(&id).copied())
                .collect();
            dispatch.template_id = dispatch
                .template_id
                .and_then(|id| template_ids.get(&id).copied());
            if dispatch.outcome == chatmux_common::DispatchOutcome::Pending {
                dispatch.outcome = chatmux_common::DispatchOutcome::Skipped;
                dispatch.error_detail =
                    Some("Imported pending dispatch was safely disabled".to_owned());
            }
            self.store.save_dispatch(dispatch).await?;
        }
        for mut message in archive.messages {
            message.id = message_ids[&message.id];
            message.workspace_id = workspace_id;
            message.parent_message_id = message
                .parent_message_id
                .and_then(|id| message_ids.get(&id).copied());
            message.child_message_ids = message
                .child_message_ids
                .into_iter()
                .filter_map(|id| message_ids.get(&id).copied())
                .collect();
            message.dispatch_id = message
                .dispatch_id
                .and_then(|id| dispatch_ids.get(&id).copied());
            message.source_binding_id = None;
            self.store.save_message(message).await?;
        }
        for mut cursor in archive.delivery_cursors {
            cursor.id = chatmux_common::DeliveryCursorId::new();
            cursor.workspace_id = workspace_id;
            cursor.last_delivered_message_id = cursor
                .last_delivered_message_id
                .and_then(|id| message_ids.get(&id).copied());
            cursor.frozen = true;
            self.store.save_cursor(cursor).await?;
        }
        for mut profile in archive.export_profiles {
            profile.id = chatmux_common::ExportProfileId::new();
            profile.workspace_id = workspace_id;
            self.store.save_export_profile(profile).await?;
        }
        Ok(workspace_id)
    }

    pub async fn handle_ui_command(
        &self,
        command: UiCommand,
    ) -> Result<Vec<UiEvent>, StorageError> {
        let should_record = !matches!(
            command,
            UiCommand::RequestDiagnosticsSnapshot { .. } | UiCommand::ClearDiagnostics { .. }
        );
        let command_name = ui_command_name(&command);
        let workspace_id = ui_command_workspace_id(&command);
        let payload = truncate_text(render_json(&command), 8_000);

        let result = self.handle_ui_command_inner(command).await;

        if should_record {
            match &result {
                Ok(events) => {
                    let diagnostic = enrich_diagnostic(
                        diagnostic_event(
                            workspace_id.unwrap_or_default(),
                            DiagnosticScope::Workspace,
                            DiagnosticSource::Ui,
                            DiagnosticLevel::Debug,
                            "ui_command",
                            format!("UI command: {command_name}"),
                            format!("{command_name} succeeded"),
                            format!(
                                "command:\n{payload}\n\nresult:\n{}",
                                summarize_ui_events(events)
                            ),
                        ),
                        &command_name,
                        &payload,
                        Some(events.len().to_string()),
                        None,
                    );

                    let _ = self.store.save_diagnostic(diagnostic.clone()).await;

                    let mut events_with_diagnostic = events.clone();
                    events_with_diagnostic.push(UiEvent::DiagnosticRaised { diagnostic });
                    return Ok(events_with_diagnostic);
                }
                Err(error) => {
                    let diagnostic = enrich_diagnostic(
                        diagnostic_event(
                            workspace_id.unwrap_or_default(),
                            DiagnosticScope::Workspace,
                            DiagnosticSource::Ui,
                            DiagnosticLevel::Warning,
                            "ui_command_failed",
                            format!("UI command failed: {command_name}"),
                            error.to_string(),
                            format!("command:\n{payload}\n\nerror:\n{error}"),
                        ),
                        &command_name,
                        &payload,
                        None,
                        None,
                    );

                    let _ = self.store.save_diagnostic(diagnostic).await;
                }
            }
        }

        result
    }

    async fn handle_ui_command_inner(
        &self,
        command: UiCommand,
    ) -> Result<Vec<UiEvent>, StorageError> {
        match command {
            UiCommand::RequestWorkspaceList => Ok(vec![UiEvent::WorkspaceList {
                workspaces: self.store.list_workspaces().await?,
            }]),
            UiCommand::RequestRunLedger { run_id } => Ok(vec![UiEvent::RunLedgerSnapshot {
                ledger: self.run_ledger(run_id).await?,
            }]),
            UiCommand::CreateWorkspace { name } => {
                let workspace_id = chatmux_common::WorkspaceId::new();
                let templates = builtin_templates(workspace_id);
                let workspace = Workspace {
                    id: workspace_id,
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
                    default_template_id: templates.first().map(|template| template.id),
                    active_export_profile_ids: vec![],
                    tags: vec![],
                    notes: None,
                };
                self.store.save_workspace(workspace.clone()).await?;
                for template in &templates {
                    self.store.save_template(template.clone()).await?;
                }
                let template_id = templates.first().map(|template| template.id);
                for policy in default_workspace_edge_policies(workspace.id, template_id) {
                    self.store.save_edge_policy(policy).await?;
                }
                Ok(vec![
                    UiEvent::WorkspaceList {
                        workspaces: self.store.list_workspaces().await?,
                    },
                    UiEvent::WorkspaceSnapshot {
                        snapshot: self.snapshot_workspace(workspace.id).await?,
                    },
                ])
            }
            UiCommand::DeleteWorkspace { workspace_id } => {
                self.store.delete_workspace(workspace_id).await?;
                Ok(vec![UiEvent::WorkspaceList {
                    workspaces: self.store.list_workspaces().await?,
                }])
            }
            // Distinct from DeleteWorkspace: the workspace survives, so the UI
            // needs a fresh snapshot of the now-empty history rather than just
            // a workspace list.
            UiCommand::ClearWorkspaceData { workspace_id } => {
                self.store.clear_workspace_data(workspace_id).await?;
                Ok(vec![
                    UiEvent::WorkspaceList {
                        workspaces: self.store.list_workspaces().await?,
                    },
                    UiEvent::WorkspaceSnapshot {
                        snapshot: self.snapshot_workspace(workspace_id).await?,
                    },
                ])
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
            UiCommand::DuplicateWorkspace { workspace_id } => {
                let Some(source) = self.store.get_workspace(workspace_id).await? else {
                    return Err(StorageError::NotFound(format!(
                        "workspace {}",
                        workspace_id.0
                    )));
                };
                let existing_names = self
                    .store
                    .list_workspaces()
                    .await?
                    .into_iter()
                    .map(|workspace| workspace.name)
                    .collect::<BTreeSet<_>>();
                let base = format!("{} Copy", source.name);
                let mut name = base.clone();
                let mut suffix = 2u32;
                while existing_names.contains(&name) {
                    name = format!("{base} {suffix}");
                    suffix = suffix.saturating_add(1);
                }
                let new_workspace_id = chatmux_common::WorkspaceId::new();
                let templates = self.store.list_templates(workspace_id).await?;
                let mut template_ids = BTreeMap::new();
                for mut template in templates {
                    let old_id = template.id;
                    template.id = chatmux_common::TemplateId::new();
                    template.workspace_id = new_workspace_id;
                    template_ids.insert(old_id, template.id);
                    self.store.save_template(template).await?;
                }
                for mut policy in self.store.list_edge_policies(workspace_id).await? {
                    policy.id = chatmux_common::EdgePolicyId::new();
                    policy.workspace_id = new_workspace_id;
                    policy.template_id = policy
                        .template_id
                        .and_then(|template_id| template_ids.get(&template_id).copied());
                    self.store.save_edge_policy(policy).await?;
                }
                let mut profile_ids = Vec::new();
                for mut profile in self.store.list_export_profiles(workspace_id).await? {
                    profile.id = chatmux_common::ExportProfileId::new();
                    profile.workspace_id = new_workspace_id;
                    profile_ids.push(profile.id);
                    self.store.save_export_profile(profile).await?;
                }
                let duplicated = Workspace {
                    id: new_workspace_id,
                    name,
                    archived: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    enabled_providers: source.enabled_providers,
                    default_mode: source.default_mode,
                    default_context_strategy: source.default_context_strategy,
                    default_template_id: source
                        .default_template_id
                        .and_then(|template_id| template_ids.get(&template_id).copied()),
                    active_export_profile_ids: profile_ids,
                    tags: source.tags,
                    notes: source.notes,
                };
                self.store.save_workspace(duplicated).await?;
                Ok(vec![
                    UiEvent::WorkspaceList {
                        workspaces: self.store.list_workspaces().await?,
                    },
                    UiEvent::WorkspaceSnapshot {
                        snapshot: self.snapshot_workspace(new_workspace_id).await?,
                    },
                ])
            }
            UiCommand::ExportWorkspaceArchive { workspace_id } => {
                let archive = self.workspace_archive(workspace_id).await?;
                let filename = format!(
                    "{}-chatmux-workspace.json",
                    slugify(&archive.workspace.name)
                );
                let body = serde_json::to_string_pretty(&archive)
                    .map_err(|error| StorageError::Invariant(error.to_string()))?;
                Ok(vec![UiEvent::ExportRendered {
                    format: ExportFormat::Json,
                    mime_type: "application/json".to_owned(),
                    filename,
                    body,
                }])
            }
            UiCommand::ImportWorkspaceArchive { body } => {
                let workspace_id = self.import_workspace_archive(&body).await?;
                Ok(vec![
                    UiEvent::WorkspaceList {
                        workspaces: self.store.list_workspaces().await?,
                    },
                    UiEvent::WorkspaceSnapshot {
                        snapshot: self.snapshot_workspace(workspace_id).await?,
                    },
                ])
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
            UiCommand::RequestDiagnosticsSnapshot { query } => {
                Ok(vec![UiEvent::DiagnosticsSnapshot {
                    snapshot: self.diagnostics_snapshot(query).await?,
                }])
            }
            UiCommand::ClearDiagnostics { query } => Ok(vec![UiEvent::DiagnosticsSnapshot {
                snapshot: self.clear_diagnostics(query).await?,
            }]),
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
            UiCommand::PersistPinnedSummary {
                workspace_id,
                summary_message_id,
                name,
                body,
            } => {
                let name = name.trim();
                let body = body.trim();
                if name.is_empty() || body.is_empty() {
                    return Err(StorageError::Invariant(
                        "pinned summaries require both a name and summary text".to_owned(),
                    ));
                }
                let existing = if let Some(message_id) = summary_message_id {
                    self.store.get_message(message_id).await?
                } else {
                    None
                };
                if existing
                    .as_ref()
                    .is_some_and(|message| message.workspace_id != workspace_id)
                {
                    return Err(StorageError::Invariant(
                        "pinned summary belongs to a different workspace".to_owned(),
                    ));
                }
                let message = Message {
                    id: summary_message_id.unwrap_or_else(chatmux_common::MessageId::new),
                    workspace_id,
                    participant_id: ProviderId::System,
                    role: MessageRole::System,
                    round: None,
                    parent_message_id: None,
                    child_message_ids: Vec::new(),
                    branch_index: None,
                    timestamp: existing
                        .as_ref()
                        .map(|message| message.timestamp)
                        .unwrap_or_else(Utc::now),
                    body_text: body.to_owned(),
                    body_blocks: vec![chatmux_common::Block::Paragraph {
                        text: body.to_owned(),
                    }],
                    source_binding_id: None,
                    dispatch_id: None,
                    raw_response_text: None,
                    network_capture: None,
                    tags: vec!["pinned-summary".to_owned(), format!("summary-name:{name}")],
                    capture_confidence: chatmux_common::CaptureConfidence::Certain,
                };
                self.store.save_message(message).await?;
                Ok(vec![UiEvent::WorkspaceSnapshot {
                    snapshot: self.snapshot_workspace(workspace_id).await?,
                }])
            }
            UiCommand::DeletePinnedSummary {
                workspace_id,
                summary_message_id,
            } => {
                let Some(summary) = self.store.get_message(summary_message_id).await? else {
                    return Ok(vec![]);
                };
                if summary.workspace_id != workspace_id
                    || !summary.tags.iter().any(|tag| tag == "pinned-summary")
                {
                    return Err(StorageError::Invariant(
                        "message is not a pinned summary in this workspace".to_owned(),
                    ));
                }
                for mut policy in self.store.list_edge_policies(workspace_id).await? {
                    let mut changed = false;
                    if matches!(
                        policy.catch_up_policy,
                        CatchUpPolicy::PinnedSummary { summary_message_id: Some(id) } if id == summary_message_id
                    ) {
                        policy.catch_up_policy = CatchUpPolicy::PinnedSummary {
                            summary_message_id: None,
                        };
                        changed = true;
                    }
                    if matches!(
                        policy.truncation_policy,
                        TruncationPolicy::SwapForSummary { summary_message_id: Some(id), .. } if id == summary_message_id
                    ) {
                        let limit = match policy.truncation_policy {
                            TruncationPolicy::SwapForSummary {
                                soft_character_limit,
                                ..
                            } => soft_character_limit,
                            _ => unreachable!(),
                        };
                        policy.truncation_policy = TruncationPolicy::SwapForSummary {
                            soft_character_limit: limit,
                            summary_message_id: None,
                        };
                        changed = true;
                    }
                    if changed {
                        self.store.save_edge_policy(policy).await?;
                    }
                }
                self.store.delete_message(summary_message_id).await?;
                Ok(vec![UiEvent::WorkspaceSnapshot {
                    snapshot: self.snapshot_workspace(workspace_id).await?,
                }])
            }
            UiCommand::ResetDeliveryCursor { cursor_id } => {
                let Some(mut cursor) = self.store.get_cursor(cursor_id).await? else {
                    return Err(StorageError::NotFound(format!(
                        "delivery cursor {}",
                        cursor_id.0
                    )));
                };
                cursor.last_delivered_message_id = None;
                cursor.last_delivered_at = None;
                let workspace_id = cursor.workspace_id;
                self.store.save_cursor(cursor).await?;
                Ok(vec![UiEvent::WorkspaceSnapshot {
                    snapshot: self.snapshot_workspace(workspace_id).await?,
                }])
            }
            UiCommand::SetDeliveryCursorFrozen { cursor_id, frozen } => {
                let Some(mut cursor) = self.store.get_cursor(cursor_id).await? else {
                    return Err(StorageError::NotFound(format!(
                        "delivery cursor {}",
                        cursor_id.0
                    )));
                };
                cursor.frozen = frozen;
                let workspace_id = cursor.workspace_id;
                self.store.save_cursor(cursor).await?;
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
                let Some(workspace) = self.store.get_workspace(workspace_id).await? else {
                    return Err(StorageError::NotFound(format!(
                        "workspace {}",
                        workspace_id.0
                    )));
                };
                let timing_policy = chatmux_common::TimingPolicy {
                    max_rounds: Some(20),
                    ..chatmux_common::TimingPolicy::default()
                };
                self.start_configured_run(
                    &workspace,
                    RunConfiguration {
                        mode,
                        participants: workspace.enabled_providers.clone(),
                        timing_policy,
                        ..RunConfiguration::default()
                    },
                )
                .await
            }
            UiCommand::StartConfiguredRun {
                workspace_id,
                configuration,
            } => {
                let Some(workspace) = self.store.get_workspace(workspace_id).await? else {
                    return Err(StorageError::NotFound(format!(
                        "workspace {}",
                        workspace_id.0
                    )));
                };
                self.start_configured_run(&workspace, configuration).await
            }
            UiCommand::PauseRun { run_id }
            | UiCommand::StopRun { run_id }
            | UiCommand::AbortRun { run_id } => {
                let Some(mut run) = self.store.get_run(run_id).await? else {
                    return Ok(vec![]);
                };
                run.status = match command {
                    UiCommand::PauseRun { .. } => RunStatus::Paused,
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
            UiCommand::ResumeRun { run_id } => self.resume_run_execution(run_id).await,
            UiCommand::PreviewNextRound { run_id } => self.preview_next_round(run_id).await,
            UiCommand::ResumeRunWithOverrides {
                run_id,
                payload_overrides,
                skipped_targets,
                injected_user_message,
            } => {
                self.resume_run_with_overrides(
                    run_id,
                    payload_overrides,
                    skipped_targets,
                    injected_user_message,
                )
                .await
            }
            UiCommand::StepRun { run_id } => self.step_run_execution(run_id).await,
            UiCommand::PreviewManualMessage {
                workspace_id,
                targets,
                text,
                selected_message_ids,
                pinned_note,
                target_notes,
                include_target_prior_turns,
                parent_message_id,
            } => {
                self.preview_manual_message(
                    workspace_id,
                    targets,
                    text,
                    selected_message_ids,
                    pinned_note,
                    target_notes,
                    include_target_prior_turns,
                    parent_message_id,
                )
                .await
            }
            UiCommand::SendManualMessage {
                workspace_id,
                targets,
                text,
                approval_mode,
                selected_message_ids,
                pinned_note,
                target_notes,
                include_target_prior_turns,
                payload_overrides,
                parent_message_id,
            } => {
                if approval_mode == chatmux_common::ApprovalMode::AutoSend {
                    self.ensure_automation_enabled().await?;
                }
                let preparation = self
                    .prepare_manual_message(
                        workspace_id,
                        targets,
                        text,
                        selected_message_ids,
                        pinned_note,
                        target_notes,
                        include_target_prior_turns,
                        parent_message_id,
                    )
                    .await?;
                for template in preparation.templates_to_persist {
                    self.store.save_template(template).await?;
                }

                let mut events = Vec::new();
                let participant_set = preparation
                    .packages
                    .iter()
                    .map(|package| package.target_participant_id)
                    .collect::<BTreeSet<_>>();
                let run = Run {
                    id: chatmux_common::RunId::new(),
                    workspace_id,
                    mode: preparation.workspace_mode,
                    graph_snapshot: compile_graph(preparation.workspace_mode, &participant_set),
                    participant_set,
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
                if let Some(mut parent) = preparation.parent_message
                    && !parent
                        .child_message_ids
                        .contains(&preparation.user_message.id)
                {
                    parent.child_message_ids.push(preparation.user_message.id);
                    self.store.save_message(parent.clone()).await?;
                    events.push(UiEvent::MessageCaptured { message: parent });
                }
                self.store
                    .save_message(preparation.user_message.clone())
                    .await?;
                events.push(UiEvent::MessageCaptured {
                    message: preparation.user_message,
                });

                for package in preparation.packages {
                    let rendered_payload = payload_overrides
                        .get(&package.target_participant_id)
                        .cloned()
                        .unwrap_or(package.rendered_payload);
                    let dispatch = Dispatch {
                        id: chatmux_common::DispatchId::new(),
                        run_id: run.id,
                        round_id: Some(round.id),
                        round_number: round.round_number,
                        target_participant_id: package.target_participant_id,
                        source_message_ids: package.source_message_ids,
                        template_id: package.template_id,
                        rendered_payload,
                        sent_at: None,
                        captured_at: None,
                        outcome: prepared_outcome(approval_mode),
                        error_detail: None,
                        retry_count: 0,
                    };
                    self.store.save_dispatch(dispatch.clone()).await?;
                    events.push(UiEvent::DispatchUpdated { dispatch });
                }
                Ok(events)
            }
            UiCommand::AcknowledgeDispatchDelivered { dispatch_id } => {
                self.acknowledge_dispatch_delivered(dispatch_id).await
            }
            UiCommand::AcknowledgeDispatchFailed {
                dispatch_id,
                detail,
            } => self.acknowledge_dispatch_failed(dispatch_id, detail).await,
            UiCommand::AcknowledgeDispatchCaptured {
                dispatch_id,
                messages,
            } => {
                self.acknowledge_dispatch_captured(dispatch_id, messages)
                    .await
            }
            UiCommand::SyncProviderConversation {
                workspace_id,
                provider,
            }
            | UiCommand::RequestProviderTabCandidates {
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
            UiCommand::BindProviderTab {
                workspace_id,
                provider,
                tab_id,
                window_id,
                origin,
                tab_title,
                tab_url,
                conversation_id,
                conversation_title,
                conversation_url,
                pin,
            } => {
                let binding = self
                    .upsert_binding_for_provider(workspace_id, provider, |binding| {
                        binding.tab_id = Some(tab_id);
                        binding.window_id = window_id;
                        binding.origin = origin.clone();
                        binding.tab_title = tab_title.clone();
                        binding.tab_url = tab_url.clone();
                        binding.pinned = pin;
                        let model_label = binding
                            .bound_conversation_ref
                            .as_ref()
                            .and_then(|item| item.model_label.clone())
                            .or_else(|| {
                                binding
                                    .conversation_ref
                                    .as_ref()
                                    .and_then(|item| item.model_label.clone())
                            });
                        binding.bound_conversation_ref = if conversation_id.is_some() {
                            Some(chatmux_common::ConversationRef {
                                conversation_id: conversation_id.clone(),
                                title: conversation_title.clone(),
                                url: conversation_url.clone().or_else(|| tab_url.clone()),
                                model_label,
                            })
                        } else {
                            None
                        };
                        binding.conversation_ref = None;
                        binding.provider_control = None;
                        binding.health_state = ProviderHealth::Ready;
                        binding.stale = binding.has_bound_target();
                    })
                    .await?;
                Ok(vec![
                    UiEvent::WorkspaceSnapshot {
                        snapshot: self.snapshot_workspace(workspace_id).await?,
                    },
                    UiEvent::ProviderHealthChanged {
                        workspace_id,
                        provider,
                        health: ProviderHealth::Ready,
                        blocking_state: None,
                    },
                    UiEvent::ProviderControlUpdated {
                        workspace_id,
                        snapshot: provider_control_snapshot_from_binding(binding),
                    },
                ])
            }
            UiCommand::PersistProviderDefaults { provider, defaults } => {
                let mut settings = self.store.load_settings().await?;
                settings
                    .provider_defaults
                    .insert(provider, defaults.clone());
                self.store.save_settings(settings).await?;
                Ok(vec![UiEvent::ProviderDefaultsUpdated {
                    provider,
                    defaults,
                }])
            }
            UiCommand::OpenProviderTab { .. } => Ok(vec![]),
            UiCommand::DeleteTemplate { template_id } => {
                let mut workspace_id = None;
                for workspace in self.store.list_workspaces().await? {
                    if self
                        .store
                        .list_templates(workspace.id)
                        .await?
                        .iter()
                        .any(|template| template.id == template_id)
                    {
                        workspace_id = Some(workspace.id);
                        break;
                    }
                }
                self.store.delete_template(template_id).await?;
                if let Some(workspace_id) = workspace_id {
                    Ok(vec![UiEvent::WorkspaceSnapshot {
                        snapshot: self.snapshot_workspace(workspace_id).await?,
                    }])
                } else {
                    Ok(vec![UiEvent::WorkspaceList {
                        workspaces: self.store.list_workspaces().await?,
                    }])
                }
            }
            UiCommand::ExportSelection {
                workspace_id,
                format,
                layout,
                profile_id,
            } => {
                self.render_export_request(ExportRequest {
                    workspace_id,
                    scope: ExportScopePreset::EntireWorkspace,
                    format,
                    layout,
                    profile_id,
                    participants: BTreeSet::new(),
                    roles: BTreeSet::new(),
                    selected_message_ids: BTreeSet::new(),
                    selected_rounds: BTreeSet::new(),
                    run_id: None,
                    time_range_iso: None,
                    delivery_outcomes: Vec::new(),
                    tags: Vec::new(),
                    query: None,
                    invert_selection: false,
                    include_flags: export_engine::default_metadata_flags(),
                    include_front_matter: true,
                    filename_template: None,
                })
                .await
            }
            UiCommand::ExportConfigured { request } => self.render_export_request(request).await,
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
                    raw_response_text: message
                        .as_ref()
                        .and_then(|item| item.raw_response_text.clone()),
                    network_capture: message
                        .as_ref()
                        .and_then(|item| item.network_capture.clone()),
                    message,
                    dispatch,
                }])
            }
            UiCommand::SetKillSwitch { active } => {
                let mut settings = self.store.load_settings().await?;
                settings.kill_switch_active = active;
                self.store.save_settings(settings).await?;
                let mut events = vec![UiEvent::KillSwitchChanged { active }];
                if active {
                    for workspace in self.store.list_workspaces().await? {
                        for mut run in self.store.list_runs(workspace.id).await? {
                            if run.status != RunStatus::Running {
                                continue;
                            }
                            run.status = RunStatus::Paused;
                            self.store.save_run(run.clone()).await?;
                            for dispatch in self.store.list_dispatches(run.id).await? {
                                if dispatch.outcome == chatmux_common::DispatchOutcome::Pending {
                                    let failed = mark_failed(
                                        dispatch,
                                        "global kill switch halted the pending dispatch",
                                    )?;
                                    self.store.save_dispatch(failed.clone()).await?;
                                    events.push(UiEvent::DispatchUpdated { dispatch: failed });
                                }
                            }
                            events.push(UiEvent::RunUpdated {
                                run: run.clone(),
                                rounds: self.store.list_rounds(run.id).await?,
                            });
                        }
                    }
                }
                Ok(events)
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

    #[allow(clippy::too_many_arguments)]
    async fn preview_manual_message(
        &self,
        workspace_id: chatmux_common::WorkspaceId,
        targets: Vec<ProviderId>,
        text: String,
        selected_message_ids: BTreeSet<chatmux_common::MessageId>,
        pinned_note: Option<String>,
        target_notes: BTreeMap<ProviderId, String>,
        include_target_prior_turns: bool,
        parent_message_id: Option<chatmux_common::MessageId>,
    ) -> Result<Vec<UiEvent>, StorageError> {
        let preparation = self
            .prepare_manual_message(
                workspace_id,
                targets,
                text,
                selected_message_ids,
                pinned_note,
                target_notes,
                include_target_prior_turns,
                parent_message_id,
            )
            .await?;
        Ok(vec![UiEvent::ManualMessagePreview {
            packages: preparation.packages,
        }])
    }

    #[allow(clippy::too_many_arguments)]
    async fn prepare_manual_message(
        &self,
        workspace_id: chatmux_common::WorkspaceId,
        targets: Vec<ProviderId>,
        text: String,
        selected_message_ids: BTreeSet<chatmux_common::MessageId>,
        pinned_note: Option<String>,
        target_notes: BTreeMap<ProviderId, String>,
        include_target_prior_turns: bool,
        parent_message_id: Option<chatmux_common::MessageId>,
    ) -> Result<ManualMessagePreparation, StorageError> {
        let Some(workspace) = self.store.get_workspace(workspace_id).await? else {
            return Err(StorageError::NotFound(format!(
                "workspace {}",
                workspace_id.0
            )));
        };
        if targets.is_empty() {
            return Err(StorageError::Invariant(
                "manual package preparation failed: no target provider was selected; select at least one ready provider and retry"
                    .to_owned(),
            ));
        }
        if let Some(target) = targets.iter().find(|target| {
            matches!(target, ProviderId::User | ProviderId::System)
                || !workspace.enabled_providers.contains(target)
        }) {
            return Err(StorageError::Invariant(format!(
                "manual package preparation failed: {} is not an enabled provider target; enable and bind the provider, then retry",
                target.display_name()
            )));
        }
        let existing_messages = self.store.list_messages(workspace_id).await?;
        let parent_message = if let Some(parent_id) = parent_message_id {
            let Some(parent) = self.store.get_message(parent_id).await? else {
                return Err(StorageError::NotFound(format!(
                    "parent message {}",
                    parent_id.0
                )));
            };
            if parent.workspace_id != workspace_id {
                return Err(StorageError::Invariant(
                    "parent message belongs to another workspace".to_owned(),
                ));
            }
            Some(parent)
        } else {
            existing_messages.last().cloned()
        };
        let mut templates = self.store.list_templates(workspace_id).await?;
        let templates_to_persist = if templates.is_empty() {
            templates = builtin_templates(workspace_id);
            templates.clone()
        } else {
            Vec::new()
        };
        let template = workspace
            .default_template_id
            .and_then(|template_id| {
                templates
                    .iter()
                    .find(|template| template.id == template_id)
            })
            .or_else(|| templates.first())
            .ok_or_else(|| {
                StorageError::Invariant(
                    "manual preview failed: no prompt template is available; create or restore a template and retry"
                        .to_owned(),
                )
            })?;
        let user_message = Message {
            id: chatmux_common::MessageId::new(),
            workspace_id,
            participant_id: ProviderId::User,
            role: MessageRole::User,
            round: Some(1),
            parent_message_id: parent_message.as_ref().map(|message| message.id),
            child_message_ids: Vec::new(),
            branch_index: parent_message
                .as_ref()
                .map(|message| message.child_message_ids.len() as u32 + 1),
            timestamp: Utc::now(),
            body_text: text.clone(),
            body_blocks: vec![chatmux_common::Block::Paragraph { text }],
            source_binding_id: None,
            dispatch_id: None,
            raw_response_text: None,
            network_capture: None,
            tags: Vec::new(),
            capture_confidence: chatmux_common::CaptureConfidence::Certain,
        };
        let known_message_ids = existing_messages
            .iter()
            .map(|message| message.id)
            .collect::<BTreeSet<_>>();
        if let Some(missing_id) = selected_message_ids
            .iter()
            .find(|message_id| !known_message_ids.contains(message_id))
        {
            return Err(StorageError::NotFound(format!(
                "selected message {}",
                missing_id.0
            )));
        }
        let selected_context = existing_messages
            .into_iter()
            .filter(|message| selected_message_ids.contains(&message.id))
            .collect::<Vec<_>>();
        // O(n) where n = selected targets (expected: 1–4).
        let packages = targets
            .into_iter()
            .map(|target| {
                let mut messages = selected_context
                    .iter()
                    .filter(|message| {
                        include_target_prior_turns
                            || message.participant_id != target
                            || message.role != MessageRole::Assistant
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                let source_blocks = package_source_blocks(&messages);
                messages.push(user_message.clone());
                let note = combine_manual_notes(
                    pinned_note.as_deref(),
                    target_notes.get(&target).map(String::as_str),
                );
                let rendered = render_template(template, target, &messages, note.as_deref());
                NextRoundPackage {
                    target_participant_id: target,
                    round_number: 1,
                    source_message_ids: rendered.source_message_ids,
                    source_blocks,
                    template_id: Some(template.id),
                    character_count: rendered.body.chars().count(),
                    rendered_payload: rendered.body,
                }
            })
            .collect();
        Ok(ManualMessagePreparation {
            workspace_mode: workspace.default_mode,
            parent_message,
            user_message,
            packages,
            templates_to_persist,
        })
    }

    async fn diagnostics_snapshot(
        &self,
        query: DiagnosticsQuery,
    ) -> Result<DiagnosticsSnapshot, StorageError> {
        let mut events = if let Some(workspace_id) = query.workspace_id {
            self.store.list_diagnostics(workspace_id).await?
        } else {
            let mut all_events = Vec::new();
            for workspace in self.store.list_workspaces().await? {
                all_events.extend(self.store.list_diagnostics(workspace.id).await?);
            }
            all_events
        };

        if !query.levels.is_empty() {
            events.retain(|event| query.levels.contains(&event.level));
        }
        if !query.sources.is_empty() {
            events.retain(|event| query.sources.contains(&event.source));
        }
        if !query.providers.is_empty() {
            events.retain(|event| {
                event
                    .provider_id
                    .map(|provider| query.providers.contains(&provider))
                    .unwrap_or(false)
            });
        }

        events.sort_by_key(|event| event.timestamp);
        events.reverse();

        let total_available = events.len() as u32;
        if let Some(limit) = query.limit {
            events.truncate(limit as usize);
        }

        let summary = summarize_diagnostics(query.workspace_id, &events);
        let events = events.into_iter().map(sanitize_diagnostic_event).collect();

        Ok(DiagnosticsSnapshot {
            summary,
            total_available,
            events,
            queued_count: 0,
            retention_event_cap: Some(2500),
            retention_artifact_bytes_cap: None,
        })
    }

    async fn clear_diagnostics(
        &self,
        mut query: DiagnosticsQuery,
    ) -> Result<DiagnosticsSnapshot, StorageError> {
        query.limit = None;
        let snapshot = self.diagnostics_snapshot(query.clone()).await?;
        for event in snapshot.events {
            self.store.delete_diagnostic(event.id).await?;
        }
        self.diagnostics_snapshot(query).await
    }

    pub async fn ingest_adapter_event(
        &self,
        workspace_id: chatmux_common::WorkspaceId,
        event: AdapterToBackground,
    ) -> Result<Vec<UiEvent>, StorageError> {
        let event_name = adapter_event_name(&event);
        let payload = truncate_text(render_json(&event), 8_000);
        let result: Result<Vec<UiEvent>, StorageError> = match event {
            AdapterToBackground::HealthReport { provider, health } => {
                self.persist_provider_health(workspace_id, provider, health)
                    .await?;
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
                self.persist_provider_health(workspace_id, provider, ProviderHealth::Blocked)
                    .await?;
                let diagnostic = enrich_diagnostic(
                    diagnostic_event(
                        workspace_id,
                        DiagnosticScope::Workspace,
                        DiagnosticSource::Adapter,
                        DiagnosticLevel::Warning,
                        "blocking_state_detected",
                        format!("Blocking state detected for {provider:?}"),
                        format!("{provider:?} is blocked"),
                        format!("{provider:?}: {blocking_state:?}"),
                    ),
                    "blocking_state_detected",
                    &render_json(&blocking_state),
                    None,
                    Some(provider),
                );
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
            AdapterToBackground::MessagesCaptured { provider, messages } => {
                if messages
                    .iter()
                    .any(|message| !message_matches_provider_envelope(message, provider))
                {
                    return Err(StorageError::Invariant(
                        "adapter capture failed: message identity does not match the envelope provider; discard the mismatched capture and retry"
                            .to_owned(),
                    ));
                }
                let mut events = Vec::new();
                for mut message in messages {
                    message.workspace_id = workspace_id;
                    self.store.save_message(message.clone()).await?;
                    events.push(UiEvent::MessageCaptured { message });
                }
                Ok(events)
            }
            AdapterToBackground::StructuralProbePassed { provider } => {
                self.persist_provider_health(workspace_id, provider, ProviderHealth::Ready)
                    .await?;
                Ok(vec![UiEvent::ProviderHealthChanged {
                    workspace_id,
                    provider,
                    health: ProviderHealth::Ready,
                    blocking_state: None,
                }])
            }
            AdapterToBackground::StructuralProbeFailed { provider, detail } => {
                self.persist_provider_health(workspace_id, provider, ProviderHealth::DomMismatch)
                    .await?;
                let diagnostic = enrich_diagnostic(
                    diagnostic_event(
                        workspace_id,
                        DiagnosticScope::Workspace,
                        DiagnosticSource::Adapter,
                        DiagnosticLevel::Critical,
                        "dom_mismatch",
                        format!("Structural probe failed for {provider:?}"),
                        "DOM probe did not match the expected structure".to_owned(),
                        detail,
                    ),
                    "structural_probe_failed",
                    &render_json(&provider),
                    None,
                    Some(provider),
                );
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
                        if let Some(current_ref) = conversation_ref.as_ref() {
                            if !binding.has_bound_target() && current_ref.has_stable_identity() {
                                binding.bound_conversation_ref = Some(current_ref.clone());
                            }
                            let provider_control = binding
                                .provider_control
                                .get_or_insert_with(ProviderControlState::default);
                            provider_control.conversation_id = current_ref.conversation_id.clone();
                            provider_control.conversation_title = current_ref.title.clone();
                            provider_control.model_label = current_ref.model_label.clone();
                        }
                        binding.tab_url = conversation_ref
                            .as_ref()
                            .and_then(|item| item.url.clone())
                            .or_else(|| binding.tab_url.clone());
                        binding.stale = !binding.matches_bound_target();
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
                        let current_ref = chatmux_common::ConversationRef {
                            conversation_id: snapshot.state.conversation_id.clone(),
                            title: snapshot.state.conversation_title.clone(),
                            url: binding
                                .conversation_ref
                                .as_ref()
                                .and_then(|item| item.url.clone())
                                .or_else(|| binding.tab_url.clone()),
                            model_label: snapshot.state.model_label.clone(),
                        };
                        binding.conversation_ref = Some(current_ref.clone());
                        if !binding.has_bound_target() && current_ref.has_stable_identity() {
                            binding.bound_conversation_ref = Some(current_ref);
                        }
                        binding.stale = !binding.matches_bound_target();
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
                    UiEvent::ProviderControlUpdated {
                        workspace_id,
                        snapshot,
                    },
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
                self.persist_provider_health(workspace_id, provider, ProviderHealth::SendFailed)
                    .await?;
                let diagnostic = enrich_diagnostic(
                    diagnostic_event(
                        workspace_id,
                        DiagnosticScope::Workspace,
                        DiagnosticSource::Adapter,
                        level,
                        "adapter_command_failed",
                        format!("Adapter command failed for {provider:?}"),
                        detail.clone(),
                        detail.clone(),
                    ),
                    "command_failed",
                    &render_json(&provider),
                    None,
                    Some(provider),
                );
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
        };

        match &result {
            Ok(events) => {
                let provider_id = adapter_event_provider(events);
                let diagnostic = enrich_diagnostic(
                    diagnostic_event(
                        workspace_id,
                        DiagnosticScope::Workspace,
                        DiagnosticSource::Adapter,
                        DiagnosticLevel::Debug,
                        "adapter_event",
                        format!("Adapter event: {event_name}"),
                        format!("{event_name} received"),
                        format!(
                            "event:\n{payload}\n\nresult:\n{}",
                            summarize_ui_events(events)
                        ),
                    ),
                    &event_name,
                    &payload,
                    Some(events.len().to_string()),
                    provider_id,
                );
                let _ = self.store.save_diagnostic(diagnostic.clone()).await;
                let mut events_with_diagnostic = events.clone();
                events_with_diagnostic.push(UiEvent::DiagnosticRaised { diagnostic });
                Ok(events_with_diagnostic)
            }
            Err(error) => {
                let diagnostic = enrich_diagnostic(
                    diagnostic_event(
                        workspace_id,
                        DiagnosticScope::Workspace,
                        DiagnosticSource::Adapter,
                        DiagnosticLevel::Warning,
                        "adapter_event_failed",
                        format!("Adapter event failed: {event_name}"),
                        error.to_string(),
                        format!("event:\n{payload}\n\nerror:\n{error}"),
                    ),
                    &event_name,
                    &payload,
                    None,
                    None,
                );
                let _ = self.store.save_diagnostic(diagnostic).await;
                result
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
        self.ensure_automation_enabled().await?;
        let cursors = self.store.list_cursors(run.workspace_id).await?;
        let rounds = self.store.list_rounds(run.id).await?;
        let Some(current_round) = rounds.iter().max_by_key(|round| round.round_number) else {
            return Err(StorageError::Invariant(
                "dispatch synthesis requires a persisted round".to_owned(),
            ));
        };
        let round_number = current_round.round_number;
        let mut prepared_by_key = self
            .store
            .list_dispatches(run.id)
            .await?
            .into_iter()
            .map(|dispatch| {
                (
                    (
                        dispatch.round_number,
                        dispatch.target_participant_id,
                        dispatch.source_message_ids.clone(),
                        dispatch.template_id,
                    ),
                    dispatch,
                )
            })
            .collect::<BTreeMap<_, _>>();

        let mut policies_by_target = BTreeMap::<ProviderId, Vec<&EdgePolicy>>::new();
        for edge in &run.graph_snapshot.edges {
            if let Some(policy) = policies.iter().find(|policy| {
                policy.source_participant_id == edge.source
                    && policy.target_participant_id == edge.target
                    && policy.enabled
            }) {
                policies_by_target
                    .entry(edge.target)
                    .or_default()
                    .push(policy);
            }
        }

        let mut dispatches = Vec::new();
        for (target, target_policies) in policies_by_target {
            let mut selected_ids = BTreeSet::new();
            for policy in &target_policies {
                let cursor = cursors.iter().find(|cursor| {
                    cursor.source_participant_id == policy.source_participant_id
                        && cursor.target_participant_id == policy.target_participant_id
                });
                selected_ids.extend(
                    select_messages_for_edge(workspace_messages, policy, cursor)
                        .into_iter()
                        .map(|message| message.id),
                );
            }
            let selected_messages = workspace_messages
                .iter()
                .filter(|message| selected_ids.contains(&message.id))
                .cloned()
                .collect::<Vec<_>>();
            if selected_messages.is_empty() {
                continue;
            }
            let selected_policy = target_policies
                .iter()
                .max_by_key(|policy| policy.priority)
                .copied()
                .ok_or_else(|| {
                    StorageError::Invariant(
                        "dispatch synthesis lost the target edge policy".to_owned(),
                    )
                })?;
            let template = templates
                .iter()
                .find(|template| Some(template.id) == selected_policy.template_id)
                .or_else(|| templates.first())
                .ok_or_else(|| {
                    StorageError::Invariant(
                        "dispatch synthesis requires at least one packaging template".to_owned(),
                    )
                })?;
            let rendered = render_template(template, target, &selected_messages, None);
            let dispatch_key = (
                round_number,
                target,
                rendered.source_message_ids.clone(),
                Some(template.id),
            );
            if let Some(existing) = prepared_by_key.get(&dispatch_key) {
                dispatches.push(existing.clone());
                continue;
            }
            let dispatch = Dispatch {
                id: chatmux_common::DispatchId::new(),
                run_id: run.id,
                round_id: Some(current_round.id),
                round_number,
                target_participant_id: target,
                source_message_ids: rendered.source_message_ids,
                template_id: Some(template.id),
                rendered_payload: rendered.body,
                sent_at: None,
                captured_at: None,
                outcome: prepared_outcome(selected_policy.approval_mode),
                error_detail: None,
                retry_count: 0,
            };
            self.store.save_dispatch(dispatch.clone()).await?;
            prepared_by_key.insert(dispatch_key, dispatch.clone());
            dispatches.push(dispatch);
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

    /// Pause persisted active runs on worker/browser restart without resending pending work.
    pub async fn recover_after_restart(&self) -> Result<u32, StorageError> {
        let mut settings = self.store.load_settings().await?;
        for workspace in self.store.list_workspaces().await? {
            for mut run in self.store.list_runs(workspace.id).await? {
                if run.status != RunStatus::Running {
                    continue;
                }
                run.status = RunStatus::Paused;
                self.store.save_run(run.clone()).await?;
                let last_dispatch_id = self
                    .store
                    .list_dispatches(run.id)
                    .await?
                    .into_iter()
                    .max_by_key(|dispatch| dispatch.round_number)
                    .map(|dispatch| dispatch.id);
                if let Some(marker) = settings
                    .resume_markers
                    .iter_mut()
                    .find(|marker| marker.paused_run_id == Some(run.id))
                {
                    marker.last_seen_dispatch_id = last_dispatch_id;
                } else {
                    settings.resume_markers.push(crate::storage::ResumeMarker {
                        workspace_id: workspace.id,
                        paused_run_id: Some(run.id),
                        last_seen_dispatch_id: last_dispatch_id,
                    });
                }
            }
        }
        let marker_count = settings.resume_markers.len() as u32;
        self.store.save_settings(settings).await?;
        Ok(marker_count)
    }

    async fn ensure_automation_enabled(&self) -> Result<(), StorageError> {
        if self.store.load_settings().await?.kill_switch_active {
            return Err(StorageError::Invariant(
                "dispatch preparation failed: kill switch is active; disable it before sending"
                    .to_owned(),
            ));
        }
        Ok(())
    }

    async fn start_configured_run(
        &self,
        workspace: &Workspace,
        mut configuration: RunConfiguration,
    ) -> Result<Vec<UiEvent>, StorageError> {
        self.ensure_automation_enabled().await?;
        configuration.participants.retain(|provider| {
            workspace.enabled_providers.contains(provider)
                && !matches!(provider, ProviderId::User | ProviderId::System)
        });
        if configuration.participants.is_empty() {
            return Err(StorageError::Invariant(
                "run configuration requires at least one enabled provider".to_owned(),
            ));
        }
        if configuration
            .moderator
            .is_some_and(|provider| !configuration.participants.contains(&provider))
        {
            return Err(StorageError::Invariant(
                "run moderator must belong to the participant set".to_owned(),
            ));
        }
        if configuration
            .relay_order
            .iter()
            .any(|provider| !configuration.participants.contains(provider))
            || configuration
                .relay_order
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != configuration.relay_order.len()
        {
            return Err(StorageError::Invariant(
                "relay order must contain unique members of the participant set".to_owned(),
            ));
        }
        if let BarrierPolicy::Quorum { providers } = &configuration.barrier_policy
            && (providers.is_empty() || !providers.is_subset(&configuration.participants))
        {
            return Err(StorageError::Invariant(
                "quorum providers must be a non-empty subset of run participants".to_owned(),
            ));
        }
        if configuration.timing_policy.max_concurrent_sends == 0
            || configuration.timing_policy.jitter_percent > 100
            || configuration.timing_policy.max_rounds == Some(0)
        {
            return Err(StorageError::Invariant(
                "run timing requires positive concurrency/round limits and jitter no greater than 100 percent"
                    .to_owned(),
            ));
        }
        if configuration.require_review_between_rounds
            || configuration.mode == OrchestrationMode::ModeratedAutonomous
        {
            configuration.stop_policy.require_approval_between_rounds = true;
        }

        let run = Run {
            id: chatmux_common::RunId::new(),
            workspace_id: workspace.id,
            mode: configuration.mode,
            graph_snapshot: compile_configured_graph(&configuration),
            participant_set: configuration.participants,
            barrier_policy: configuration.barrier_policy,
            timing_policy: configuration.timing_policy,
            stop_policy: configuration.stop_policy,
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

        let mut templates = self.store.list_templates(workspace.id).await?;
        if templates.is_empty() {
            templates = builtin_templates(workspace.id);
            for template in &templates {
                self.store.save_template(template.clone()).await?;
            }
        }
        let mut policies = self.store.list_edge_policies(workspace.id).await?;
        for edge in &run.graph_snapshot.edges {
            if !policies.iter().any(|policy| {
                policy.source_participant_id == edge.source
                    && policy.target_participant_id == edge.target
            }) {
                let policy = default_edge_policy(
                    workspace.id,
                    edge.source,
                    edge.target,
                    templates.first().map(|template| template.id),
                );
                self.store.save_edge_policy(policy.clone()).await?;
                policies.push(policy);
            }
        }
        let messages = self.store.list_messages(workspace.id).await?;
        let dispatches = self
            .synthesize_dispatches(&run, &policies, &messages, &templates)
            .await?;
        let mut events = vec![UiEvent::RunUpdated {
            run,
            rounds: vec![round],
        }];
        events.extend(
            dispatches
                .into_iter()
                .map(|dispatch| UiEvent::DispatchUpdated { dispatch }),
        );
        Ok(events)
    }

    async fn step_run_execution(
        &self,
        run_id: chatmux_common::RunId,
    ) -> Result<Vec<UiEvent>, StorageError> {
        self.ensure_automation_enabled().await?;
        let Some(mut run) = self.store.get_run(run_id).await? else {
            return Err(StorageError::NotFound(format!("run {}", run_id.0)));
        };
        if matches!(run.status, RunStatus::Completed | RunStatus::Aborted) {
            return Err(StorageError::Invariant(
                "completed or aborted runs cannot be stepped".to_owned(),
            ));
        }
        let mut rounds = self.store.list_rounds(run_id).await?;
        if let Some(current) = rounds.iter_mut().max_by_key(|round| round.round_number)
            && current.status == RoundStatus::Running
        {
            current.status = RoundStatus::Completed;
            current.completed_at = Some(Utc::now());
            self.store.save_round(current.clone()).await?;
        }
        run.status = RunStatus::Paused;
        self.store.save_run(run).await?;
        self.resume_run_execution(run_id).await
    }

    async fn resume_run_execution(
        &self,
        run_id: chatmux_common::RunId,
    ) -> Result<Vec<UiEvent>, StorageError> {
        self.ensure_automation_enabled().await?;
        let Some(mut run) = self.store.get_run(run_id).await? else {
            return Err(StorageError::NotFound(format!("run {}", run_id.0)));
        };
        if matches!(run.status, RunStatus::Completed | RunStatus::Aborted) {
            return Err(StorageError::Invariant(
                "completed or aborted runs cannot be resumed; start a new run in the workspace"
                    .to_owned(),
            ));
        }
        run.status = RunStatus::Running;
        self.store.save_run(run.clone()).await?;
        let mut rounds = self.store.list_rounds(run_id).await?;
        let mut current = rounds
            .iter()
            .max_by_key(|round| round.round_number)
            .cloned();
        if current.as_ref().is_none_or(|round| {
            matches!(
                round.status,
                RoundStatus::Completed | RoundStatus::Failed | RoundStatus::TimedOut
            )
        }) {
            let next = Round {
                id: chatmux_common::RoundId::new(),
                run_id,
                round_number: current
                    .as_ref()
                    .map_or(1, |round| round.round_number.saturating_add(1)),
                started_at: Some(Utc::now()),
                completed_at: None,
                status: RoundStatus::Running,
            };
            self.store.save_round(next.clone()).await?;
            rounds.push(next.clone());
            current = Some(next);
        }
        let current = current.ok_or_else(|| {
            StorageError::Invariant("run resume could not create a current round".to_owned())
        })?;
        let existing_pending = self
            .store
            .list_dispatches(run_id)
            .await?
            .into_iter()
            .filter(|dispatch| {
                dispatch.round_number == current.round_number
                    && dispatch.outcome == chatmux_common::DispatchOutcome::Pending
            })
            .collect::<Vec<_>>();
        let dispatches = if existing_pending.is_empty() {
            let policies = self.store.list_edge_policies(run.workspace_id).await?;
            let messages = self.store.list_messages(run.workspace_id).await?;
            let templates = self.store.list_templates(run.workspace_id).await?;
            self.synthesize_dispatches(&run, &policies, &messages, &templates)
                .await?
        } else {
            existing_pending
        };
        let mut events = vec![UiEvent::RunUpdated { run, rounds }];
        events.extend(
            dispatches
                .into_iter()
                .map(|dispatch| UiEvent::DispatchUpdated { dispatch }),
        );
        Ok(events)
    }

    async fn preview_next_round(
        &self,
        run_id: chatmux_common::RunId,
    ) -> Result<Vec<UiEvent>, StorageError> {
        let Some(run) = self.store.get_run(run_id).await? else {
            return Err(StorageError::NotFound(format!("run {}", run_id.0)));
        };
        if run.status != RunStatus::Paused {
            return Err(StorageError::Invariant(
                "next-round packages can only be reviewed while the run is paused".to_owned(),
            ));
        }
        let rounds = self.store.list_rounds(run_id).await?;
        let next_round_number = rounds
            .iter()
            .map(|round| round.round_number)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let existing = self
            .store
            .list_dispatches(run_id)
            .await?
            .into_iter()
            .filter(|dispatch| dispatch.outcome == chatmux_common::DispatchOutcome::Pending)
            .collect::<Vec<_>>();
        let workspace_messages = self.store.list_messages(run.workspace_id).await?;
        let (round_number, packages) = if existing.is_empty() {
            (
                next_round_number,
                self.compose_next_round_packages(&run, next_round_number)
                    .await?,
            )
        } else {
            let round_number = existing
                .iter()
                .map(|dispatch| dispatch.round_number)
                .max()
                .unwrap_or(next_round_number);
            let packages = existing
                .into_iter()
                .filter(|dispatch| dispatch.round_number == round_number)
                .map(|dispatch| NextRoundPackage {
                    target_participant_id: dispatch.target_participant_id,
                    round_number: dispatch.round_number,
                    source_blocks: package_source_blocks(
                        &workspace_messages
                            .iter()
                            .filter(|message| dispatch.source_message_ids.contains(&message.id))
                            .cloned()
                            .collect::<Vec<_>>(),
                    ),
                    source_message_ids: dispatch.source_message_ids,
                    template_id: dispatch.template_id,
                    character_count: dispatch.rendered_payload.chars().count(),
                    rendered_payload: dispatch.rendered_payload,
                })
                .collect();
            (round_number, packages)
        };
        Ok(vec![UiEvent::NextRoundPreview {
            run_id,
            round_number,
            packages,
        }])
    }

    async fn compose_next_round_packages(
        &self,
        run: &Run,
        round_number: u32,
    ) -> Result<Vec<NextRoundPackage>, StorageError> {
        let policies = self.store.list_edge_policies(run.workspace_id).await?;
        let messages = self.store.list_messages(run.workspace_id).await?;
        let templates = self.store.list_templates(run.workspace_id).await?;
        let cursors = self.store.list_cursors(run.workspace_id).await?;
        let mut policies_by_target = BTreeMap::<ProviderId, Vec<&EdgePolicy>>::new();
        for edge in &run.graph_snapshot.edges {
            if let Some(policy) = policies.iter().find(|policy| {
                policy.source_participant_id == edge.source
                    && policy.target_participant_id == edge.target
                    && policy.enabled
            }) {
                policies_by_target
                    .entry(edge.target)
                    .or_default()
                    .push(policy);
            }
        }

        let mut packages = Vec::new();
        for (target, target_policies) in policies_by_target {
            let mut selected_ids = BTreeSet::new();
            for policy in &target_policies {
                let cursor = cursors.iter().find(|cursor| {
                    cursor.source_participant_id == policy.source_participant_id
                        && cursor.target_participant_id == policy.target_participant_id
                });
                selected_ids.extend(
                    select_messages_for_edge(&messages, policy, cursor)
                        .into_iter()
                        .map(|message| message.id),
                );
            }
            let selected_messages = messages
                .iter()
                .filter(|message| selected_ids.contains(&message.id))
                .cloned()
                .collect::<Vec<_>>();
            if selected_messages.is_empty() {
                continue;
            }
            let selected_policy = target_policies
                .iter()
                .max_by_key(|policy| policy.priority)
                .copied()
                .ok_or_else(|| {
                    StorageError::Invariant(
                        "next-round preview lost the target edge policy".to_owned(),
                    )
                })?;
            let template = templates
                .iter()
                .find(|template| Some(template.id) == selected_policy.template_id)
                .or_else(|| templates.first())
                .ok_or_else(|| {
                    StorageError::Invariant(
                        "next-round preview requires at least one packaging template".to_owned(),
                    )
                })?;
            let rendered = render_template(template, target, &selected_messages, None);
            packages.push(NextRoundPackage {
                target_participant_id: target,
                round_number,
                source_message_ids: rendered.source_message_ids,
                source_blocks: package_source_blocks(&selected_messages),
                template_id: Some(template.id),
                character_count: rendered.character_count,
                rendered_payload: rendered.body,
            });
        }
        Ok(packages)
    }

    async fn resume_run_with_overrides(
        &self,
        run_id: chatmux_common::RunId,
        payload_overrides: BTreeMap<ProviderId, String>,
        skipped_targets: BTreeSet<ProviderId>,
        injected_user_message: Option<String>,
    ) -> Result<Vec<UiEvent>, StorageError> {
        let Some(run) = self.store.get_run(run_id).await? else {
            return Err(StorageError::NotFound(format!("run {}", run_id.0)));
        };
        if run.status != RunStatus::Paused {
            return Err(StorageError::Invariant(
                "reviewed packages can only be resumed from a paused run".to_owned(),
            ));
        }
        let mut injected_event = None;
        if let Some(text) = injected_user_message
            .map(|text| text.trim().to_owned())
            .filter(|text| !text.is_empty())
        {
            let messages = self.store.list_messages(run.workspace_id).await?;
            let parent = messages.last();
            let message = Message {
                id: chatmux_common::MessageId::new(),
                workspace_id: run.workspace_id,
                participant_id: ProviderId::User,
                role: MessageRole::User,
                round: self
                    .store
                    .list_rounds(run_id)
                    .await?
                    .iter()
                    .map(|round| round.round_number)
                    .max()
                    .map(|round| round.saturating_add(1)),
                parent_message_id: parent.map(|message| message.id),
                child_message_ids: Vec::new(),
                branch_index: parent.map(|message| message.child_message_ids.len() as u32 + 1),
                timestamp: Utc::now(),
                body_text: text.clone(),
                body_blocks: vec![chatmux_common::Block::Paragraph { text }],
                source_binding_id: None,
                dispatch_id: None,
                raw_response_text: None,
                network_capture: None,
                tags: Vec::new(),
                capture_confidence: chatmux_common::CaptureConfidence::Certain,
            };
            self.store.save_message(message.clone()).await?;
            injected_event = Some(UiEvent::MessageCaptured { message });
        }

        let mut events = self.resume_run_execution(run_id).await?;
        let mut affected_round = None;
        let mut has_pending = false;
        for event in &mut events {
            let UiEvent::DispatchUpdated { dispatch } = event else {
                continue;
            };
            affected_round = Some(dispatch.round_number);
            if skipped_targets.contains(&dispatch.target_participant_id) {
                dispatch.outcome = chatmux_common::DispatchOutcome::Skipped;
                dispatch.error_detail = Some("Skipped during between-round review".to_owned());
            } else {
                if let Some(payload) = payload_overrides.get(&dispatch.target_participant_id) {
                    if payload.trim().is_empty() {
                        return Err(StorageError::Invariant(
                            "reviewed outbound packages cannot be empty".to_owned(),
                        ));
                    }
                    dispatch.rendered_payload = payload.clone();
                }
                has_pending |= dispatch.outcome == chatmux_common::DispatchOutcome::Pending;
            }
            self.store.save_dispatch(dispatch.clone()).await?;
        }
        if let Some(event) = injected_event {
            events.insert(0, event);
        }
        if !has_pending && let Some(round_number) = affected_round {
            events.extend(
                self.advance_run_after_terminal_dispatch(run_id, round_number)
                    .await?,
            );
        }
        Ok(events)
    }

    async fn render_export_request(
        &self,
        mut request: ExportRequest,
    ) -> Result<Vec<UiEvent>, StorageError> {
        let Some(workspace) = self.store.get_workspace(request.workspace_id).await? else {
            return Err(StorageError::NotFound(format!(
                "workspace {}",
                request.workspace_id.0
            )));
        };
        let profiles = self
            .store
            .list_export_profiles(request.workspace_id)
            .await?;
        let selected_profile = request.profile_id.and_then(|profile_id| {
            profiles
                .iter()
                .find(|profile| profile.id == profile_id)
                .cloned()
        });
        if request.profile_id.is_some() && selected_profile.is_none() {
            return Err(StorageError::NotFound(
                "selected export profile no longer exists".to_owned(),
            ));
        }
        if let Some(profile) = &selected_profile {
            request.scope = profile.scope_preset;
            request.format = profile.format;
            request.layout = profile.layout;
            request.include_flags = if profile.include_flags == MetadataIncludeFlags::default() {
                export_engine::default_metadata_flags()
            } else {
                profile.include_flags.clone()
            };
            request.filename_template = Some(profile.filename_template.clone());
            request.participants = profile.filter_preset.participants.clone();
            request.roles = profile.filter_preset.roles.clone();
            request.run_id = profile.filter_preset.run_id;
            request.time_range_iso = profile.filter_preset.time_range_iso.clone();
            request.tags = profile.filter_preset.tags.clone();
            request.query = profile.filter_preset.query.clone();
            if let Some((start, end)) = profile.filter_preset.round_range {
                request.selected_rounds = (start..=end).collect();
            }
        }

        let messages = self.store.list_messages(request.workspace_id).await?;
        let runs = self.store.list_runs(request.workspace_id).await?;
        let mut dispatches = Vec::new();
        for run in &runs {
            dispatches.extend(self.store.list_dispatches(run.id).await?);
        }
        let diagnostics = self.store.list_diagnostics(request.workspace_id).await?;
        let selection = export_engine::apply_export_request(
            &request,
            &messages,
            &runs,
            &dispatches,
            &diagnostics,
        )
        .map_err(StorageError::Invariant)?;

        let templates = self.store.list_templates(request.workspace_id).await?;
        let template_name = selection
            .dispatches
            .iter()
            .rev()
            .find_map(|dispatch| dispatch.template_id)
            .or_else(|| {
                dispatches
                    .iter()
                    .rev()
                    .filter(|dispatch| {
                        request
                            .run_id
                            .is_none_or(|run_id| dispatch.run_id == run_id)
                    })
                    .find_map(|dispatch| dispatch.template_id)
            })
            .and_then(|template_id| {
                templates
                    .iter()
                    .find(|template| template.id == template_id)
                    .map(|template| template.name.clone())
            });
        let edge_policies = self.store.list_edge_policies(request.workspace_id).await?;
        let bindings = self.store.list_bindings(request.workspace_id).await?;
        let conversation_refs = bindings
            .iter()
            .filter_map(|binding| {
                binding
                    .conversation_ref
                    .as_ref()
                    .or(binding.bound_conversation_ref.as_ref())
                    .and_then(|reference| {
                        reference
                            .url
                            .clone()
                            .or_else(|| reference.conversation_id.clone())
                    })
            })
            .collect::<Vec<_>>();
        let model_labels = bindings
            .iter()
            .filter_map(|binding| {
                binding
                    .conversation_ref
                    .as_ref()
                    .or(binding.bound_conversation_ref.as_ref())
                    .and_then(|reference| reference.model_label.clone())
            })
            .collect::<Vec<_>>();
        let title = selected_profile
            .as_ref()
            .map(|profile| profile.name.clone())
            .unwrap_or_else(|| format!("{} Export", workspace.name));
        let document = export_engine::build_export_document(
            &workspace,
            &selection.messages,
            &selection.runs,
            &selection.dispatches,
            &selection.diagnostics,
            &export_engine::ExportBuildOptions {
                template_name,
                export_profile_name: selected_profile
                    .as_ref()
                    .map(|profile| profile.name.clone()),
                browser_name: Some("browser-extension".to_owned()),
                extension_version: Some(env!("CARGO_PKG_VERSION").to_owned()),
                title: title.clone(),
                scope: Some(request.scope),
                include_flags: request.include_flags.clone(),
                context_strategy_snapshot: Some(format!(
                    "{:?}",
                    workspace.default_context_strategy
                )),
                edge_policy_snapshot: serde_json::to_string(&edge_policies).ok(),
                conversation_refs,
                model_labels,
            },
        );
        let body = export_engine::render_document(
            &document,
            request.format,
            request.layout,
            request.include_front_matter,
        )
        .map_err(StorageError::Invariant)?;
        let participants = selection
            .messages
            .iter()
            .map(|message| message.participant_id.display_name())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ");
        let filename_values = BTreeMap::from([
            ("title".to_owned(), title),
            ("participants".to_owned(), participants.clone()),
            ("provider".to_owned(), participants),
            (
                "profile".to_owned(),
                selected_profile
                    .as_ref()
                    .map(|profile| profile.name.clone())
                    .unwrap_or_default(),
            ),
            ("scope".to_owned(), format!("{:?}", request.scope)),
            (
                "run".to_owned(),
                request
                    .run_id
                    .map(|run_id| run_id.0.to_string())
                    .unwrap_or_default(),
            ),
        ]);
        let filename_template = request
            .filename_template
            .as_deref()
            .unwrap_or("{workspace}-{date}-{format}");
        let filename = export_engine::render_filename_template_with_values(
            filename_template,
            Some(&workspace),
            request.format,
            &filename_values,
        )
        .map_err(StorageError::Invariant)?;

        Ok(vec![UiEvent::ExportRendered {
            format: request.format,
            mime_type: mime_for_export(request.format).to_owned(),
            filename,
            body,
        }])
    }

    async fn acknowledge_dispatch_delivered(
        &self,
        dispatch_id: chatmux_common::DispatchId,
    ) -> Result<Vec<UiEvent>, StorageError> {
        let Some(dispatch) = self.find_dispatch_global(dispatch_id).await? else {
            return Err(StorageError::NotFound(format!(
                "dispatch {}",
                dispatch_id.0
            )));
        };
        let Some(run) = self.store.get_run(dispatch.run_id).await? else {
            return Err(StorageError::Invariant(
                "dispatch acknowledgement failed: owning run is missing; restore the run ledger before retrying"
                    .to_owned(),
            ));
        };
        let workspace_messages = self.store.list_messages(run.workspace_id).await?;
        let delivered = mark_delivered(dispatch, Utc::now())?;
        let cursor_updates = self
            .acknowledged_cursor_updates(&delivered, run.workspace_id, &workspace_messages)
            .await?;
        self.store.save_dispatch(delivered.clone()).await?;
        for cursor in cursor_updates {
            self.store.save_cursor(cursor).await?;
        }
        Ok(vec![UiEvent::DispatchUpdated {
            dispatch: delivered,
        }])
    }

    async fn acknowledge_dispatch_failed(
        &self,
        dispatch_id: chatmux_common::DispatchId,
        detail: String,
    ) -> Result<Vec<UiEvent>, StorageError> {
        let Some(dispatch) = self.find_dispatch_global(dispatch_id).await? else {
            return Err(StorageError::NotFound(format!(
                "dispatch {}",
                dispatch_id.0
            )));
        };
        let failed = mark_failed(dispatch, &detail)?;
        self.store.save_dispatch(failed.clone()).await?;
        let mut events = vec![UiEvent::DispatchUpdated {
            dispatch: failed.clone(),
        }];
        events.extend(
            self.advance_run_after_terminal_dispatch(failed.run_id, failed.round_number)
                .await?,
        );
        Ok(events)
    }

    async fn acknowledge_dispatch_captured(
        &self,
        dispatch_id: chatmux_common::DispatchId,
        messages: Vec<Message>,
    ) -> Result<Vec<UiEvent>, StorageError> {
        let Some(dispatch) = self.find_dispatch_global(dispatch_id).await? else {
            return Err(StorageError::NotFound(format!(
                "dispatch {}",
                dispatch_id.0
            )));
        };
        let Some(run) = self.store.get_run(dispatch.run_id).await? else {
            return Err(StorageError::Invariant(
                "dispatch capture failed: owning run is missing; restore the run ledger before retrying"
                    .to_owned(),
            ));
        };
        if messages.is_empty() {
            return Err(StorageError::Invariant(
                "dispatch capture failed: no messages were provided; capture at least one provider response before acknowledging"
                    .to_owned(),
            ));
        }
        if messages.iter().any(|message| {
            !message_matches_provider_envelope(message, dispatch.target_participant_id)
        }) {
            return Err(StorageError::Invariant(
                "dispatch capture failed: message identity does not match the target provider; discard the mismatched capture and retry"
                    .to_owned(),
            ));
        }

        let captured = mark_captured(dispatch, Utc::now())?;
        let mut events = Vec::with_capacity(messages.len() + 1);
        // O(n) where n = newly captured messages for one dispatch (expected: 1–100).
        for mut message in messages {
            message.workspace_id = run.workspace_id;
            message.dispatch_id = Some(captured.id);
            message.round = Some(captured.round_number);
            self.store.save_message(message.clone()).await?;
            events.push(UiEvent::MessageCaptured { message });
        }
        self.store.save_dispatch(captured.clone()).await?;
        events.push(UiEvent::DispatchUpdated {
            dispatch: captured.clone(),
        });
        events.extend(
            self.advance_run_after_terminal_dispatch(run.id, captured.round_number)
                .await?,
        );
        Ok(events)
    }

    async fn advance_run_after_terminal_dispatch(
        &self,
        run_id: chatmux_common::RunId,
        round_number: u32,
    ) -> Result<Vec<UiEvent>, StorageError> {
        let Some(mut run) = self.store.get_run(run_id).await? else {
            return Err(StorageError::NotFound(format!("run {}", run_id.0)));
        };
        if run.status != RunStatus::Running {
            return Ok(Vec::new());
        }
        let mut rounds = self.store.list_rounds(run_id).await?;
        let Some(round_index) = rounds
            .iter()
            .position(|round| round.round_number == round_number)
        else {
            return Err(StorageError::Invariant(
                "terminal dispatch references a missing round".to_owned(),
            ));
        };
        if rounds[round_index].status != RoundStatus::Running {
            return Ok(Vec::new());
        }
        let round_dispatches = self
            .store
            .list_dispatches(run_id)
            .await?
            .into_iter()
            .filter(|dispatch| dispatch.round_number == round_number)
            .collect::<Vec<_>>();
        let active_targets = round_dispatches
            .iter()
            .map(|dispatch| dispatch.target_participant_id)
            .filter(|provider| run.participant_set.contains(provider))
            .collect::<BTreeSet<_>>();
        if active_targets.is_empty() {
            return Ok(Vec::new());
        }
        let terminal_targets = round_dispatches
            .iter()
            .filter(|dispatch| {
                (dispatch.outcome == chatmux_common::DispatchOutcome::Delivered
                    && dispatch.captured_at.is_some())
                    || matches!(
                        dispatch.outcome,
                        chatmux_common::DispatchOutcome::Timeout
                            | chatmux_common::DispatchOutcome::Error
                            | chatmux_common::DispatchOutcome::Skipped
                    )
            })
            .map(|dispatch| dispatch.target_participant_id)
            .collect::<BTreeSet<_>>();
        if !barrier_satisfied(&run.barrier_policy, &terminal_targets, &active_targets) {
            return Ok(Vec::new());
        }

        let now = Utc::now();
        let round_failed = round_dispatches.iter().any(|dispatch| {
            matches!(
                dispatch.outcome,
                chatmux_common::DispatchOutcome::Error | chatmux_common::DispatchOutcome::Timeout
            )
        });
        rounds[round_index].status = if round_failed {
            RoundStatus::Failed
        } else {
            RoundStatus::Completed
        };
        rounds[round_index].completed_at = Some(now);
        self.store.save_round(rounds[round_index].clone()).await?;

        let completed_rounds = rounds
            .iter()
            .filter(|round| {
                matches!(
                    round.status,
                    RoundStatus::Completed | RoundStatus::Failed | RoundStatus::TimedOut
                )
            })
            .count() as u32;
        let all_dispatches = self.store.list_dispatches(run_id).await?;
        let failures = all_dispatches
            .iter()
            .filter(|dispatch| dispatch.outcome == chatmux_common::DispatchOutcome::Error)
            .count() as u32;
        let timeouts = all_dispatches
            .iter()
            .filter(|dispatch| dispatch.outcome == chatmux_common::DispatchOutcome::Timeout)
            .count() as u32;
        let all_messages = self.store.list_messages(run.workspace_id).await?;
        let recent_window = run.stop_policy.stagnation_window.unwrap_or(1).max(1);
        let first_round = round_number.saturating_sub(recent_window.saturating_sub(1));
        let recent_round_bodies = (first_round..=round_number)
            .map(|candidate_round| {
                all_messages
                    .iter()
                    .filter(|message| message.round == Some(candidate_round))
                    .map(|message| message.body_text.clone())
                    .collect::<Vec<_>>()
            })
            .filter(|round| !round.is_empty())
            .collect::<Vec<_>>();
        let global_timeout_reached = run
            .timing_policy
            .global_run_timeout_secs
            .zip(run.started_at)
            .is_some_and(|(limit, started)| {
                now.signed_duration_since(started).num_seconds() >= limit as i64
            });
        let autonomous = matches!(
            run.mode,
            OrchestrationMode::Roundtable
                | OrchestrationMode::ModeratorJury
                | OrchestrationMode::RelayChain
                | OrchestrationMode::ModeratedAutonomous
        );
        if !autonomous
            || global_timeout_reached
            || should_stop_run(
                &run.timing_policy,
                &run.stop_policy,
                completed_rounds,
                failures,
                timeouts,
                &recent_round_bodies,
            )
        {
            run.status = RunStatus::Completed;
            run.ended_at = Some(now);
            self.store.save_run(run.clone()).await?;
            return Ok(vec![UiEvent::RunUpdated { run, rounds }]);
        }
        if run.stop_policy.require_approval_between_rounds
            || run.mode == OrchestrationMode::ModeratedAutonomous
        {
            run.status = RunStatus::Paused;
            self.store.save_run(run.clone()).await?;
            return Ok(vec![UiEvent::RunUpdated { run, rounds }]);
        }

        let next_round = Round {
            id: chatmux_common::RoundId::new(),
            run_id,
            round_number: round_number.saturating_add(1),
            started_at: Some(now),
            completed_at: None,
            status: RoundStatus::Running,
        };
        self.store.save_round(next_round.clone()).await?;
        rounds.push(next_round);
        let policies = self.store.list_edge_policies(run.workspace_id).await?;
        let messages = self.store.list_messages(run.workspace_id).await?;
        let templates = self.store.list_templates(run.workspace_id).await?;
        let dispatches = self
            .synthesize_dispatches(&run, &policies, &messages, &templates)
            .await?;
        if dispatches.is_empty() {
            run.status = RunStatus::Paused;
            self.store.save_run(run.clone()).await?;
        }
        let mut events = vec![UiEvent::RunUpdated { run, rounds }];
        events.extend(
            dispatches
                .into_iter()
                .map(|dispatch| UiEvent::DispatchUpdated { dispatch }),
        );
        Ok(events)
    }

    async fn acknowledged_cursor_updates(
        &self,
        dispatch: &Dispatch,
        workspace_id: chatmux_common::WorkspaceId,
        workspace_messages: &[Message],
    ) -> Result<Vec<DeliveryCursor>, StorageError> {
        let messages_by_id = workspace_messages
            .iter()
            .map(|message| (message.id, message))
            .collect::<BTreeMap<_, _>>();
        let mut delivered_by_source = BTreeMap::<ProviderId, Vec<Message>>::new();

        // O(n) where n = source messages in one dispatch (expected: 1–1000).
        for message_id in &dispatch.source_message_ids {
            let Some(message) = messages_by_id.get(message_id) else {
                return Err(StorageError::Invariant(
                    "cursor acknowledgement failed: a source message is missing; restore the message ledger before retrying"
                        .to_owned(),
                ));
            };
            delivered_by_source
                .entry(message.participant_id)
                .or_default()
                .push((*message).clone());
        }

        let cursors = self.store.list_cursors(workspace_id).await?;
        let mut updates = Vec::new();
        for (source, delivered_messages) in delivered_by_source {
            let existing = cursors.iter().find(|cursor| {
                cursor.source_participant_id == source
                    && cursor.target_participant_id == dispatch.target_participant_id
            });
            let cursor = existing.cloned().unwrap_or(DeliveryCursor {
                id: DeliveryCursorId::new(),
                workspace_id,
                source_participant_id: source,
                target_participant_id: dispatch.target_participant_id,
                last_delivered_message_id: None,
                last_delivered_at: None,
                frozen: false,
            });
            if cursor_is_at_or_after(&cursor, &delivered_messages, workspace_messages) {
                continue;
            }
            updates.push(advance_cursor(&cursor, &delivered_messages));
        }
        Ok(updates)
    }

    async fn find_dispatch_global(
        &self,
        dispatch_id: chatmux_common::DispatchId,
    ) -> Result<Option<Dispatch>, StorageError> {
        // O(w*r*d) over persisted workspaces, runs, and dispatches; acknowledgement volume is
        // low and the browser store can add an indexed lookup behind StateStore later.
        for workspace in self.store.list_workspaces().await? {
            for run in self.store.list_runs(workspace.id).await? {
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
        }
        Ok(None)
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
            tab_title: None,
            tab_url: None,
            pinned: false,
            stale: false,
            bound_conversation_ref: None,
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

    async fn persist_provider_health(
        &self,
        workspace_id: chatmux_common::WorkspaceId,
        provider: ProviderId,
        health: ProviderHealth,
    ) -> Result<ParticipantBinding, StorageError> {
        self.upsert_binding_for_provider(workspace_id, provider, |binding| {
            binding.health_state = health;
        })
        .await
    }
}

fn cursor_is_at_or_after(
    cursor: &DeliveryCursor,
    delivered_messages: &[Message],
    workspace_messages: &[Message],
) -> bool {
    if cursor.frozen {
        return true;
    }
    let Some(current_id) = cursor.last_delivered_message_id else {
        return false;
    };
    let Some(candidate_id) = delivered_messages.last().map(|message| message.id) else {
        return true;
    };
    let current_position = workspace_messages
        .iter()
        .position(|message| message.id == current_id);
    let candidate_position = workspace_messages
        .iter()
        .position(|message| message.id == candidate_id);

    match (current_position, candidate_position) {
        (Some(current), Some(candidate)) => current >= candidate,
        _ => true,
    }
}

fn message_matches_provider_envelope(message: &Message, provider: ProviderId) -> bool {
    match message.role {
        MessageRole::Assistant => message.participant_id == provider,
        MessageRole::User => message.participant_id == ProviderId::User,
        MessageRole::System => message.participant_id == ProviderId::System,
    }
}

fn summarize_diagnostics(
    workspace_id: Option<chatmux_common::WorkspaceId>,
    events: &[DiagnosticEvent],
) -> WorkspaceDiagnosticsSummary {
    let mut summary = WorkspaceDiagnosticsSummary {
        workspace_id,
        total: events.len() as u32,
        ..WorkspaceDiagnosticsSummary::default()
    };

    for event in events {
        match event.level {
            DiagnosticLevel::Critical => summary.critical += 1,
            DiagnosticLevel::Warning => summary.warning += 1,
            DiagnosticLevel::Info => summary.info += 1,
            DiagnosticLevel::Debug => summary.debug += 1,
        }
        summary.last_event_at = Some(
            summary
                .last_event_at
                .map(|current| current.max(event.timestamp))
                .unwrap_or(event.timestamp),
        );
    }

    summary
}

#[allow(clippy::too_many_arguments)] // Reason: diagnostic construction mirrors the structured event fields.
fn diagnostic_event(
    workspace_id: chatmux_common::WorkspaceId,
    scope: DiagnosticScope,
    source: DiagnosticSource,
    level: DiagnosticLevel,
    code: &str,
    title: String,
    summary: String,
    detail: String,
) -> DiagnosticEvent {
    DiagnosticEvent {
        id: chatmux_common::DiagnosticEventId::new(),
        workspace_id,
        scope,
        source,
        binding_id: None,
        provider_id: None,
        run_id: None,
        round_id: None,
        message_id: None,
        dispatch_id: None,
        timestamp: Utc::now(),
        level,
        code: code.to_owned(),
        title,
        summary,
        detail,
        tags: vec![source_tag(source), level_tag(level)],
        attributes: BTreeMap::new(),
        artifact_refs: Vec::new(),
        snapshot_ref: None,
    }
}

fn enrich_diagnostic(
    mut diagnostic: DiagnosticEvent,
    event_name: &str,
    payload: &str,
    result_count: Option<String>,
    provider_id: Option<ProviderId>,
) -> DiagnosticEvent {
    diagnostic.provider_id = provider_id;
    diagnostic
        .attributes
        .insert("event_name".to_owned(), event_name.to_owned());
    diagnostic.attributes.insert(
        "payload_json".to_owned(),
        truncate_text(payload.to_owned(), 4_000),
    );
    if let Some(result_count) = result_count {
        diagnostic
            .attributes
            .insert("result_count".to_owned(), result_count);
    }
    diagnostic
}

fn sanitize_diagnostic_event(mut event: DiagnosticEvent) -> DiagnosticEvent {
    event.title = truncate_text(event.title, 240);
    event.summary = truncate_text(event.summary, 1_200);
    event.detail = truncate_text(event.detail, 8_000);
    event.attributes = event
        .attributes
        .into_iter()
        .take(24)
        .map(|(key, value)| (key, truncate_text(value, 2_000)))
        .collect();
    event
}

fn summarize_ui_events(events: &[UiEvent]) -> String {
    events
        .iter()
        .map(|event| match event {
            UiEvent::WorkspaceList { workspaces } => {
                format!("workspace_list(count={})", workspaces.len())
            }
            UiEvent::WorkspaceSnapshot { snapshot } => format!(
                "workspace_snapshot(workspace={}, messages={}, diagnostics={})",
                snapshot
                    .workspace
                    .as_ref()
                    .map(|item| item.name.as_str())
                    .unwrap_or("none"),
                snapshot.recent_messages.len(),
                snapshot.diagnostics.len()
            ),
            UiEvent::RunUpdated { run, rounds } => {
                format!(
                    "run_updated(status={:?}, rounds={})",
                    run.status,
                    rounds.len()
                )
            }
            UiEvent::NextRoundPreview {
                round_number,
                packages,
                ..
            } => format!(
                "next_round_preview(round={round_number}, packages={})",
                packages.len()
            ),
            UiEvent::ManualMessagePreview { packages } => {
                format!("manual_message_preview(packages={})", packages.len())
            }
            UiEvent::RunLedgerSnapshot { ledger } => format!(
                "run_ledger_snapshot(dispatches={}, rounds={})",
                ledger.dispatches.len(),
                ledger.rounds.len()
            ),
            UiEvent::MessageCaptured { message } => format!(
                "message_captured(provider={}, chars={})",
                message.participant_id.display_name(),
                message.body_text.len()
            ),
            UiEvent::DispatchUpdated { dispatch } => format!(
                "dispatch_updated(target={}, outcome={:?})",
                dispatch.target_participant_id.display_name(),
                dispatch.outcome
            ),
            UiEvent::DiagnosticRaised { diagnostic } => {
                format!(
                    "diagnostic_raised(code={}, level={:?})",
                    diagnostic.code, diagnostic.level
                )
            }
            UiEvent::DiagnosticsSnapshot { snapshot } => {
                format!("diagnostics_snapshot(events={})", snapshot.events.len())
            }
            UiEvent::ProviderHealthChanged {
                provider, health, ..
            } => {
                format!(
                    "provider_health_changed(provider={}, health={:?})",
                    provider.display_name(),
                    health
                )
            }
            UiEvent::ProviderControlUpdated { snapshot, .. } => {
                format!(
                    "provider_control_updated(provider={})",
                    snapshot.provider.display_name()
                )
            }
            UiEvent::ProviderTabCandidates {
                provider,
                candidates,
                ..
            } => {
                format!(
                    "provider_tab_candidates(provider={}, count={})",
                    provider.display_name(),
                    candidates.len()
                )
            }
            UiEvent::ProviderDefaultsUpdated { provider, .. } => {
                format!(
                    "provider_defaults_updated(provider={})",
                    provider.display_name()
                )
            }
            UiEvent::ExportRendered {
                format, filename, ..
            } => {
                format!("export_rendered(format={format:?}, filename={filename})")
            }
            UiEvent::MessageInspection {
                message, dispatch, ..
            } => format!(
                "message_inspection(message={}, dispatch={})",
                message.is_some(),
                dispatch.is_some()
            ),
            UiEvent::KillSwitchChanged { active } => {
                format!("kill_switch_changed(active={active})")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn truncate_text(value: String, max_chars: usize) -> String {
    let total = value.chars().count();
    if total <= max_chars {
        return value;
    }

    let truncated = value.chars().take(max_chars).collect::<String>();
    format!("{truncated}\n… [truncated {} chars]", total - max_chars)
}

fn combine_manual_notes(pinned_note: Option<&str>, target_note: Option<&str>) -> Option<String> {
    let pinned_note = pinned_note.filter(|note| !note.trim().is_empty());
    let target_note = target_note.filter(|note| !note.trim().is_empty());
    match (pinned_note, target_note) {
        (Some(pinned), Some(target)) => {
            Some(format!("Pinned note:\n{pinned}\n\nTarget note:\n{target}"))
        }
        (Some(pinned), None) => Some(format!("Pinned note:\n{pinned}")),
        (None, Some(target)) => Some(format!("Target note:\n{target}")),
        (None, None) => None,
    }
}

fn package_source_blocks(messages: &[Message]) -> Vec<chatmux_common::PackageSourceBlock> {
    messages
        .iter()
        .map(|message| chatmux_common::PackageSourceBlock {
            message_id: message.id,
            participant_id: message.participant_id,
            role: message.role,
            preview: truncate_text(message.body_text.clone(), 180),
        })
        .collect()
}

fn source_tag(source: DiagnosticSource) -> String {
    format!("source:{source:?}").to_lowercase()
}

fn level_tag(level: DiagnosticLevel) -> String {
    format!("level:{level:?}").to_lowercase()
}

fn render_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "<serialization failed>".to_owned())
}

fn ui_command_name(command: &UiCommand) -> String {
    match command {
        UiCommand::RequestWorkspaceList => "request_workspace_list",
        UiCommand::CreateWorkspace { .. } => "create_workspace",
        UiCommand::DeleteWorkspace { .. } => "delete_workspace",
        UiCommand::RenameWorkspace { .. } => "rename_workspace",
        UiCommand::DuplicateWorkspace { .. } => "duplicate_workspace",
        UiCommand::ExportWorkspaceArchive { .. } => "export_workspace_archive",
        UiCommand::ImportWorkspaceArchive { .. } => "import_workspace_archive",
        UiCommand::SetWorkspaceArchived { .. } => "set_workspace_archived",
        UiCommand::OpenWorkspace { .. } => "open_workspace",
        UiCommand::PersistTemplate { .. } => "persist_template",
        UiCommand::PersistEdgePolicy { .. } => "persist_edge_policy",
        UiCommand::PersistPinnedSummary { .. } => "persist_pinned_summary",
        UiCommand::DeletePinnedSummary { .. } => "delete_pinned_summary",
        UiCommand::ResetDeliveryCursor { .. } => "reset_delivery_cursor",
        UiCommand::SetDeliveryCursorFrozen { .. } => "set_delivery_cursor_frozen",
        UiCommand::PersistExportProfile { .. } => "persist_export_profile",
        UiCommand::DeleteTemplate { .. } => "delete_template",
        UiCommand::StartRun { .. } => "start_run",
        UiCommand::StartConfiguredRun { .. } => "start_configured_run",
        UiCommand::PauseRun { .. } => "pause_run",
        UiCommand::ResumeRun { .. } => "resume_run",
        UiCommand::PreviewNextRound { .. } => "preview_next_round",
        UiCommand::ResumeRunWithOverrides { .. } => "resume_run_with_overrides",
        UiCommand::StepRun { .. } => "step_run",
        UiCommand::StopRun { .. } => "stop_run",
        UiCommand::AbortRun { .. } => "abort_run",
        UiCommand::PreviewManualMessage { .. } => "preview_manual_message",
        UiCommand::SendManualMessage { .. } => "send_manual_message",
        UiCommand::AcknowledgeDispatchDelivered { .. } => "acknowledge_dispatch_delivered",
        UiCommand::AcknowledgeDispatchFailed { .. } => "acknowledge_dispatch_failed",
        UiCommand::AcknowledgeDispatchCaptured { .. } => "acknowledge_dispatch_captured",
        UiCommand::SyncProviderConversation { .. } => "sync_provider_conversation",
        UiCommand::RequestProviderTabCandidates { .. } => "request_provider_tab_candidates",
        UiCommand::BindProviderTab { .. } => "bind_provider_tab",
        UiCommand::OpenProviderTab { .. } => "open_provider_tab",
        UiCommand::ExportSelection { .. } => "export_selection",
        UiCommand::ExportConfigured { .. } => "export_configured",
        UiCommand::RequestMessageInspection { .. } => "request_message_inspection",
        UiCommand::SetKillSwitch { .. } => "set_kill_switch",
        UiCommand::ClearWorkspaceData { .. } => "clear_workspace_data",
        UiCommand::ToggleProvider { .. } => "toggle_provider",
        UiCommand::RequestProviderControlState { .. } => "request_provider_control_state",
        UiCommand::CreateProviderProject { .. } => "create_provider_project",
        UiCommand::SelectProviderProject { .. } => "select_provider_project",
        UiCommand::CreateProviderConversation { .. } => "create_provider_conversation",
        UiCommand::SelectProviderConversation { .. } => "select_provider_conversation",
        UiCommand::SetProviderModel { .. } => "set_provider_model",
        UiCommand::SetProviderReasoning { .. } => "set_provider_reasoning",
        UiCommand::SetProviderFeatureFlag { .. } => "set_provider_feature_flag",
        UiCommand::PersistProviderDefaults { .. } => "persist_provider_defaults",
        UiCommand::RequestWorkspaceSnapshot { .. } => "request_workspace_snapshot",
        UiCommand::RequestRunLedger { .. } => "request_run_ledger",
        UiCommand::RequestDiagnosticsSnapshot { .. } => "request_diagnostics_snapshot",
        UiCommand::ClearDiagnostics { .. } => "clear_diagnostics",
    }
    .to_owned()
}

fn ui_command_workspace_id(command: &UiCommand) -> Option<chatmux_common::WorkspaceId> {
    match command {
        UiCommand::CreateWorkspace { .. }
        | UiCommand::ImportWorkspaceArchive { .. }
        | UiCommand::RequestWorkspaceList
        | UiCommand::PauseRun { .. }
        | UiCommand::ResumeRun { .. }
        | UiCommand::PreviewNextRound { .. }
        | UiCommand::ResumeRunWithOverrides { .. }
        | UiCommand::RequestRunLedger { .. }
        | UiCommand::StepRun { .. }
        | UiCommand::StopRun { .. }
        | UiCommand::AbortRun { .. }
        | UiCommand::RequestMessageInspection { .. }
        | UiCommand::SetKillSwitch { .. }
        | UiCommand::AcknowledgeDispatchDelivered { .. }
        | UiCommand::AcknowledgeDispatchFailed { .. }
        | UiCommand::AcknowledgeDispatchCaptured { .. }
        | UiCommand::ResetDeliveryCursor { .. }
        | UiCommand::SetDeliveryCursorFrozen { .. }
        | UiCommand::DeleteTemplate { .. }
        | UiCommand::PersistProviderDefaults { .. } => None,
        UiCommand::DeleteWorkspace { workspace_id }
        | UiCommand::RenameWorkspace { workspace_id, .. }
        | UiCommand::DuplicateWorkspace { workspace_id }
        | UiCommand::SetWorkspaceArchived { workspace_id, .. }
        | UiCommand::OpenWorkspace { workspace_id }
        | UiCommand::ExportWorkspaceArchive { workspace_id }
        | UiCommand::PersistPinnedSummary { workspace_id, .. }
        | UiCommand::DeletePinnedSummary { workspace_id, .. }
        | UiCommand::StartRun { workspace_id, .. }
        | UiCommand::StartConfiguredRun { workspace_id, .. }
        | UiCommand::PreviewManualMessage { workspace_id, .. }
        | UiCommand::SendManualMessage { workspace_id, .. }
        | UiCommand::SyncProviderConversation { workspace_id, .. }
        | UiCommand::RequestProviderTabCandidates { workspace_id, .. }
        | UiCommand::BindProviderTab { workspace_id, .. }
        | UiCommand::OpenProviderTab { workspace_id, .. }
        | UiCommand::ExportSelection { workspace_id, .. }
        | UiCommand::ClearWorkspaceData { workspace_id }
        | UiCommand::ToggleProvider { workspace_id, .. }
        | UiCommand::RequestProviderControlState { workspace_id, .. }
        | UiCommand::CreateProviderProject { workspace_id, .. }
        | UiCommand::SelectProviderProject { workspace_id, .. }
        | UiCommand::CreateProviderConversation { workspace_id, .. }
        | UiCommand::SelectProviderConversation { workspace_id, .. }
        | UiCommand::SetProviderModel { workspace_id, .. }
        | UiCommand::SetProviderReasoning { workspace_id, .. }
        | UiCommand::SetProviderFeatureFlag { workspace_id, .. }
        | UiCommand::RequestWorkspaceSnapshot { workspace_id } => Some(*workspace_id),
        UiCommand::ExportConfigured { request } => Some(request.workspace_id),
        UiCommand::PersistTemplate { template } => Some(template.workspace_id),
        UiCommand::PersistEdgePolicy { policy } => Some(policy.workspace_id),
        UiCommand::PersistExportProfile { profile } => Some(profile.workspace_id),
        UiCommand::RequestDiagnosticsSnapshot { query } | UiCommand::ClearDiagnostics { query } => {
            query.workspace_id
        }
    }
}

fn slugify(value: &str) -> String {
    let slug = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let compact = slug
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if compact.is_empty() {
        "workspace".to_owned()
    } else {
        compact
    }
}

fn adapter_event_name(event: &AdapterToBackground) -> String {
    match event {
        AdapterToBackground::HealthReport { .. } => "health_report",
        AdapterToBackground::BlockingStateDetected { .. } => "blocking_state_detected",
        AdapterToBackground::MessagesCaptured { .. } => "messages_captured",
        AdapterToBackground::StructuralProbePassed { .. } => "structural_probe_passed",
        AdapterToBackground::StructuralProbeFailed { .. } => "structural_probe_failed",
        AdapterToBackground::ConversationRefDiscovered { .. } => "conversation_ref_discovered",
        AdapterToBackground::ProviderControlSnapshotCaptured { .. } => {
            "provider_control_snapshot_captured"
        }
        AdapterToBackground::CommandFailed { .. } => "command_failed",
    }
    .to_owned()
}

fn adapter_event_provider(events: &[UiEvent]) -> Option<ProviderId> {
    events.iter().find_map(|event| match event {
        UiEvent::ProviderHealthChanged { provider, .. } => Some(*provider),
        UiEvent::ProviderControlUpdated { snapshot, .. } => Some(snapshot.provider),
        _ => None,
    })
}

fn default_edge_policy(
    workspace_id: chatmux_common::WorkspaceId,
    source: ProviderId,
    target: ProviderId,
    template_id: Option<chatmux_common::TemplateId>,
) -> EdgePolicy {
    EdgePolicy {
        id: chatmux_common::EdgePolicyId::new(),
        workspace_id,
        source_participant_id: source,
        target_participant_id: target,
        enabled: true,
        catch_up_policy: chatmux_common::CatchUpPolicy::FullHistory,
        incremental_policy: chatmux_common::IncrementalPolicy::UnseenDeltaOnly,
        self_exclusion: true,
        include_user_turns: true,
        include_system_notes: true,
        include_pinned_summaries: true,
        include_moderator_annotations: true,
        include_target_prior_turns: false,
        truncation_policy: chatmux_common::TruncationPolicy::WarnOnly {
            soft_character_limit: 48_000,
        },
        priority: 0,
        approval_mode: chatmux_common::ApprovalMode::AutoSend,
        template_id,
    }
}

fn default_workspace_edge_policies(
    workspace_id: chatmux_common::WorkspaceId,
    template_id: Option<chatmux_common::TemplateId>,
) -> Vec<EdgePolicy> {
    let providers = [
        ProviderId::Gpt,
        ProviderId::Gemini,
        ProviderId::Grok,
        ProviderId::Claude,
    ];
    let sources = [
        ProviderId::User,
        ProviderId::Gpt,
        ProviderId::Gemini,
        ProviderId::Grok,
        ProviderId::Claude,
    ];

    sources
        .into_iter()
        .flat_map(|source| {
            providers
                .into_iter()
                .filter(move |target| *target != source)
                .map(move |target| (source, target))
        })
        .enumerate()
        .map(|(priority, (source, target))| {
            let mut policy = default_edge_policy(workspace_id, source, target, template_id);
            policy.priority = i32::try_from(priority).unwrap_or(i32::MAX);
            policy
        })
        .collect()
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
        state.conversation_title = binding
            .conversation_ref
            .as_ref()
            .and_then(|item| item.title.clone());
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
        ApprovalMode, Block, CaptureConfidence, DispatchOutcome, ExportFilterPreset, ExportLayout,
        ExportProfile, ExportProfileId, ExportScopePreset, MessageId, MessageRole,
        MetadataIncludeFlags, RouteEdge, RoutingGraph, StopPolicy, TemplateId, TemplateKind,
        WorkspaceId,
    };
    use futures::executor::block_on;

    fn workspace(workspace_id: WorkspaceId, providers: BTreeSet<ProviderId>) -> Workspace {
        Workspace {
            id: workspace_id,
            name: "Workspace".to_owned(),
            archived: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            enabled_providers: providers,
            default_mode: OrchestrationMode::Broadcast,
            default_context_strategy: ContextStrategy::WorkspaceDefault,
            default_template_id: None,
            active_export_profile_ids: Vec::new(),
            tags: Vec::new(),
            notes: None,
        }
    }

    fn assistant_message(workspace_id: WorkspaceId, provider: ProviderId, body: &str) -> Message {
        Message {
            id: MessageId::new(),
            workspace_id,
            participant_id: provider,
            role: MessageRole::Assistant,
            round: Some(1),
            parent_message_id: None,
            child_message_ids: Vec::new(),
            branch_index: None,
            timestamp: Utc::now(),
            body_text: body.to_owned(),
            body_blocks: vec![Block::Paragraph {
                text: body.to_owned(),
            }],
            source_binding_id: None,
            dispatch_id: None,
            raw_response_text: None,
            network_capture: None,
            tags: Vec::new(),
            capture_confidence: CaptureConfidence::Certain,
        }
    }

    #[test]
    fn manual_preview_renders_without_persisting_state() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            store
                .save_workspace(workspace(workspace_id, BTreeSet::from([ProviderId::Gpt])))
                .await
                .expect("workspace saves");

            let events = coordinator
                .handle_ui_command(UiCommand::PreviewManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Gpt],
                    text: "review this exact package".to_owned(),
                    selected_message_ids: BTreeSet::new(),
                    pinned_note: None,
                    target_notes: BTreeMap::new(),
                    include_target_prior_turns: false,
                    parent_message_id: None,
                })
                .await
                .expect("manual preview renders");

            let packages = events.iter().find_map(|event| match event {
                UiEvent::ManualMessagePreview { packages } => Some(packages),
                _ => None,
            });
            assert!(matches!(packages, Some(items) if
                items.len() == 1
                    && items[0].target_participant_id == ProviderId::Gpt
                    && items[0].rendered_payload
                        == "<user-input>\nreview this exact package\n</user-input>"));
            assert!(
                store
                    .list_messages(workspace_id)
                    .await
                    .expect("messages load")
                    .is_empty()
            );
            assert!(
                store
                    .list_runs(workspace_id)
                    .await
                    .expect("runs load")
                    .is_empty()
            );
            assert!(
                store
                    .list_cursors(workspace_id)
                    .await
                    .expect("cursors load")
                    .is_empty()
            );
        });
    }

    #[test]
    fn manual_preview_includes_selected_context_notes_and_excludes_target_history() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            store
                .save_workspace(workspace(
                    workspace_id,
                    BTreeSet::from([ProviderId::Gpt, ProviderId::Claude]),
                ))
                .await
                .expect("workspace saves");
            let source = assistant_message(workspace_id, ProviderId::Gpt, "source evidence");
            let target = assistant_message(workspace_id, ProviderId::Claude, "target prior answer");
            store
                .save_message(source.clone())
                .await
                .expect("source saves");
            store
                .save_message(target.clone())
                .await
                .expect("target saves");

            let events = coordinator
                .handle_ui_command(UiCommand::PreviewManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Claude],
                    text: "new instruction".to_owned(),
                    selected_message_ids: BTreeSet::from([source.id, target.id]),
                    pinned_note: Some("Use the evidence.".to_owned()),
                    target_notes: BTreeMap::from([
                        (ProviderId::Claude, "Act as reviewer.".to_owned()),
                        (ProviderId::Gpt, "Not for Claude.".to_owned()),
                    ]),
                    include_target_prior_turns: false,
                    parent_message_id: None,
                })
                .await
                .expect("manual preview renders");

            let package = events.iter().find_map(|event| match event {
                UiEvent::ManualMessagePreview { packages } => packages.first(),
                _ => None,
            });
            assert!(matches!(package, Some(item)
                if item.rendered_payload.contains("<gpt-response>\nsource evidence\n</gpt-response>")
                    && item.rendered_payload.contains("Use the evidence.")
                    && item.rendered_payload.contains("Act as reviewer.")
                    && !item.rendered_payload.contains("target prior answer")
                    && !item.rendered_payload.contains("Not for Claude.")
                    && item.source_message_ids.contains(&source.id)
                    && !item.source_message_ids.contains(&target.id)));
        });
    }

    #[test]
    fn manual_send_persists_exact_reviewed_payload_override() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            store
                .save_workspace(workspace(workspace_id, BTreeSet::from([ProviderId::Gpt])))
                .await
                .expect("workspace saves");

            let events = coordinator
                .handle_ui_command(UiCommand::SendManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Gpt],
                    text: "original draft".to_owned(),
                    approval_mode: ApprovalMode::AutoSend,
                    selected_message_ids: BTreeSet::new(),
                    pinned_note: None,
                    target_notes: BTreeMap::new(),
                    include_target_prior_turns: false,
                    payload_overrides: BTreeMap::from([(
                        ProviderId::Gpt,
                        "edited exact GPT payload".to_owned(),
                    )]),
                    parent_message_id: None,
                })
                .await
                .expect("manual send prepares");

            let dispatch = events.iter().find_map(|event| match event {
                UiEvent::DispatchUpdated { dispatch } => Some(dispatch),
                _ => None,
            });
            assert!(matches!(dispatch, Some(item)
                if item.target_participant_id == ProviderId::Gpt
                    && item.rendered_payload == "edited exact GPT payload"
                    && item.outcome == DispatchOutcome::Pending));
        });
    }

    #[test]
    fn manual_send_matches_preview_for_selected_context_without_override() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            store
                .save_workspace(workspace(
                    workspace_id,
                    BTreeSet::from([ProviderId::Gpt, ProviderId::Claude]),
                ))
                .await
                .expect("workspace saves");
            let source = assistant_message(workspace_id, ProviderId::Gpt, "source evidence");
            store
                .save_message(source.clone())
                .await
                .expect("source saves");
            let selected_message_ids = BTreeSet::from([source.id]);
            let target_notes =
                BTreeMap::from([(ProviderId::Claude, "Act as reviewer.".to_owned())]);

            let preview = coordinator
                .handle_ui_command(UiCommand::PreviewManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Claude],
                    text: "new instruction".to_owned(),
                    selected_message_ids: selected_message_ids.clone(),
                    pinned_note: Some("Use the evidence.".to_owned()),
                    target_notes: target_notes.clone(),
                    include_target_prior_turns: false,
                    parent_message_id: None,
                })
                .await
                .expect("preview renders");
            let preview_payload = preview.iter().find_map(|event| match event {
                UiEvent::ManualMessagePreview { packages } => packages
                    .first()
                    .map(|package| package.rendered_payload.clone()),
                _ => None,
            });

            let sent = coordinator
                .handle_ui_command(UiCommand::SendManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Claude],
                    text: "new instruction".to_owned(),
                    approval_mode: ApprovalMode::AutoSend,
                    selected_message_ids,
                    pinned_note: Some("Use the evidence.".to_owned()),
                    target_notes,
                    include_target_prior_turns: false,
                    payload_overrides: BTreeMap::new(),
                    parent_message_id: None,
                })
                .await
                .expect("send prepares");
            let sent_payload = sent.iter().find_map(|event| match event {
                UiEvent::DispatchUpdated { dispatch } => Some(dispatch.rendered_payload.clone()),
                _ => None,
            });

            assert_eq!(sent_payload, preview_payload);
        });
    }

    #[test]
    fn manual_auto_send_persists_pending_dispatch_and_run() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            store
                .save_workspace(workspace(workspace_id, BTreeSet::from([ProviderId::Gpt])))
                .await
                .expect("workspace saves");

            let events = coordinator
                .handle_ui_command(UiCommand::SendManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Gpt],
                    text: "hello".to_owned(),
                    approval_mode: ApprovalMode::AutoSend,
                    selected_message_ids: BTreeSet::new(),
                    pinned_note: None,
                    target_notes: BTreeMap::new(),
                    include_target_prior_turns: false,
                    payload_overrides: BTreeMap::new(),
                    parent_message_id: None,
                })
                .await
                .expect("manual send prepares");

            let dispatch = events.iter().find_map(|event| match event {
                UiEvent::DispatchUpdated { dispatch } => Some(dispatch),
                _ => None,
            });
            assert!(matches!(dispatch, Some(item) if
                item.outcome == DispatchOutcome::Pending
                    && item.sent_at.is_none()
                    && item.rendered_payload == "<user-input>\nhello\n</user-input>"));

            let runs = store.list_runs(workspace_id).await.expect("runs load");
            assert_eq!(runs.len(), 1);
            let persisted = store
                .list_dispatches(runs[0].id)
                .await
                .expect("dispatches load");
            assert_eq!(persisted.len(), 1);
            assert_eq!(persisted[0].outcome, DispatchOutcome::Pending);
        });
    }

    #[test]
    fn kill_switch_blocks_auto_send_preparation_without_persisting_a_run() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            store
                .save_workspace(workspace(workspace_id, BTreeSet::from([ProviderId::Gpt])))
                .await
                .expect("workspace saves");
            store
                .save_settings(SettingsState {
                    kill_switch_active: true,
                    ..SettingsState::default()
                })
                .await
                .expect("settings save");

            let result = coordinator
                .handle_ui_command(UiCommand::SendManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Gpt],
                    text: "must not send".to_owned(),
                    approval_mode: ApprovalMode::AutoSend,
                    selected_message_ids: BTreeSet::new(),
                    pinned_note: None,
                    target_notes: BTreeMap::new(),
                    include_target_prior_turns: false,
                    payload_overrides: BTreeMap::new(),
                    parent_message_id: None,
                })
                .await;

            assert!(matches!(result, Err(StorageError::Invariant(detail)) if
                detail.contains("kill switch is active")));
            assert!(
                store
                    .list_runs(workspace_id)
                    .await
                    .expect("runs load")
                    .is_empty()
            );
            assert!(
                store
                    .list_messages(workspace_id)
                    .await
                    .expect("messages load")
                    .is_empty()
            );
        });
    }

    #[test]
    fn delivered_ack_is_idempotent_and_is_the_only_cursor_advance() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            store
                .save_workspace(workspace(workspace_id, BTreeSet::from([ProviderId::Gpt])))
                .await
                .expect("workspace saves");

            let prepared = coordinator
                .handle_ui_command(UiCommand::SendManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Gpt],
                    text: "ack me".to_owned(),
                    approval_mode: ApprovalMode::AutoSend,
                    selected_message_ids: BTreeSet::new(),
                    pinned_note: None,
                    target_notes: BTreeMap::new(),
                    include_target_prior_turns: false,
                    payload_overrides: BTreeMap::new(),
                    parent_message_id: None,
                })
                .await
                .expect("manual send prepares");
            let dispatch = prepared
                .iter()
                .find_map(|event| match event {
                    UiEvent::DispatchUpdated { dispatch } => Some(dispatch.clone()),
                    _ => None,
                })
                .expect("pending dispatch event");
            assert!(
                store
                    .list_cursors(workspace_id)
                    .await
                    .expect("cursors load")
                    .is_empty()
            );

            coordinator
                .handle_ui_command(UiCommand::AcknowledgeDispatchDelivered {
                    dispatch_id: dispatch.id,
                })
                .await
                .expect("delivery ack succeeds");

            let delivered = store
                .list_dispatches(dispatch.run_id)
                .await
                .expect("dispatches load");
            assert!(matches!(delivered.as_slice(), [item] if
                item.outcome == DispatchOutcome::Delivered
                    && item.sent_at.is_some()
                    && item.rendered_payload == dispatch.rendered_payload));
            let first_cursors = store
                .list_cursors(workspace_id)
                .await
                .expect("cursors load");
            let user_message_id = dispatch.source_message_ids[0];
            assert!(matches!(first_cursors.as_slice(), [cursor] if
                cursor.source_participant_id == ProviderId::User
                    && cursor.target_participant_id == ProviderId::Gpt
                    && cursor.last_delivered_message_id == Some(user_message_id)));

            coordinator
                .handle_ui_command(UiCommand::AcknowledgeDispatchDelivered {
                    dispatch_id: dispatch.id,
                })
                .await
                .expect("repeated delivery ack is idempotent");
            assert_eq!(
                store
                    .list_cursors(workspace_id)
                    .await
                    .expect("cursors reload"),
                first_cursors
            );
        });
    }

    #[test]
    fn failed_ack_is_idempotent_and_never_advances_cursor() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            store
                .save_workspace(workspace(workspace_id, BTreeSet::from([ProviderId::Gpt])))
                .await
                .expect("workspace saves");
            let prepared = coordinator
                .handle_ui_command(UiCommand::SendManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Gpt],
                    text: "fail me".to_owned(),
                    approval_mode: ApprovalMode::AutoSend,
                    selected_message_ids: BTreeSet::new(),
                    pinned_note: None,
                    target_notes: BTreeMap::new(),
                    include_target_prior_turns: false,
                    payload_overrides: BTreeMap::new(),
                    parent_message_id: None,
                })
                .await
                .expect("manual send prepares");
            let dispatch = prepared
                .iter()
                .find_map(|event| match event {
                    UiEvent::DispatchUpdated { dispatch } => Some(dispatch.clone()),
                    _ => None,
                })
                .expect("pending dispatch event");

            for _ in 0..2 {
                coordinator
                    .handle_ui_command(UiCommand::AcknowledgeDispatchFailed {
                        dispatch_id: dispatch.id,
                        detail: "input element disappeared".to_owned(),
                    })
                    .await
                    .expect("failure ack is idempotent");
            }

            let failed = store
                .list_dispatches(dispatch.run_id)
                .await
                .expect("dispatches load");
            assert!(matches!(failed.as_slice(), [item] if
                item.outcome == DispatchOutcome::Error
                    && item.error_detail.as_deref() == Some("input element disappeared")
                    && item.retry_count == 1
                    && item.sent_at.is_none()));
            assert!(
                store
                    .list_cursors(workspace_id)
                    .await
                    .expect("cursors load")
                    .is_empty()
            );
        });
    }

    #[test]
    fn captured_ack_links_messages_to_delivered_dispatch_idempotently() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            store
                .save_workspace(workspace(workspace_id, BTreeSet::from([ProviderId::Gpt])))
                .await
                .expect("workspace saves");
            let prepared = coordinator
                .handle_ui_command(UiCommand::SendManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Gpt],
                    text: "respond".to_owned(),
                    approval_mode: ApprovalMode::AutoSend,
                    selected_message_ids: BTreeSet::new(),
                    pinned_note: None,
                    target_notes: BTreeMap::new(),
                    include_target_prior_turns: false,
                    payload_overrides: BTreeMap::new(),
                    parent_message_id: None,
                })
                .await
                .expect("manual send prepares");
            let dispatch = prepared
                .iter()
                .find_map(|event| match event {
                    UiEvent::DispatchUpdated { dispatch } => Some(dispatch.clone()),
                    _ => None,
                })
                .expect("pending dispatch event");
            coordinator
                .handle_ui_command(UiCommand::AcknowledgeDispatchDelivered {
                    dispatch_id: dispatch.id,
                })
                .await
                .expect("delivery ack succeeds");

            let response = Message {
                id: MessageId::new(),
                workspace_id: WorkspaceId::new(),
                participant_id: ProviderId::Gpt,
                role: MessageRole::Assistant,
                round: None,
                parent_message_id: None,
                child_message_ids: Vec::new(),
                branch_index: None,
                timestamp: Utc::now(),
                body_text: "provider response".to_owned(),
                body_blocks: vec![Block::Paragraph {
                    text: "provider response".to_owned(),
                }],
                source_binding_id: None,
                dispatch_id: None,
                raw_response_text: Some("raw provider response".to_owned()),
                network_capture: None,
                tags: Vec::new(),
                capture_confidence: CaptureConfidence::Certain,
            };

            for _ in 0..2 {
                coordinator
                    .handle_ui_command(UiCommand::AcknowledgeDispatchCaptured {
                        dispatch_id: dispatch.id,
                        messages: vec![response.clone()],
                    })
                    .await
                    .expect("capture ack is idempotent");
            }

            let messages = store
                .list_messages(workspace_id)
                .await
                .expect("messages load");
            let linked = messages
                .iter()
                .find(|message| message.id == response.id)
                .expect("captured response persists");
            assert_eq!(linked.workspace_id, workspace_id);
            assert_eq!(linked.dispatch_id, Some(dispatch.id));
            assert_eq!(linked.round, Some(1));
            assert_eq!(messages.len(), 2);
            let dispatches = store
                .list_dispatches(dispatch.run_id)
                .await
                .expect("dispatches load");
            assert!(matches!(dispatches.as_slice(), [item] if item.captured_at.is_some()));
        });
    }

    #[test]
    fn adapter_health_reports_are_persisted_on_binding() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();

            coordinator
                .ingest_adapter_event(
                    workspace_id,
                    AdapterToBackground::HealthReport {
                        provider: ProviderId::Gpt,
                        health: ProviderHealth::RateLimited,
                    },
                )
                .await
                .expect("health report succeeds");

            let bindings = store
                .list_bindings(workspace_id)
                .await
                .expect("bindings load");
            assert!(matches!(bindings.as_slice(), [binding] if
                binding.provider_id == ProviderId::Gpt
                    && binding.health_state == ProviderHealth::RateLimited));
        });
    }

    #[test]
    fn adapter_message_envelope_rejects_mismatched_provider_identity() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            let mismatched = Message {
                id: MessageId::new(),
                workspace_id,
                participant_id: ProviderId::Claude,
                role: MessageRole::Assistant,
                round: None,
                parent_message_id: None,
                child_message_ids: Vec::new(),
                branch_index: None,
                timestamp: Utc::now(),
                body_text: "wrong provider".to_owned(),
                body_blocks: Vec::new(),
                source_binding_id: None,
                dispatch_id: None,
                raw_response_text: None,
                network_capture: None,
                tags: Vec::new(),
                capture_confidence: CaptureConfidence::Certain,
            };

            let result = coordinator
                .ingest_adapter_event(
                    workspace_id,
                    AdapterToBackground::MessagesCaptured {
                        provider: ProviderId::Gpt,
                        messages: vec![mismatched],
                    },
                )
                .await;

            assert!(matches!(result, Err(StorageError::Invariant(detail)) if
                detail.contains("message identity does not match")));
            assert!(
                store
                    .list_messages(workspace_id)
                    .await
                    .expect("messages load")
                    .is_empty()
            );
        });
    }

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
                parent_message_id: None,
                child_message_ids: Vec::new(),
                branch_index: None,
                timestamp: Utc::now(),
                body_text: "source".to_owned(),
                body_blocks: vec![Block::Paragraph {
                    text: "source".to_owned(),
                }],
                source_binding_id: None,
                dispatch_id: None,
                raw_response_text: None,
                network_capture: None,
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

    #[test]
    fn synthesized_dispatch_is_pending_and_does_not_advance_cursor() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            let run = Run {
                id: chatmux_common::RunId::new(),
                workspace_id,
                mode: OrchestrationMode::Directed,
                graph_snapshot: RoutingGraph {
                    nodes: BTreeSet::from([ProviderId::Gpt, ProviderId::Claude]),
                    edges: vec![RouteEdge {
                        source: ProviderId::Gpt,
                        target: ProviderId::Claude,
                        policy_id: None,
                    }],
                },
                participant_set: BTreeSet::from([ProviderId::Gpt, ProviderId::Claude]),
                barrier_policy: BarrierPolicy::WaitForAll,
                timing_policy: chatmux_common::TimingPolicy::default(),
                stop_policy: StopPolicy::default(),
                status: RunStatus::Running,
                started_at: Some(Utc::now()),
                ended_at: None,
            };
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
            let policy = EdgePolicy {
                id: chatmux_common::EdgePolicyId::new(),
                workspace_id,
                source_participant_id: ProviderId::Gpt,
                target_participant_id: ProviderId::Claude,
                enabled: true,
                catch_up_policy: chatmux_common::CatchUpPolicy::FullHistory,
                incremental_policy: chatmux_common::IncrementalPolicy::UnseenDeltaOnly,
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
            };
            let message = Message {
                id: MessageId::new(),
                workspace_id,
                participant_id: ProviderId::Gpt,
                role: MessageRole::Assistant,
                round: Some(1),
                parent_message_id: None,
                child_message_ids: Vec::new(),
                branch_index: None,
                timestamp: Utc::now(),
                body_text: "source".to_owned(),
                body_blocks: vec![Block::Paragraph {
                    text: "source".to_owned(),
                }],
                source_binding_id: None,
                dispatch_id: None,
                raw_response_text: None,
                network_capture: None,
                tags: Vec::new(),
                capture_confidence: CaptureConfidence::Certain,
            };
            store.save_run(run.clone()).await.expect("run saves");
            store
                .save_round(Round {
                    id: chatmux_common::RoundId::new(),
                    run_id: run.id,
                    round_number: 1,
                    started_at: Some(Utc::now()),
                    completed_at: None,
                    status: RoundStatus::Running,
                })
                .await
                .expect("round saves");

            let dispatches = coordinator
                .synthesize_dispatches(
                    &run,
                    std::slice::from_ref(&policy),
                    std::slice::from_ref(&message),
                    std::slice::from_ref(&template),
                )
                .await
                .expect("dispatch synthesis succeeds");

            assert!(matches!(dispatches.as_slice(), [dispatch] if
                dispatch.outcome == DispatchOutcome::Pending && dispatch.sent_at.is_none()));
            assert!(
                store
                    .list_cursors(workspace_id)
                    .await
                    .expect("cursors load")
                    .is_empty()
            );

            let repeated = coordinator
                .synthesize_dispatches(
                    &run,
                    std::slice::from_ref(&policy),
                    std::slice::from_ref(&message),
                    std::slice::from_ref(&template),
                )
                .await
                .expect("repeated synthesis succeeds");
            assert_eq!(repeated[0].id, dispatches[0].id);
            assert_eq!(
                store
                    .list_dispatches(run.id)
                    .await
                    .expect("dispatches load")
                    .len(),
                1
            );

            store
                .save_settings(SettingsState {
                    kill_switch_active: true,
                    ..SettingsState::default()
                })
                .await
                .expect("settings save");
            let blocked = coordinator
                .synthesize_dispatches(&run, &[policy], &[message], &[template])
                .await;
            assert!(matches!(blocked, Err(StorageError::Invariant(detail)) if
                detail.contains("kill switch is active")));
        });
    }

    #[test]
    fn create_workspace_returns_list_and_snapshot() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store);

            let events = coordinator
                .handle_ui_command(UiCommand::CreateWorkspace {
                    name: "Workspace 1".to_owned(),
                })
                .await
                .expect("workspace creation succeeds");

            assert!(
                events
                    .iter()
                    .any(|event| matches!(event, UiEvent::WorkspaceList { .. })),
                "workspace creation should refresh the workspace list"
            );

            let snapshot = events.iter().find_map(|event| match event {
                UiEvent::WorkspaceSnapshot { snapshot } => Some(snapshot),
                _ => None,
            });
            assert!(
                snapshot
                    .and_then(|snapshot| snapshot.workspace.as_ref())
                    .is_some(),
                "workspace creation should return the created workspace snapshot"
            );
            assert_eq!(
                snapshot.map(|snapshot| snapshot.edge_policies.len()),
                Some(16),
                "a new workspace should expose every user/provider and provider/provider route before its first run"
            );
        });
    }

    #[test]
    fn clear_workspace_data_removes_history_and_keeps_the_workspace_and_its_configuration() {
        // These two commands were once a single match arm, so "Clear workspace
        // data" silently destroyed the workspace, its bindings and its routing
        // graph while reporting that data had been cleared. The two halves of
        // this test are what stop them collapsing back together.
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());

            let created = coordinator
                .handle_ui_command(UiCommand::CreateWorkspace {
                    name: "Retained".to_owned(),
                })
                .await
                .expect("workspace creation succeeds");
            let workspace_id = created
                .iter()
                .find_map(|event| match event {
                    UiEvent::WorkspaceSnapshot { snapshot } => {
                        snapshot.workspace.as_ref().map(|workspace| workspace.id)
                    }
                    _ => None,
                })
                .expect("creation returns the new workspace");

            store
                .save_message(assistant_message(
                    workspace_id,
                    ProviderId::Gpt,
                    "transcript body",
                ))
                .await
                .expect("message saves");

            let policies_before = store
                .list_edge_policies(workspace_id)
                .await
                .expect("policies list")
                .len();
            let templates_before = store
                .list_templates(workspace_id)
                .await
                .expect("templates list")
                .len();
            assert!(
                policies_before > 0 && templates_before > 0,
                "fixture must start with configuration worth preserving"
            );

            coordinator
                .handle_ui_command(UiCommand::ClearWorkspaceData { workspace_id })
                .await
                .expect("clearing workspace data succeeds");

            assert!(
                store
                    .get_workspace(workspace_id)
                    .await
                    .expect("workspace lookup")
                    .is_some(),
                "clearing history must not delete the workspace itself"
            );
            assert!(
                store
                    .list_messages(workspace_id)
                    .await
                    .expect("messages list")
                    .is_empty(),
                "clearing history must remove the transcript"
            );
            assert_eq!(
                store
                    .list_edge_policies(workspace_id)
                    .await
                    .expect("policies list")
                    .len(),
                policies_before,
                "clearing history must preserve the routing graph"
            );
            assert_eq!(
                store
                    .list_templates(workspace_id)
                    .await
                    .expect("templates list")
                    .len(),
                templates_before,
                "clearing history must preserve templates"
            );

            // The destructive sibling still removes everything.
            coordinator
                .handle_ui_command(UiCommand::DeleteWorkspace { workspace_id })
                .await
                .expect("deleting the workspace succeeds");
            assert!(
                store
                    .get_workspace(workspace_id)
                    .await
                    .expect("workspace lookup")
                    .is_none(),
                "deleting a workspace must remove the workspace record"
            );
            assert!(
                store
                    .list_edge_policies(workspace_id)
                    .await
                    .expect("policies list")
                    .is_empty(),
                "deleting a workspace must remove its routing graph"
            );
        });
    }

    #[test]
    fn send_manual_message_links_requested_parent_and_child() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();

            store
                .save_workspace(Workspace {
                    id: workspace_id,
                    name: "Workspace".to_owned(),
                    archived: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    enabled_providers: BTreeSet::from([ProviderId::Gpt]),
                    default_mode: OrchestrationMode::Broadcast,
                    default_context_strategy: ContextStrategy::WorkspaceDefault,
                    default_template_id: None,
                    active_export_profile_ids: Vec::new(),
                    tags: Vec::new(),
                    notes: None,
                })
                .await
                .expect("workspace saves");

            let parent = Message {
                id: MessageId::new(),
                workspace_id,
                participant_id: ProviderId::Gpt,
                role: MessageRole::Assistant,
                round: Some(1),
                parent_message_id: None,
                child_message_ids: Vec::new(),
                branch_index: None,
                timestamp: Utc::now(),
                body_text: "parent".to_owned(),
                body_blocks: vec![Block::Paragraph {
                    text: "parent".to_owned(),
                }],
                source_binding_id: None,
                dispatch_id: None,
                raw_response_text: None,
                network_capture: None,
                tags: Vec::new(),
                capture_confidence: CaptureConfidence::Certain,
            };
            store
                .save_message(parent.clone())
                .await
                .expect("parent saves");

            let events = coordinator
                .handle_ui_command(UiCommand::SendManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Gpt],
                    text: "child".to_owned(),
                    approval_mode: ApprovalMode::AutoSend,
                    selected_message_ids: BTreeSet::new(),
                    pinned_note: None,
                    target_notes: BTreeMap::new(),
                    include_target_prior_turns: false,
                    payload_overrides: BTreeMap::new(),
                    parent_message_id: Some(parent.id),
                })
                .await
                .expect("manual message succeeds");

            let messages = store
                .list_messages(workspace_id)
                .await
                .expect("messages load");
            let updated_parent = messages
                .iter()
                .find(|message| message.id == parent.id)
                .expect("parent remains stored");
            assert_eq!(updated_parent.child_message_ids.len(), 1);

            let child_id = updated_parent.child_message_ids[0];
            let child = messages
                .iter()
                .find(|message| message.id == child_id)
                .expect("child message stored");
            assert_eq!(child.parent_message_id, Some(parent.id));
            assert_eq!(child.branch_index, Some(1));

            assert!(events.iter().any(|event| matches!(
                event,
                UiEvent::MessageCaptured { message } if message.id == parent.id
                    && message.child_message_ids == updated_parent.child_message_ids
            )));
        });
    }

    #[test]
    fn send_manual_message_rejects_parent_from_another_workspace() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            let other_workspace_id = WorkspaceId::new();

            for id in [workspace_id, other_workspace_id] {
                store
                    .save_workspace(Workspace {
                        id,
                        name: "Workspace".to_owned(),
                        archived: false,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        enabled_providers: BTreeSet::from([ProviderId::Gpt]),
                        default_mode: OrchestrationMode::Broadcast,
                        default_context_strategy: ContextStrategy::WorkspaceDefault,
                        default_template_id: None,
                        active_export_profile_ids: Vec::new(),
                        tags: Vec::new(),
                        notes: None,
                    })
                    .await
                    .expect("workspace saves");
            }

            let parent = Message {
                id: MessageId::new(),
                workspace_id: other_workspace_id,
                participant_id: ProviderId::Gpt,
                role: MessageRole::Assistant,
                round: Some(1),
                parent_message_id: None,
                child_message_ids: Vec::new(),
                branch_index: None,
                timestamp: Utc::now(),
                body_text: "foreign parent".to_owned(),
                body_blocks: vec![Block::Paragraph {
                    text: "foreign parent".to_owned(),
                }],
                source_binding_id: None,
                dispatch_id: None,
                raw_response_text: None,
                network_capture: None,
                tags: Vec::new(),
                capture_confidence: CaptureConfidence::Certain,
            };
            store
                .save_message(parent.clone())
                .await
                .expect("parent saves");

            let result = coordinator
                .handle_ui_command(UiCommand::SendManualMessage {
                    workspace_id,
                    targets: vec![ProviderId::Gpt],
                    text: "child".to_owned(),
                    approval_mode: ApprovalMode::AutoSend,
                    selected_message_ids: BTreeSet::new(),
                    pinned_note: None,
                    target_notes: BTreeMap::new(),
                    include_target_prior_turns: false,
                    payload_overrides: BTreeMap::new(),
                    parent_message_id: Some(parent.id),
                })
                .await;

            assert!(matches!(result, Err(StorageError::Invariant(message)) if
                message == "parent message belongs to another workspace"));
        });
    }

    #[test]
    fn conversation_ref_promotes_provisional_binding_to_bound_target() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();

            store
                .save_workspace(Workspace {
                    id: workspace_id,
                    name: "Workspace".to_owned(),
                    archived: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    enabled_providers: BTreeSet::from([ProviderId::Gpt]),
                    default_mode: OrchestrationMode::Broadcast,
                    default_context_strategy: ContextStrategy::WorkspaceDefault,
                    default_template_id: None,
                    active_export_profile_ids: Vec::new(),
                    tags: Vec::new(),
                    notes: None,
                })
                .await
                .expect("workspace saves");

            coordinator
                .handle_ui_command(UiCommand::BindProviderTab {
                    workspace_id,
                    provider: ProviderId::Gpt,
                    tab_id: 42,
                    window_id: Some(7),
                    origin: Some("https://chatgpt.com".to_owned()),
                    tab_title: Some("ChatGPT".to_owned()),
                    tab_url: Some("https://chatgpt.com/".to_owned()),
                    conversation_id: None,
                    conversation_title: None,
                    conversation_url: None,
                    pin: true,
                })
                .await
                .expect("provisional bind succeeds");

            let binding = store
                .list_bindings(workspace_id)
                .await
                .expect("bindings load")
                .into_iter()
                .find(|binding| binding.provider_id == ProviderId::Gpt)
                .expect("binding exists");
            assert!(
                binding.bound_conversation_ref.is_none(),
                "provider-home binds should remain provisional until a chat identity is discovered"
            );

            coordinator
                .ingest_adapter_event(
                    workspace_id,
                    AdapterToBackground::ConversationRefDiscovered {
                        provider: ProviderId::Gpt,
                        conversation_ref: Some(chatmux_common::ConversationRef {
                            conversation_id: Some("chat-123".to_owned()),
                            title: Some("Chat 123".to_owned()),
                            url: Some("https://chatgpt.com/c/chat-123".to_owned()),
                            model_label: None,
                        }),
                    },
                )
                .await
                .expect("conversation ref event succeeds");

            let promoted = store
                .list_bindings(workspace_id)
                .await
                .expect("bindings reload")
                .into_iter()
                .find(|binding| binding.provider_id == ProviderId::Gpt)
                .expect("binding still exists");
            assert_eq!(
                promoted
                    .bound_conversation_ref
                    .as_ref()
                    .and_then(|item| item.conversation_id.clone())
                    .as_deref(),
                Some("chat-123")
            );
        });
    }

    #[test]
    fn rebinding_provider_clears_previous_send_failure() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();

            store
                .save_workspace(Workspace {
                    id: workspace_id,
                    name: "Workspace".to_owned(),
                    archived: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    enabled_providers: BTreeSet::from([ProviderId::Gpt]),
                    default_mode: OrchestrationMode::Broadcast,
                    default_context_strategy: ContextStrategy::WorkspaceDefault,
                    default_template_id: None,
                    active_export_profile_ids: Vec::new(),
                    tags: Vec::new(),
                    notes: None,
                })
                .await
                .expect("workspace saves");

            coordinator
                .persist_provider_health(workspace_id, ProviderId::Gpt, ProviderHealth::SendFailed)
                .await
                .expect("failure health persists");

            let events = coordinator
                .handle_ui_command(UiCommand::BindProviderTab {
                    workspace_id,
                    provider: ProviderId::Gpt,
                    tab_id: 42,
                    window_id: Some(7),
                    origin: Some("https://chatgpt.com".to_owned()),
                    tab_title: Some("ChatGPT".to_owned()),
                    tab_url: Some("https://chatgpt.com/".to_owned()),
                    conversation_id: None,
                    conversation_title: None,
                    conversation_url: None,
                    pin: true,
                })
                .await
                .expect("rebind succeeds");

            assert!(events.iter().any(|event| matches!(
                event,
                UiEvent::ProviderHealthChanged {
                    provider: ProviderId::Gpt,
                    health: ProviderHealth::Ready,
                    blocking_state: None,
                    ..
                }
            )));

            let binding = store
                .list_bindings(workspace_id)
                .await
                .expect("bindings load")
                .into_iter()
                .find(|binding| binding.provider_id == ProviderId::Gpt)
                .expect("binding exists");
            assert_eq!(binding.health_state, ProviderHealth::Ready);
        });
    }

    #[test]
    fn configured_roundtable_stops_after_max_round_barrier() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            let providers = BTreeSet::from([ProviderId::Gpt, ProviderId::Claude]);
            store
                .save_workspace(workspace(workspace_id, providers.clone()))
                .await
                .expect("workspace saves");
            let seed = Message {
                id: MessageId::new(),
                workspace_id,
                participant_id: ProviderId::User,
                role: MessageRole::User,
                round: None,
                parent_message_id: None,
                child_message_ids: Vec::new(),
                branch_index: None,
                timestamp: Utc::now(),
                body_text: "compare this".to_owned(),
                body_blocks: vec![Block::Paragraph {
                    text: "compare this".to_owned(),
                }],
                source_binding_id: None,
                dispatch_id: None,
                raw_response_text: None,
                network_capture: None,
                tags: Vec::new(),
                capture_confidence: CaptureConfidence::Certain,
            };
            store.save_message(seed).await.expect("seed saves");

            let events = coordinator
                .handle_ui_command(UiCommand::StartConfiguredRun {
                    workspace_id,
                    configuration: RunConfiguration {
                        mode: OrchestrationMode::Roundtable,
                        participants: providers,
                        timing_policy: chatmux_common::TimingPolicy {
                            max_rounds: Some(1),
                            ..chatmux_common::TimingPolicy::default()
                        },
                        ..RunConfiguration::default()
                    },
                })
                .await
                .expect("configured run starts");
            let dispatches = events
                .iter()
                .filter_map(|event| match event {
                    UiEvent::DispatchUpdated { dispatch } => Some(dispatch.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(dispatches.len(), 2);
            assert!(dispatches.iter().all(|dispatch| {
                dispatch.outcome == DispatchOutcome::Pending
                    && dispatch.rendered_payload.contains("compare this")
            }));

            for dispatch in &dispatches {
                coordinator
                    .handle_ui_command(UiCommand::AcknowledgeDispatchDelivered {
                        dispatch_id: dispatch.id,
                    })
                    .await
                    .expect("delivery acknowledged");
                coordinator
                    .handle_ui_command(UiCommand::AcknowledgeDispatchCaptured {
                        dispatch_id: dispatch.id,
                        messages: vec![Message {
                            id: MessageId::new(),
                            workspace_id: WorkspaceId::new(),
                            participant_id: dispatch.target_participant_id,
                            role: MessageRole::Assistant,
                            round: None,
                            parent_message_id: None,
                            child_message_ids: Vec::new(),
                            branch_index: None,
                            timestamp: Utc::now(),
                            body_text: format!(
                                "{} response",
                                dispatch.target_participant_id.display_name()
                            ),
                            body_blocks: Vec::new(),
                            source_binding_id: None,
                            dispatch_id: None,
                            raw_response_text: None,
                            network_capture: None,
                            tags: Vec::new(),
                            capture_confidence: CaptureConfidence::Certain,
                        }],
                    })
                    .await
                    .expect("capture acknowledged");
            }

            let run = store
                .list_runs(workspace_id)
                .await
                .expect("runs load")
                .into_iter()
                .next()
                .expect("run exists");
            assert_eq!(run.status, RunStatus::Completed);
            let rounds = store.list_rounds(run.id).await.expect("rounds load");
            assert!(matches!(rounds.as_slice(), [round] if round.status == RoundStatus::Completed));
        });
    }

    #[test]
    fn directed_run_packages_other_context_and_excludes_target_prior_assistant_turn() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            let providers = BTreeSet::from([
                ProviderId::Gpt,
                ProviderId::Gemini,
                ProviderId::Grok,
                ProviderId::Claude,
            ]);
            store
                .save_workspace(workspace(workspace_id, providers.clone()))
                .await
                .expect("workspace saves");
            for (provider, body) in [
                (ProviderId::User, "user question"),
                (ProviderId::Gpt, "GPT context"),
                (ProviderId::Gemini, "Gemini context"),
                (ProviderId::Grok, "Grok context"),
                (ProviderId::Claude, "Claude prior answer must be excluded"),
            ] {
                store
                    .save_message(Message {
                        id: MessageId::new(),
                        workspace_id,
                        participant_id: provider,
                        role: if provider == ProviderId::User {
                            MessageRole::User
                        } else {
                            MessageRole::Assistant
                        },
                        round: Some(1),
                        parent_message_id: None,
                        child_message_ids: Vec::new(),
                        branch_index: None,
                        timestamp: Utc::now(),
                        body_text: body.to_owned(),
                        body_blocks: vec![Block::Paragraph {
                            text: body.to_owned(),
                        }],
                        source_binding_id: None,
                        dispatch_id: None,
                        raw_response_text: None,
                        network_capture: None,
                        tags: Vec::new(),
                        capture_confidence: CaptureConfidence::Certain,
                    })
                    .await
                    .expect("message saves");
            }

            let events = coordinator
                .handle_ui_command(UiCommand::StartConfiguredRun {
                    workspace_id,
                    configuration: RunConfiguration {
                        mode: OrchestrationMode::Directed,
                        participants: providers,
                        moderator: Some(ProviderId::Claude),
                        ..RunConfiguration::default()
                    },
                })
                .await
                .expect("directed run starts");
            let dispatches = events
                .iter()
                .filter_map(|event| match event {
                    UiEvent::DispatchUpdated { dispatch } => Some(dispatch),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(dispatches.len(), 1);
            let payload = &dispatches[0].rendered_payload;
            assert_eq!(dispatches[0].target_participant_id, ProviderId::Claude);
            assert!(payload.contains("user question"));
            assert!(payload.contains("GPT context"));
            assert!(payload.contains("Gemini context"));
            assert!(payload.contains("Grok context"));
            assert!(!payload.contains("Claude prior answer must be excluded"));
        });
    }

    #[test]
    fn moderated_round_preview_edits_skip_and_user_injection_are_persisted_on_resume() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            let providers = BTreeSet::from([ProviderId::Gpt, ProviderId::Claude]);
            store
                .save_workspace(workspace(workspace_id, providers.clone()))
                .await
                .expect("workspace saves");
            store
                .save_message(Message {
                    id: MessageId::new(),
                    workspace_id,
                    participant_id: ProviderId::User,
                    role: MessageRole::User,
                    round: None,
                    parent_message_id: None,
                    child_message_ids: Vec::new(),
                    branch_index: None,
                    timestamp: Utc::now(),
                    body_text: "seed".to_owned(),
                    body_blocks: vec![Block::Paragraph {
                        text: "seed".to_owned(),
                    }],
                    source_binding_id: None,
                    dispatch_id: None,
                    raw_response_text: None,
                    network_capture: None,
                    tags: Vec::new(),
                    capture_confidence: CaptureConfidence::Certain,
                })
                .await
                .expect("seed saves");

            let events = coordinator
                .handle_ui_command(UiCommand::StartConfiguredRun {
                    workspace_id,
                    configuration: RunConfiguration {
                        mode: OrchestrationMode::Roundtable,
                        participants: providers,
                        timing_policy: chatmux_common::TimingPolicy {
                            max_rounds: Some(2),
                            ..chatmux_common::TimingPolicy::default()
                        },
                        require_review_between_rounds: true,
                        ..RunConfiguration::default()
                    },
                })
                .await
                .expect("configured run starts");
            let first_round = events
                .iter()
                .filter_map(|event| match event {
                    UiEvent::DispatchUpdated { dispatch } => Some(dispatch.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            for dispatch in &first_round {
                coordinator
                    .handle_ui_command(UiCommand::AcknowledgeDispatchDelivered {
                        dispatch_id: dispatch.id,
                    })
                    .await
                    .expect("delivery acknowledged");
                coordinator
                    .handle_ui_command(UiCommand::AcknowledgeDispatchCaptured {
                        dispatch_id: dispatch.id,
                        messages: vec![Message {
                            id: MessageId::new(),
                            workspace_id,
                            participant_id: dispatch.target_participant_id,
                            role: MessageRole::Assistant,
                            round: None,
                            parent_message_id: None,
                            child_message_ids: Vec::new(),
                            branch_index: None,
                            timestamp: Utc::now(),
                            body_text: format!(
                                "{} first answer",
                                dispatch.target_participant_id.display_name()
                            ),
                            body_blocks: Vec::new(),
                            source_binding_id: None,
                            dispatch_id: None,
                            raw_response_text: None,
                            network_capture: None,
                            tags: Vec::new(),
                            capture_confidence: CaptureConfidence::Certain,
                        }],
                    })
                    .await
                    .expect("capture acknowledged");
            }
            let run = store
                .list_runs(workspace_id)
                .await
                .expect("runs load")
                .into_iter()
                .next()
                .expect("run exists");
            assert_eq!(run.status, RunStatus::Paused);

            let preview = coordinator
                .handle_ui_command(UiCommand::PreviewNextRound { run_id: run.id })
                .await
                .expect("preview renders");
            assert!(preview.iter().any(|event| matches!(
                event,
                UiEvent::NextRoundPreview {
                    round_number: 2,
                    packages,
                    ..
                } if packages.len() == 2
            )));

            let resumed = coordinator
                .handle_ui_command(UiCommand::ResumeRunWithOverrides {
                    run_id: run.id,
                    payload_overrides: BTreeMap::from([(
                        ProviderId::Gpt,
                        "edited exact GPT payload".to_owned(),
                    )]),
                    skipped_targets: BTreeSet::from([ProviderId::Claude]),
                    injected_user_message: Some("focus on evidence".to_owned()),
                })
                .await
                .expect("reviewed run resumes");
            let second_round = resumed
                .iter()
                .filter_map(|event| match event {
                    UiEvent::DispatchUpdated { dispatch } => Some(dispatch),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert!(second_round.iter().any(|dispatch| {
                dispatch.target_participant_id == ProviderId::Gpt
                    && dispatch.rendered_payload == "edited exact GPT payload"
                    && dispatch.outcome == DispatchOutcome::Pending
            }));
            assert!(second_round.iter().any(|dispatch| {
                dispatch.target_participant_id == ProviderId::Claude
                    && dispatch.outcome == DispatchOutcome::Skipped
            }));
            assert!(
                store
                    .list_messages(workspace_id)
                    .await
                    .expect("messages load")
                    .iter()
                    .any(|message| {
                        message.participant_id == ProviderId::User
                            && message.body_text == "focus on evidence"
                            && message.round == Some(2)
                    })
            );
        });
    }

    #[test]
    fn restart_recovery_pauses_once_and_never_resends_pending_dispatch() {
        block_on(async {
            let store = InMemoryStateStore::default();
            let coordinator = BackgroundCoordinator::new(store.clone());
            let workspace_id = WorkspaceId::new();
            store
                .save_workspace(workspace(workspace_id, BTreeSet::from([ProviderId::Gpt])))
                .await
                .expect("workspace saves");
            let run = Run {
                id: chatmux_common::RunId::new(),
                workspace_id,
                mode: OrchestrationMode::Roundtable,
                graph_snapshot: RoutingGraph {
                    nodes: BTreeSet::from([ProviderId::User, ProviderId::Gpt]),
                    edges: Vec::new(),
                },
                participant_set: BTreeSet::from([ProviderId::Gpt]),
                barrier_policy: BarrierPolicy::WaitForAll,
                timing_policy: chatmux_common::TimingPolicy::default(),
                stop_policy: StopPolicy::default(),
                status: RunStatus::Running,
                started_at: Some(Utc::now()),
                ended_at: None,
            };
            store.save_run(run.clone()).await.expect("run saves");
            let dispatch = Dispatch {
                id: chatmux_common::DispatchId::new(),
                run_id: run.id,
                round_id: None,
                round_number: 1,
                target_participant_id: ProviderId::Gpt,
                source_message_ids: Vec::new(),
                template_id: None,
                rendered_payload: "pending exact payload".to_owned(),
                sent_at: None,
                captured_at: None,
                outcome: DispatchOutcome::Pending,
                error_detail: None,
                retry_count: 0,
            };
            store
                .save_dispatch(dispatch.clone())
                .await
                .expect("dispatch saves");

            assert_eq!(
                coordinator.recover_after_restart().await.expect("recovery"),
                1
            );
            assert_eq!(
                coordinator
                    .recover_after_restart()
                    .await
                    .expect("repeat recovery"),
                1
            );
            assert_eq!(
                store
                    .get_run(run.id)
                    .await
                    .expect("run loads")
                    .unwrap()
                    .status,
                RunStatus::Paused
            );
            let persisted = store
                .list_dispatches(run.id)
                .await
                .expect("dispatches load");
            assert!(matches!(persisted.as_slice(), [item] if item == &dispatch));
        });
    }
}
