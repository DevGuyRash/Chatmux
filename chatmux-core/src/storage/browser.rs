//! Browser-backed persistence using IndexedDB and extension `storage.local`.

use super::{SettingsState, StateStore, StorageError};
use async_trait::async_trait;
use chatmux_common::{
    BindingId, DeliveryCursor, DeliveryCursorId, DiagnosticEvent, DiagnosticEventId, Dispatch,
    DispatchId, EdgePolicy, EdgePolicyId, ExportProfile, ExportProfileId, Message, MessageId,
    ParticipantBinding, Round, RoundId, Run, RunId, Template, TemplateId, Workspace, WorkspaceId,
};
use js_sys::Array;
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

const DB_NAME: &str = "chatmux";
const SETTINGS_KEY: &str = "settings_state";
const STORE_WORKSPACES: &str = "workspaces";
const STORE_BINDINGS: &str = "bindings";
const STORE_MESSAGES: &str = "messages";
const STORE_RUNS: &str = "runs";
const STORE_ROUNDS: &str = "rounds";
const STORE_DISPATCHES: &str = "dispatches";
const STORE_CURSORS: &str = "cursors";
const STORE_EDGE_POLICIES: &str = "edge_policies";
const STORE_TEMPLATES: &str = "templates";
const STORE_EXPORT_PROFILES: &str = "export_profiles";
const STORE_DIAGNOSTICS: &str = "diagnostics";
const DIAGNOSTIC_EVENT_CAP: usize = 2500;

