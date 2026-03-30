//! Canonical export model and renderers for Chatmux.

use chatmux_common::{
    DiagnosticEvent, Dispatch, ExportFormat, ExportLayout, ExportProfile, Message, ProviderId, Run,
    Workspace, WorkspaceSnapshot,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub title: String,
    pub exported_at: DateTime<Utc>,
    pub workspace_name: Option<String>,
    pub workspace_id: Option<String>,
    pub participant_labels: Vec<String>,
    pub orchestration_mode: Option<String>,
    pub run_id: Option<String>,
    pub round_range: Option<String>,
    pub message_count: usize,
    pub template_name: Option<String>,
    pub export_profile_name: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub additional: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportDocument {
    pub metadata: ExportMetadata,
    pub messages: Vec<Message>,
    pub dispatches: Vec<Dispatch>,
    pub diagnostics: Vec<DiagnosticEvent>,
    pub run: Option<Run>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExportBuildOptions {
    pub template_name: Option<String>,
    pub export_profile_name: Option<String>,
    pub browser_name: Option<String>,
    pub extension_version: Option<String>,
    pub title: String,
}

impl ExportDocument {
    pub fn from_workspace_snapshot(
        snapshot: &WorkspaceSnapshot,
        profile: Option<&ExportProfile>,
    ) -> Self {
        let workspace = snapshot.workspace.clone();
        let participant_labels = snapshot
            .bindings
            .iter()
            .map(|binding| binding.provider_id.display_name().to_owned())
            .collect();

        Self {
            metadata: ExportMetadata {
                title: workspace
                    .as_ref()
                    .map(|item| format!("{} Export", item.name))
                    .unwrap_or_else(|| "Chatmux Export".to_owned()),
                exported_at: Utc::now(),
                workspace_name: workspace.as_ref().map(|item| item.name.clone()),
                workspace_id: workspace.as_ref().map(|item| item.id.0.to_string()),
                participant_labels,
                orchestration_mode: snapshot.runs.last().map(|run| format!("{:?}", run.mode)),
                run_id: snapshot.runs.last().map(|run| run.id.0.to_string()),
                round_range: None,
                message_count: snapshot.recent_messages.len(),
                template_name: None,
                export_profile_name: profile.map(|item| item.name.clone()),
                tags: workspace
                    .as_ref()
                    .map(|item| item.tags.clone())
                    .unwrap_or_default(),
                notes: workspace.as_ref().and_then(|item| item.notes.clone()),
                additional: BTreeMap::new(),
            },
            messages: snapshot.recent_messages.clone(),
            dispatches: Vec::new(),
            diagnostics: snapshot.diagnostics.clone(),
            run: snapshot.runs.last().cloned(),
        }
    }
}

pub fn render_export(
    document: &ExportDocument,
    format: ExportFormat,
    layout: ExportLayout,
) -> String {
    match format {
        ExportFormat::Markdown => render_markdown(document, layout),
        ExportFormat::Json => render_json(document),
        ExportFormat::Toml => render_toml(document),
    }
}

pub fn build_export_document(
    workspace: &Workspace,
    messages: &[Message],
    runs: &[Run],
    dispatches: &[Dispatch],
    diagnostics: &[DiagnosticEvent],
    options: &ExportBuildOptions,
) -> ExportDocument {
    let mut additional = BTreeMap::new();
    if let Some(browser_name) = &options.browser_name {
        additional.insert("browser_name".to_owned(), browser_name.clone());
    }
    if let Some(extension_version) = &options.extension_version {
        additional.insert("extension_version".to_owned(), extension_version.clone());
    }

    ExportDocument {
        metadata: ExportMetadata {
            title: options.title.clone(),
            exported_at: Utc::now(),
            workspace_name: Some(workspace.name.clone()),
            workspace_id: Some(workspace.id.0.to_string()),
            participant_labels: workspace
                .enabled_providers
                .iter()
                .map(|provider| provider.display_name().to_owned())
                .collect(),
            orchestration_mode: runs.last().map(|run| format!("{:?}", run.mode)),
            run_id: runs.last().map(|run| run.id.0.to_string()),
            round_range: None,
            message_count: messages.len(),
            template_name: options.template_name.clone(),
            export_profile_name: options.export_profile_name.clone(),
            tags: workspace.tags.clone(),
            notes: workspace.notes.clone(),
            additional,
        },
        messages: messages.to_vec(),
        dispatches: dispatches.to_vec(),
        diagnostics: diagnostics.to_vec(),
        run: runs.last().cloned(),
    }
}

pub fn render_document(
    document: &ExportDocument,
    format: ExportFormat,
    layout: ExportLayout,
    _include_front_matter: bool,
) -> Result<String, String> {
    Ok(render_export(document, format, layout))
}

pub fn render_filename(
    template: &str,
    substitutions: &BTreeMap<&str, String>,
) -> Result<String, String> {
    let mut rendered = template.to_owned();
    for (key, value) in substitutions {
        rendered = rendered.replace(&format!("{{{key}}}"), value);
    }
    Ok(slugify(&rendered))
}

pub fn render_markdown(document: &ExportDocument, layout: ExportLayout) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", document.metadata.title));
    out.push_str(&format!(
        "- Exported At: {}\n- Message Count: {}\n",
        document.metadata.exported_at.to_rfc3339(),
        document.metadata.message_count
    ));

    if let Some(name) = &document.metadata.workspace_name {
        out.push_str(&format!("- Workspace: {}\n", name));
    }

    out.push('\n');

    match layout {
        ExportLayout::Chronological => {
            for message in &document.messages {
                append_message(&mut out, message, None);
            }
        }
        ExportLayout::GroupedByRound => {
            let mut grouped: BTreeMap<Option<u32>, Vec<&Message>> = BTreeMap::new();
            for message in &document.messages {
                grouped.entry(message.round).or_default().push(message);
            }
            for (round, messages) in grouped {
                out.push_str(&format!("## Round {}\n\n", round.unwrap_or(0)));
                for message in messages {
                    append_message(&mut out, message, None);
                }
            }
        }
        ExportLayout::GroupedByParticipant => {
            let mut grouped: BTreeMap<ProviderId, Vec<&Message>> = BTreeMap::new();
            for message in &document.messages {
                grouped
                    .entry(message.participant_id)
                    .or_default()
                    .push(message);
            }
            for (participant, messages) in grouped {
                out.push_str(&format!("## {}\n\n", participant.display_name()));
                for message in messages {
                    append_message(&mut out, message, Some(participant));
                }
            }
        }
    }

    if !document.diagnostics.is_empty() {
        out.push_str("## Diagnostics\n\n");
        for diagnostic in &document.diagnostics {
            out.push_str(&format!(
                "- [{:?}] {}: {}\n",
                diagnostic.level, diagnostic.code, diagnostic.detail
            ));
        }
    }

    out
}

fn append_message(out: &mut String, message: &Message, participant_override: Option<ProviderId>) {
    let participant = participant_override.unwrap_or(message.participant_id);
    out.push_str(&format!(
        "### {} · {:?} · {}\n\n{}\n\n",
        participant.display_name(),
        message.role,
        message.timestamp.to_rfc3339(),
        message.body_text
    ));
}

pub fn render_json(document: &ExportDocument) -> String {
    serde_json::to_string_pretty(document).expect("export JSON rendering should succeed")
}

pub fn render_toml(document: &ExportDocument) -> String {
    toml::to_string_pretty(document).expect("export TOML rendering should succeed")
}

pub fn render_filename_template(
    template: &str,
    workspace: Option<&Workspace>,
    format: ExportFormat,
) -> String {
    let mut rendered = template.to_owned();
    let workspace_name = workspace
        .map(|item| slugify(&item.name))
        .unwrap_or_else(|| "chatmux".to_owned());
    let extension = match format {
        ExportFormat::Markdown => "md",
        ExportFormat::Json => "json",
        ExportFormat::Toml => "toml",
    };

    let replacements = [
        ("{workspace}", workspace_name),
        ("{date}", Utc::now().format("%Y-%m-%d").to_string()),
        ("{format}", extension.to_owned()),
    ];

    for (key, value) in replacements {
        rendered = rendered.replace(key, &value);
    }

    let sanitized = slugify(&rendered);
    let expected_suffix = format!(".{extension}");
    if sanitized.ends_with(&expected_suffix) {
        sanitized
    } else {
        format!("{}.{}", sanitized, extension)
    }
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;

    for ch in input.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }

    out.trim_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chatmux_common::{CaptureConfidence, MessageRole, WorkspaceId};

    fn message(body: &str) -> Message {
        Message {
            id: chatmux_common::MessageId::new(),
            workspace_id: WorkspaceId::new(),
            participant_id: ProviderId::Gpt,
            role: MessageRole::Assistant,
            round: Some(1),
            timestamp: Utc::now(),
            body_text: body.to_owned(),
            body_blocks: Vec::new(),
            source_binding_id: None,
            dispatch_id: None,
            raw_response_text: None,
            network_capture: None,
            tags: Vec::new(),
            capture_confidence: CaptureConfidence::Certain,
        }
    }

    #[test]
    fn renders_markdown_chronologically() {
        let document = ExportDocument {
            metadata: ExportMetadata {
                title: "Test Export".to_owned(),
                exported_at: Utc::now(),
                workspace_name: Some("Alpha".to_owned()),
                workspace_id: None,
                participant_labels: vec!["ChatGPT".to_owned()],
                orchestration_mode: None,
                run_id: None,
                round_range: None,
                message_count: 1,
                template_name: None,
                export_profile_name: None,
                tags: Vec::new(),
                notes: None,
                additional: BTreeMap::new(),
            },
            messages: vec![message("hello")],
            dispatches: Vec::new(),
            diagnostics: Vec::new(),
            run: None,
        };

        let rendered = render_markdown(&document, ExportLayout::Chronological);
        assert!(rendered.contains("Test Export"));
        assert!(rendered.contains("hello"));
    }

    #[test]
    fn filename_template_slugifies_values() {
        let workspace = Workspace {
            id: WorkspaceId::new(),
            name: "My Workspace".to_owned(),
            archived: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            enabled_providers: Default::default(),
            default_mode: chatmux_common::OrchestrationMode::Broadcast,
            default_context_strategy: chatmux_common::ContextStrategy::WorkspaceDefault,
            default_template_id: None,
            active_export_profile_ids: Vec::new(),
            tags: Vec::new(),
            notes: None,
        };

        let rendered = render_filename_template(
            "{workspace}-{format}",
            Some(&workspace),
            ExportFormat::Markdown,
        );
        assert_eq!(rendered, "my-workspace-md.md");
    }
}
