//! Shared UI event reduction and bridge error handling.

use leptos::prelude::{GetUntracked, Set, Update};

use crate::models::{DiagnosticLevel, Run, UiEvent};
use crate::state::{
    app_state::{AppState, ExportState, MessageInspectionState, ProviderRuntimeState},
    binding_state::BindingState,
    diagnostics_state::DiagnosticsState,
    message_state::MessageState,
    run_state::ActiveRunState,
    workspace_state::WorkspaceListState,
};

pub fn dispatch_command_result(
    app_state: AppState,
    workspace_state: WorkspaceListState,
    run_state: ActiveRunState,
    binding_state: BindingState,
    message_state: MessageState,
    diagnostics_state: DiagnosticsState,
    result: Result<Vec<UiEvent>, String>,
) {
    match result {
        Ok(events) => apply_events(
            app_state,
            workspace_state,
            run_state,
            binding_state,
            message_state,
            diagnostics_state,
            events,
        ),
        Err(error) => {
            app_state.set_last_error.set(Some(error));
            app_state.set_bridge_ready.set(false);
        }
    }
}

pub fn apply_events(
    app_state: AppState,
    workspace_state: WorkspaceListState,
    run_state: ActiveRunState,
    binding_state: BindingState,
    message_state: MessageState,
    diagnostics_state: DiagnosticsState,
    events: Vec<UiEvent>,
) {
    for event in events {
        apply_event(
            app_state,
            workspace_state,
            run_state,
            binding_state,
            message_state,
            diagnostics_state,
            event,
        );
    }
    app_state.set_last_error.set(None);
    app_state.set_bridge_ready.set(true);
}

