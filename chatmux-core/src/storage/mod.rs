//! Storage abstractions for heavy IndexedDB-backed state and lightweight settings.

use async_trait::async_trait;
use chatmux_common::{
    DeliveryCursor, DeliveryCursorId, DiagnosticEvent, Dispatch, DispatchId, EdgePolicy,
    EdgePolicyId, ExportProfile, ExportProfileId, Message, MessageId, ParticipantBinding,
    ProviderControlDefaults, ProviderId, Round, RoundId, Run, RunId, Template, TemplateId,
    Workspace, WorkspaceId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
mod browser;

#[cfg(target_arch = "wasm32")]
pub use browser::BrowserStateStore;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResumeMarker {
    pub workspace_id: WorkspaceId,
    pub paused_run_id: Option<RunId>,
    pub last_seen_dispatch_id: Option<DispatchId>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsState {
    pub preferred_surface: Option<String>,
    pub enabled_workspace_ids: Vec<WorkspaceId>,
    pub resume_markers: Vec<ResumeMarker>,
    pub kill_switch_active: bool,
    pub provider_defaults: BTreeMap<ProviderId, ProviderControlDefaults>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum StorageError {
    #[error("entity not found: {0}")]
    NotFound(String),
    #[error("storage invariant failed: {0}")]
    Invariant(String),
    #[error("browser storage unavailable: {0}")]
    BrowserUnavailable(String),
}

#[async_trait(?Send)]
pub trait StateStore {
    async fn save_workspace(&self, workspace: Workspace) -> Result<(), StorageError>;
    async fn list_workspaces(&self) -> Result<Vec<Workspace>, StorageError>;
    async fn get_workspace(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Option<Workspace>, StorageError>;
    async fn delete_workspace(&self, workspace_id: WorkspaceId) -> Result<(), StorageError>;

    async fn save_binding(&self, binding: ParticipantBinding) -> Result<(), StorageError>;
    async fn list_bindings(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<ParticipantBinding>, StorageError>;

    async fn save_message(&self, message: Message) -> Result<(), StorageError>;
    async fn list_messages(&self, workspace_id: WorkspaceId) -> Result<Vec<Message>, StorageError>;
    async fn get_message(&self, message_id: MessageId) -> Result<Option<Message>, StorageError>;

    async fn save_run(&self, run: Run) -> Result<(), StorageError>;
    async fn get_run(&self, run_id: RunId) -> Result<Option<Run>, StorageError>;
    async fn list_runs(&self, workspace_id: WorkspaceId) -> Result<Vec<Run>, StorageError>;

    async fn save_round(&self, round: Round) -> Result<(), StorageError>;
    async fn list_rounds(&self, run_id: RunId) -> Result<Vec<Round>, StorageError>;

    async fn save_dispatch(&self, dispatch: Dispatch) -> Result<(), StorageError>;
    async fn list_dispatches(&self, run_id: RunId) -> Result<Vec<Dispatch>, StorageError>;

    async fn save_cursor(&self, cursor: DeliveryCursor) -> Result<(), StorageError>;
    async fn list_cursors(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<DeliveryCursor>, StorageError>;
    async fn get_cursor(
        &self,
        cursor_id: DeliveryCursorId,
    ) -> Result<Option<DeliveryCursor>, StorageError>;

    async fn save_edge_policy(&self, policy: EdgePolicy) -> Result<(), StorageError>;
    async fn list_edge_policies(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<EdgePolicy>, StorageError>;

    async fn save_template(&self, template: Template) -> Result<(), StorageError>;
    async fn list_templates(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<Template>, StorageError>;
    async fn delete_template(&self, template_id: TemplateId) -> Result<(), StorageError>;

    async fn save_export_profile(&self, profile: ExportProfile) -> Result<(), StorageError>;
    async fn list_export_profiles(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<ExportProfile>, StorageError>;

    async fn save_diagnostic(&self, diagnostic: DiagnosticEvent) -> Result<(), StorageError>;
    async fn delete_diagnostic(
        &self,
        diagnostic_id: chatmux_common::DiagnosticEventId,
    ) -> Result<(), StorageError>;
    async fn list_diagnostics(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<DiagnosticEvent>, StorageError>;

    async fn load_settings(&self) -> Result<SettingsState, StorageError>;
    async fn save_settings(&self, settings: SettingsState) -> Result<(), StorageError>;
}

#[cfg(target_arch = "wasm32")]
pub type RuntimeStateStore = BrowserStateStore;

#[cfg(not(target_arch = "wasm32"))]
pub type RuntimeStateStore = InMemoryStateStore;

#[derive(Debug, Clone, Default)]
pub struct InMemoryStateStore {
    inner: Arc<Mutex<InnerStore>>,
}

#[derive(Debug, Default)]
struct InnerStore {
    workspaces: BTreeMap<WorkspaceId, Workspace>,
    bindings: BTreeMap<WorkspaceId, BTreeMap<chatmux_common::BindingId, ParticipantBinding>>,
    messages: BTreeMap<WorkspaceId, BTreeMap<MessageId, Message>>,
    runs: BTreeMap<RunId, Run>,
    rounds: BTreeMap<RunId, BTreeMap<RoundId, Round>>,
    dispatches: BTreeMap<RunId, BTreeMap<DispatchId, Dispatch>>,
    cursors: BTreeMap<WorkspaceId, BTreeMap<DeliveryCursorId, DeliveryCursor>>,
    edge_policies: BTreeMap<WorkspaceId, BTreeMap<EdgePolicyId, EdgePolicy>>,
    templates: BTreeMap<WorkspaceId, BTreeMap<TemplateId, Template>>,
    export_profiles: BTreeMap<WorkspaceId, BTreeMap<ExportProfileId, ExportProfile>>,
    diagnostics:
        BTreeMap<WorkspaceId, BTreeMap<chatmux_common::DiagnosticEventId, DiagnosticEvent>>,
    settings: SettingsState,
}

#[async_trait(?Send)]
impl StateStore for InMemoryStateStore {
    async fn save_workspace(&self, workspace: Workspace) -> Result<(), StorageError> {
        self.inner
            .lock()
            .expect("memory store poisoned")
            .workspaces
            .insert(workspace.id, workspace);
        Ok(())
    }

    async fn list_workspaces(&self) -> Result<Vec<Workspace>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .workspaces
            .values()
            .cloned()
            .collect())
    }

    async fn get_workspace(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Option<Workspace>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .workspaces
            .get(&workspace_id)
            .cloned())
    }

    async fn delete_workspace(&self, workspace_id: WorkspaceId) -> Result<(), StorageError> {
        let mut inner = self.inner.lock().expect("memory store poisoned");
        inner.workspaces.remove(&workspace_id);
        inner.bindings.remove(&workspace_id);
        inner.messages.remove(&workspace_id);
        inner.cursors.remove(&workspace_id);
        inner.edge_policies.remove(&workspace_id);
        inner.templates.remove(&workspace_id);
        inner.export_profiles.remove(&workspace_id);
        inner.diagnostics.remove(&workspace_id);

        let run_ids = inner
            .runs
            .iter()
            .filter_map(|(run_id, run)| (run.workspace_id == workspace_id).then_some(*run_id))
            .collect::<Vec<_>>();
        for run_id in run_ids {
            inner.runs.remove(&run_id);
            inner.rounds.remove(&run_id);
            inner.dispatches.remove(&run_id);
        }
        Ok(())
    }

    async fn save_binding(&self, binding: ParticipantBinding) -> Result<(), StorageError> {
        self.inner
            .lock()
            .expect("memory store poisoned")
            .bindings
            .entry(binding.workspace_id)
            .or_default()
            .insert(binding.id, binding);
        Ok(())
    }

    async fn list_bindings(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<ParticipantBinding>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .bindings
            .get(&workspace_id)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn save_message(&self, message: Message) -> Result<(), StorageError> {
        let mut inner = self.inner.lock().expect("memory store poisoned");
        let messages = inner.messages.entry(message.workspace_id).or_default();
        let mut message = message;
        if let Some(existing) = messages.get(&message.id) {
            message.timestamp = existing.timestamp;
        }
        messages.insert(message.id, message);
        Ok(())
    }

    async fn list_messages(&self, workspace_id: WorkspaceId) -> Result<Vec<Message>, StorageError> {
        let mut messages: Vec<_> = self
            .inner
            .lock()
            .expect("memory store poisoned")
            .messages
            .get(&workspace_id)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default();
        messages.sort_by_key(|message| message.timestamp);
        Ok(messages)
    }

    async fn get_message(&self, message_id: MessageId) -> Result<Option<Message>, StorageError> {
        let inner = self.inner.lock().expect("memory store poisoned");
        for messages in inner.messages.values() {
            if let Some(message) = messages.get(&message_id) {
                return Ok(Some(message.clone()));
            }
        }
        Ok(None)
    }

    async fn save_run(&self, run: Run) -> Result<(), StorageError> {
        self.inner
            .lock()
            .expect("memory store poisoned")
            .runs
            .insert(run.id, run);
        Ok(())
    }

    async fn get_run(&self, run_id: RunId) -> Result<Option<Run>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .runs
            .get(&run_id)
            .cloned())
    }

    async fn list_runs(&self, workspace_id: WorkspaceId) -> Result<Vec<Run>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .runs
            .values()
            .filter(|run| run.workspace_id == workspace_id)
            .cloned()
            .collect())
    }

    async fn save_round(&self, round: Round) -> Result<(), StorageError> {
        self.inner
            .lock()
            .expect("memory store poisoned")
            .rounds
            .entry(round.run_id)
            .or_default()
            .insert(round.id, round);
        Ok(())
    }

    async fn list_rounds(&self, run_id: RunId) -> Result<Vec<Round>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .rounds
            .get(&run_id)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn save_dispatch(&self, dispatch: Dispatch) -> Result<(), StorageError> {
        self.inner
            .lock()
            .expect("memory store poisoned")
            .dispatches
            .entry(dispatch.run_id)
            .or_default()
            .insert(dispatch.id, dispatch);
        Ok(())
    }

    async fn list_dispatches(&self, run_id: RunId) -> Result<Vec<Dispatch>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .dispatches
            .get(&run_id)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn save_cursor(&self, cursor: DeliveryCursor) -> Result<(), StorageError> {
        self.inner
            .lock()
            .expect("memory store poisoned")
            .cursors
            .entry(cursor.workspace_id)
            .or_default()
            .insert(cursor.id, cursor);
        Ok(())
    }

    async fn list_cursors(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<DeliveryCursor>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .cursors
            .get(&workspace_id)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn get_cursor(
        &self,
        cursor_id: DeliveryCursorId,
    ) -> Result<Option<DeliveryCursor>, StorageError> {
        let inner = self.inner.lock().expect("memory store poisoned");
        for cursors in inner.cursors.values() {
            if let Some(cursor) = cursors.get(&cursor_id) {
                return Ok(Some(cursor.clone()));
            }
        }
        Ok(None)
    }

    async fn save_edge_policy(&self, policy: EdgePolicy) -> Result<(), StorageError> {
        self.inner
            .lock()
            .expect("memory store poisoned")
            .edge_policies
            .entry(policy.workspace_id)
            .or_default()
            .insert(policy.id, policy);
        Ok(())
    }

    async fn list_edge_policies(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<EdgePolicy>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .edge_policies
            .get(&workspace_id)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn save_template(&self, template: Template) -> Result<(), StorageError> {
        self.inner
            .lock()
            .expect("memory store poisoned")
            .templates
            .entry(template.workspace_id)
            .or_default()
            .insert(template.id, template);
        Ok(())
    }

    async fn list_templates(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<Template>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .templates
            .get(&workspace_id)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn delete_template(&self, template_id: TemplateId) -> Result<(), StorageError> {
        let mut inner = self.inner.lock().expect("memory store poisoned");
        for templates in inner.templates.values_mut() {
            if templates.remove(&template_id).is_some() {
                return Ok(());
            }
        }
        Err(StorageError::NotFound(format!("template {template_id:?}")))
    }

    async fn save_export_profile(&self, profile: ExportProfile) -> Result<(), StorageError> {
        self.inner
            .lock()
            .expect("memory store poisoned")
            .export_profiles
            .entry(profile.workspace_id)
            .or_default()
            .insert(profile.id, profile);
        Ok(())
    }

    async fn list_export_profiles(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<ExportProfile>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .export_profiles
            .get(&workspace_id)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn save_diagnostic(&self, diagnostic: DiagnosticEvent) -> Result<(), StorageError> {
        const DIAGNOSTIC_EVENT_CAP: usize = 2500;

        let mut inner = self.inner.lock().expect("memory store poisoned");
        let workspace_id = diagnostic.workspace_id;
        inner
            .diagnostics
            .entry(workspace_id)
            .or_default()
            .insert(diagnostic.id, diagnostic);

        if let Some(items) = inner.diagnostics.get_mut(&workspace_id) {
            if items.len() > DIAGNOSTIC_EVENT_CAP {
                let mut ordered = items.values().cloned().collect::<Vec<_>>();
                ordered.sort_by_key(|event| event.timestamp);
                let overflow = ordered.len() - DIAGNOSTIC_EVENT_CAP;
                for event in ordered.into_iter().take(overflow) {
                    items.remove(&event.id);
                }
            }
        }
        Ok(())
    }

    async fn delete_diagnostic(
        &self,
        diagnostic_id: chatmux_common::DiagnosticEventId,
    ) -> Result<(), StorageError> {
        let mut inner = self.inner.lock().expect("memory store poisoned");
        for diagnostics in inner.diagnostics.values_mut() {
            diagnostics.remove(&diagnostic_id);
        }
        Ok(())
    }

    async fn list_diagnostics(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<DiagnosticEvent>, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .diagnostics
            .get(&workspace_id)
            .map(|items| items.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn load_settings(&self) -> Result<SettingsState, StorageError> {
        Ok(self
            .inner
            .lock()
            .expect("memory store poisoned")
            .settings
            .clone())
    }

    async fn save_settings(&self, settings: SettingsState) -> Result<(), StorageError> {
        self.inner.lock().expect("memory store poisoned").settings = settings;
        Ok(())
    }
}