#[wasm_bindgen(module = "/src/storage/browser_bridge.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn idb_put(
        db_name: &str,
        store_name: &str,
        store_names: JsValue,
        value: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn idb_get_all(
        db_name: &str,
        store_name: &str,
        store_names: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn idb_get(
        db_name: &str,
        store_name: &str,
        store_names: JsValue,
        key: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn idb_delete(
        db_name: &str,
        store_name: &str,
        store_names: JsValue,
        key: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn storage_local_get(key: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn storage_local_set(key: &str, value: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Debug, Clone)]
pub struct BrowserStateStore {
    db_name: &'static str,
}

impl Default for BrowserStateStore {
    fn default() -> Self {
        Self { db_name: DB_NAME }
    }
}

impl BrowserStateStore {
    fn store_names() -> JsValue {
        Array::from_iter([
            JsValue::from_str(STORE_WORKSPACES),
            JsValue::from_str(STORE_BINDINGS),
            JsValue::from_str(STORE_MESSAGES),
            JsValue::from_str(STORE_RUNS),
            JsValue::from_str(STORE_ROUNDS),
            JsValue::from_str(STORE_DISPATCHES),
            JsValue::from_str(STORE_CURSORS),
            JsValue::from_str(STORE_EDGE_POLICIES),
            JsValue::from_str(STORE_TEMPLATES),
            JsValue::from_str(STORE_EXPORT_PROFILES),
            JsValue::from_str(STORE_DIAGNOSTICS),
        ])
        .into()
    }

    async fn save_entity<T: Serialize>(&self, store: &str, value: &T) -> Result<(), StorageError> {
        let value = serde_wasm_bindgen::to_value(value)
            .map_err(|error| StorageError::Invariant(error.to_string()))?;
        idb_put(self.db_name, store, Self::store_names(), value)
            .await
            .map_err(js_error)?;
        Ok(())
    }

    async fn list_entities<T: DeserializeOwned>(
        &self,
        store: &str,
    ) -> Result<Vec<T>, StorageError> {
        let value = idb_get_all(self.db_name, store, Self::store_names())
            .await
            .map_err(js_error)?;
        serde_wasm_bindgen::from_value(value)
            .map_err(|error| StorageError::Invariant(error.to_string()))
    }

    async fn get_entity<T: DeserializeOwned, K: Serialize>(
        &self,
        store: &str,
        key: &K,
    ) -> Result<Option<T>, StorageError> {
        let key = serde_wasm_bindgen::to_value(key)
            .map_err(|error| StorageError::Invariant(error.to_string()))?;
        let value = idb_get(self.db_name, store, Self::store_names(), key)
            .await
            .map_err(js_error)?;
        if value.is_undefined() || value.is_null() {
            Ok(None)
        } else {
            serde_wasm_bindgen::from_value(value)
                .map(Some)
                .map_err(|error| StorageError::Invariant(error.to_string()))
        }
    }

    async fn delete_entity<K: Serialize>(&self, store: &str, key: &K) -> Result<(), StorageError> {
        let key = serde_wasm_bindgen::to_value(key)
            .map_err(|error| StorageError::Invariant(error.to_string()))?;
        idb_delete(self.db_name, store, Self::store_names(), key)
            .await
            .map_err(js_error)?;
        Ok(())
    }
}

#[async_trait(?Send)]
impl StateStore for BrowserStateStore {
    async fn save_workspace(&self, workspace: Workspace) -> Result<(), StorageError> {
        self.save_entity(STORE_WORKSPACES, &workspace).await
    }

    async fn list_workspaces(&self) -> Result<Vec<Workspace>, StorageError> {
        self.list_entities(STORE_WORKSPACES).await
    }

    async fn get_workspace(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Option<Workspace>, StorageError> {
        self.get_entity(STORE_WORKSPACES, &workspace_id).await
    }

    async fn delete_workspace(&self, workspace_id: WorkspaceId) -> Result<(), StorageError> {
        self.delete_entity(STORE_WORKSPACES, &workspace_id).await?;

        for binding in self.list_bindings(workspace_id).await? {
            self.delete_entity(STORE_BINDINGS, &binding.id).await?;
        }
        for message in self.list_messages(workspace_id).await? {
            self.delete_entity(STORE_MESSAGES, &message.id).await?;
        }
        for cursor in self.list_cursors(workspace_id).await? {
            self.delete_entity(STORE_CURSORS, &cursor.id).await?;
        }
        for policy in self.list_edge_policies(workspace_id).await? {
            self.delete_entity(STORE_EDGE_POLICIES, &policy.id).await?;
        }
        for template in self.list_templates(workspace_id).await? {
            self.delete_entity(STORE_TEMPLATES, &template.id).await?;
        }
        for profile in self.list_export_profiles(workspace_id).await? {
            self.delete_entity(STORE_EXPORT_PROFILES, &profile.id)
                .await?;
        }
        for diagnostic in self.list_diagnostics(workspace_id).await? {
            self.delete_entity(STORE_DIAGNOSTICS, &diagnostic.id)
                .await?;
        }
        for run in self.list_runs(workspace_id).await? {
            for round in self.list_rounds(run.id).await? {
                self.delete_entity(STORE_ROUNDS, &round.id).await?;
            }
            for dispatch in self.list_dispatches(run.id).await? {
                self.delete_entity(STORE_DISPATCHES, &dispatch.id).await?;
            }
            self.delete_entity(STORE_RUNS, &run.id).await?;
        }
        Ok(())
    }

    async fn save_binding(&self, binding: ParticipantBinding) -> Result<(), StorageError> {
        self.save_entity(STORE_BINDINGS, &binding).await
    }

    async fn list_bindings(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<ParticipantBinding>, StorageError> {
        Ok(self
            .list_entities::<ParticipantBinding>(STORE_BINDINGS)
            .await?
            .into_iter()
            .filter(|binding| binding.workspace_id == workspace_id)
            .collect())
    }

    async fn save_message(&self, message: Message) -> Result<(), StorageError> {
        self.save_entity(STORE_MESSAGES, &message).await
    }

    async fn list_messages(&self, workspace_id: WorkspaceId) -> Result<Vec<Message>, StorageError> {
        let mut messages = self
            .list_entities::<Message>(STORE_MESSAGES)
            .await?
            .into_iter()
            .filter(|message| message.workspace_id == workspace_id)
            .collect::<Vec<_>>();
        messages.sort_by_key(|message| message.timestamp);
        Ok(messages)
    }

    async fn get_message(&self, message_id: MessageId) -> Result<Option<Message>, StorageError> {
        self.get_entity(STORE_MESSAGES, &message_id).await
    }

    async fn save_run(&self, run: Run) -> Result<(), StorageError> {
        self.save_entity(STORE_RUNS, &run).await
    }

    async fn get_run(&self, run_id: RunId) -> Result<Option<Run>, StorageError> {
        self.get_entity(STORE_RUNS, &run_id).await
    }

    async fn list_runs(&self, workspace_id: WorkspaceId) -> Result<Vec<Run>, StorageError> {
        Ok(self
            .list_entities::<Run>(STORE_RUNS)
            .await?
            .into_iter()
            .filter(|run| run.workspace_id == workspace_id)
            .collect())
    }

    async fn save_round(&self, round: Round) -> Result<(), StorageError> {
        self.save_entity(STORE_ROUNDS, &round).await
    }

    async fn list_rounds(&self, run_id: RunId) -> Result<Vec<Round>, StorageError> {
        Ok(self
            .list_entities::<Round>(STORE_ROUNDS)
            .await?
            .into_iter()
            .filter(|round| round.run_id == run_id)
            .collect())
    }

    async fn save_dispatch(&self, dispatch: Dispatch) -> Result<(), StorageError> {
        self.save_entity(STORE_DISPATCHES, &dispatch).await
    }

    async fn list_dispatches(&self, run_id: RunId) -> Result<Vec<Dispatch>, StorageError> {
        Ok(self
            .list_entities::<Dispatch>(STORE_DISPATCHES)
            .await?
            .into_iter()
            .filter(|dispatch| dispatch.run_id == run_id)
            .collect())
    }

    async fn save_cursor(&self, cursor: DeliveryCursor) -> Result<(), StorageError> {
        self.save_entity(STORE_CURSORS, &cursor).await
    }

    async fn list_cursors(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<DeliveryCursor>, StorageError> {
        Ok(self
            .list_entities::<DeliveryCursor>(STORE_CURSORS)
            .await?
            .into_iter()
            .filter(|cursor| cursor.workspace_id == workspace_id)
            .collect())
    }

    async fn get_cursor(
        &self,
        cursor_id: DeliveryCursorId,
    ) -> Result<Option<DeliveryCursor>, StorageError> {
        self.get_entity(STORE_CURSORS, &cursor_id).await
    }

    async fn save_edge_policy(&self, policy: EdgePolicy) -> Result<(), StorageError> {
        self.save_entity(STORE_EDGE_POLICIES, &policy).await
    }

    async fn list_edge_policies(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<EdgePolicy>, StorageError> {
        Ok(self
            .list_entities::<EdgePolicy>(STORE_EDGE_POLICIES)
            .await?
            .into_iter()
            .filter(|policy| policy.workspace_id == workspace_id)
            .collect())
    }

    async fn save_template(&self, template: Template) -> Result<(), StorageError> {
        self.save_entity(STORE_TEMPLATES, &template).await
    }

    async fn list_templates(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<Template>, StorageError> {
        Ok(self
            .list_entities::<Template>(STORE_TEMPLATES)
            .await?
            .into_iter()
            .filter(|template| template.workspace_id == workspace_id)
            .collect())
    }

    async fn delete_template(&self, template_id: TemplateId) -> Result<(), StorageError> {
        self.delete_entity(STORE_TEMPLATES, &template_id).await
    }

    async fn save_export_profile(&self, profile: ExportProfile) -> Result<(), StorageError> {
        self.save_entity(STORE_EXPORT_PROFILES, &profile).await
    }

    async fn list_export_profiles(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<ExportProfile>, StorageError> {
        Ok(self
            .list_entities::<ExportProfile>(STORE_EXPORT_PROFILES)
            .await?
            .into_iter()
            .filter(|profile| profile.workspace_id == workspace_id)
            .collect())
    }

    async fn save_diagnostic(&self, diagnostic: DiagnosticEvent) -> Result<(), StorageError> {
        self.save_entity(STORE_DIAGNOSTICS, &diagnostic).await?;

        let mut diagnostics = self
            .list_entities::<DiagnosticEvent>(STORE_DIAGNOSTICS)
            .await?;
        diagnostics.sort_by_key(|event| event.timestamp);

        if diagnostics.len() > DIAGNOSTIC_EVENT_CAP {
            let overflow = diagnostics.len() - DIAGNOSTIC_EVENT_CAP;
            for event in diagnostics.into_iter().take(overflow) {
                self.delete_entity(STORE_DIAGNOSTICS, &event.id).await?;
            }
        }

        Ok(())
    }

    async fn delete_diagnostic(
        &self,
        diagnostic_id: DiagnosticEventId,
    ) -> Result<(), StorageError> {
        self.delete_entity(STORE_DIAGNOSTICS, &diagnostic_id).await
    }

    async fn list_diagnostics(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<DiagnosticEvent>, StorageError> {
        Ok(self
            .list_entities::<DiagnosticEvent>(STORE_DIAGNOSTICS)
            .await?
            .into_iter()
            .filter(|diagnostic| diagnostic.workspace_id == workspace_id)
            .collect())
    }

    async fn load_settings(&self) -> Result<SettingsState, StorageError> {
        let value = storage_local_get(SETTINGS_KEY).await.map_err(js_error)?;
        if value.is_undefined() || value.is_null() {
            Ok(SettingsState::default())
        } else {
            serde_wasm_bindgen::from_value(value)
                .map_err(|error| StorageError::Invariant(error.to_string()))
        }
    }

    async fn save_settings(&self, settings: SettingsState) -> Result<(), StorageError> {
        let value = serde_wasm_bindgen::to_value(&settings)
            .map_err(|error| StorageError::Invariant(error.to_string()))?;
        storage_local_set(SETTINGS_KEY, value)
            .await
            .map_err(js_error)?;
        Ok(())
    }
}

fn js_error(error: JsValue) -> StorageError {
    let detail = error
        .as_string()
        .or_else(|| {
            error
                .dyn_ref::<js_sys::Error>()
                .map(|item| item.message().into())
        })
        .unwrap_or_else(|| format!("{error:?}"));
    StorageError::BrowserUnavailable(detail)
}

#[allow(dead_code)]
fn _keep_ids_referenced(
    _: BindingId,
    _: DeliveryCursorId,
    _: DiagnosticEventId,
    _: DispatchId,
    _: EdgePolicyId,
    _: ExportProfileId,
    _: RoundId,
    _: TemplateId,
) {
}