fn apply_event(
    app_state: AppState,
    workspace_state: WorkspaceListState,
    run_state: ActiveRunState,
    binding_state: BindingState,
    message_state: MessageState,
    diagnostics_state: DiagnosticsState,
    event: UiEvent,
) {
    match event {
        UiEvent::WorkspaceList { workspaces } => {
            let last_active_workspace_id = app_state.ui_settings.get_untracked().last_active_workspace_id;
            workspace_state.set_workspaces.set(workspaces);
            if last_active_workspace_id.is_some_and(|workspace_id| {
                !workspace_state
                    .workspaces
                    .get_untracked()
                    .iter()
                    .any(|workspace| workspace.id == workspace_id)
            }) {
                app_state.set_ui_settings.update(|settings| {
                    settings.last_active_workspace_id = None;
                });
            }
        }
        UiEvent::WorkspaceSnapshot { snapshot } => {
            if let Some(workspace) = snapshot.workspace.clone() {
                app_state.set_active_workspace_id.set(Some(workspace.id));
                app_state.set_ui_settings.update(|settings| {
                    settings.last_active_workspace_id = Some(workspace.id);
                });
                workspace_state.set_workspaces.update(
                    |workspaces: &mut Vec<crate::models::Workspace>| {
                        if let Some(existing) =
                            workspaces.iter_mut().find(|item| item.id == workspace.id)
                        {
                            *existing = workspace.clone();
                        } else {
                            workspaces.push(workspace);
                        }
                    },
                );
            }

            binding_state.set_bindings.set(snapshot.bindings.clone());
            message_state
                .set_messages
                .set(snapshot.recent_messages.clone());
            diagnostics_state
                .set_events
                .set(snapshot.diagnostics.clone());
            diagnostics_state
                .set_summary
                .set(snapshot.diagnostics_summary.clone());
            diagnostics_state
                .set_unread_count
                .set(unread_count(&snapshot.diagnostics));
            run_state.set_run.set(select_run(&snapshot.runs));
            run_state.set_rounds.set(Vec::new());
            app_state
                .set_kill_switch_active
                .set(snapshot.kill_switch_active);
            app_state.set_provider_controls.update(|registry| {
                registry.snapshots.clear();
                registry.tab_candidates.clear();
                for snapshot in &snapshot.provider_controls {
                    registry
                        .snapshots
                        .insert(snapshot.provider, snapshot.clone());
                }
            });
            workspace_state.set_snapshot.set(Some(snapshot));
        }
        UiEvent::RunUpdated { run, rounds } => {
            run_state.set_run.set(Some(run));
            run_state.set_rounds.set(rounds);
        }
        UiEvent::MessageCaptured { message } => {
            message_state
                .set_messages
                .update(|messages: &mut Vec<crate::models::Message>| {
                    if let Some(existing) = messages.iter_mut().find(|item| item.id == message.id) {
                        *existing = message.clone();
                    } else {
                        messages.push(message.clone());
                        messages.sort_by_key(|item| item.timestamp);
                    }
                });
        }
        UiEvent::DispatchUpdated { .. } => {}
        UiEvent::DiagnosticRaised { diagnostic } => {
            diagnostics_state.set_events.update(
                |events: &mut Vec<crate::models::DiagnosticEvent>| {
                    if let Some(existing) = events.iter_mut().find(|item| item.id == diagnostic.id)
                    {
                        *existing = diagnostic.clone();
                    } else {
                        events.push(diagnostic);
                    }
                },
            );
            diagnostics_state.set_summary.set(summarize_diagnostics(
                &diagnostics_state.events.get_untracked(),
            ));
            diagnostics_state
                .set_unread_count
                .set(unread_count(&diagnostics_state.events.get_untracked()));
        }
        UiEvent::DiagnosticsSnapshot { snapshot } => {
            diagnostics_state.set_events.set(snapshot.events.clone());
            diagnostics_state.set_summary.set(snapshot.summary.clone());
            diagnostics_state
                .set_unread_count
                .set(unread_count(&snapshot.events));
        }
        UiEvent::ProviderHealthChanged {
            provider,
            health,
            blocking_state,
            ..
        } => {
            app_state.set_provider_health.update(
                |states: &mut std::collections::BTreeMap<
                    crate::models::ProviderId,
                    ProviderRuntimeState,
                >| {
                    states.insert(
                        provider,
                        ProviderRuntimeState {
                            health,
                            blocking_state,
                        },
                    );
                },
            );
        }
        UiEvent::ProviderControlUpdated { snapshot, .. } => {
            app_state.set_provider_controls.update(|registry| {
                registry.snapshots.insert(snapshot.provider, snapshot);
            });
        }
        UiEvent::ProviderTabCandidates {
            provider,
            candidates,
            ..
        } => {
            app_state.set_provider_controls.update(|registry| {
                registry.tab_candidates.insert(provider, candidates);
            });
        }
        UiEvent::ProviderDefaultsUpdated { provider, defaults } => {
            app_state.set_provider_controls.update(|registry| {
                registry.defaults.insert(provider, defaults);
            });
        }
        UiEvent::ExportRendered {
            format,
            mime_type,
            filename,
            body,
        } => {
            app_state.set_export.set(Some(ExportState {
                format,
                mime_type,
                filename,
                body,
            }));
        }
        UiEvent::MessageInspection {
            message,
            dispatch,
            sent_payload,
            raw_response_text,
            network_capture,
        } => {
            app_state.set_inspection.set(Some(MessageInspectionState {
                message,
                dispatch,
                sent_payload,
                raw_response_text,
                network_capture,
            }));
        }
        UiEvent::KillSwitchChanged { active } => {
            app_state.set_kill_switch_active.set(active);
        }
    }
}

fn unread_count(events: &[crate::models::DiagnosticEvent]) -> u32 {
    events
        .iter()
        .filter(|event| {
            matches!(
                event.level,
                DiagnosticLevel::Critical | DiagnosticLevel::Warning
            )
        })
        .count() as u32
}

fn summarize_diagnostics(
    events: &[crate::models::DiagnosticEvent],
) -> crate::models::WorkspaceDiagnosticsSummary {
    let mut summary = crate::models::WorkspaceDiagnosticsSummary {
        total: events.len() as u32,
        ..crate::models::WorkspaceDiagnosticsSummary::default()
    };

    for event in events {
        match event.level {
            crate::models::DiagnosticLevel::Critical => summary.critical += 1,
            crate::models::DiagnosticLevel::Warning => summary.warning += 1,
            crate::models::DiagnosticLevel::Info => summary.info += 1,
            crate::models::DiagnosticLevel::Debug => summary.debug += 1,
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

fn select_run(runs: &[Run]) -> Option<Run> {
    runs.iter()
        .find(|run| {
            matches!(
                run.status,
                crate::models::RunStatus::Running | crate::models::RunStatus::Paused
            )
        })
        .cloned()
        .or_else(|| runs.last().cloned())
}
