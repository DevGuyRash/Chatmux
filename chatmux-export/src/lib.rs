//! Canonical export selection, model, renderers, and filename templates for Chatmux.

use chatmux_common::{
    DiagnosticEvent, Dispatch, DispatchOutcome, ExportFormat, ExportLayout, ExportProfile,
    ExportRequest, ExportScopePreset, Message, MetadataIncludeFlags, ProviderId, Run, Workspace,
    WorkspaceSnapshot,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const EXPORT_SCHEMA_VERSION: &str = "chatmux.export.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub title: Option<String>,
    pub exported_at: Option<DateTime<Utc>>,
    pub workspace_name: Option<String>,
    pub workspace_id: Option<String>,
    pub scope_type: Option<String>,
    pub participant_labels: Vec<String>,
    pub orchestration_mode: Option<String>,
    pub run_id: Option<String>,
    pub round_range: Option<String>,
    pub message_count: Option<usize>,
    pub template_name: Option<String>,
    pub export_profile_name: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub additional: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportDocument {
    pub schema_version: String,
    pub metadata: ExportMetadata,
    pub participants: BTreeMap<String, String>,
    pub messages: Vec<Message>,
    pub runs: Vec<Run>,
    pub dispatches: Vec<Dispatch>,
    pub diagnostics: Vec<DiagnosticEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExportSelection {
    pub messages: Vec<Message>,
    pub runs: Vec<Run>,
    pub dispatches: Vec<Dispatch>,
    pub diagnostics: Vec<DiagnosticEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExportBuildOptions {
    pub template_name: Option<String>,
    pub export_profile_name: Option<String>,
    pub browser_name: Option<String>,
    pub extension_version: Option<String>,
    pub title: String,
    pub scope: Option<ExportScopePreset>,
    pub include_flags: MetadataIncludeFlags,
    pub context_strategy_snapshot: Option<String>,
    pub edge_policy_snapshot: Option<String>,
    pub conversation_refs: Vec<String>,
    pub model_labels: Vec<String>,
}

impl ExportDocument {
    pub fn from_workspace_snapshot(
        snapshot: &WorkspaceSnapshot,
        profile: Option<&ExportProfile>,
    ) -> Self {
        let workspace = snapshot.workspace.clone();
        let mut participants = BTreeMap::new();
        for binding in &snapshot.bindings {
            participants.insert(
                provider_key(binding.provider_id).to_owned(),
                binding.provider_id.display_name().to_owned(),
            );
        }
        Self {
            schema_version: EXPORT_SCHEMA_VERSION.to_owned(),
            metadata: ExportMetadata {
                title: workspace
                    .as_ref()
                    .map(|item| format!("{} Export", item.name)),
                exported_at: Some(Utc::now()),
                workspace_name: workspace.as_ref().map(|item| item.name.clone()),
                workspace_id: workspace.as_ref().map(|item| item.id.0.to_string()),
                scope_type: Some("entire_workspace".to_owned()),
                participant_labels: participants.values().cloned().collect(),
                orchestration_mode: snapshot.runs.last().map(|run| format!("{:?}", run.mode)),
                run_id: snapshot.runs.last().map(|run| run.id.0.to_string()),
                round_range: round_range(&snapshot.recent_messages),
                message_count: Some(snapshot.recent_messages.len()),
                template_name: None,
                export_profile_name: profile.map(|item| item.name.clone()),
                tags: workspace
                    .as_ref()
                    .map(|item| item.tags.clone())
                    .unwrap_or_default(),
                notes: workspace.as_ref().and_then(|item| item.notes.clone()),
                additional: BTreeMap::new(),
            },
            participants,
            messages: snapshot.recent_messages.clone(),
            runs: snapshot.runs.clone(),
            dispatches: Vec::new(),
            diagnostics: snapshot.diagnostics.clone(),
        }
    }
}

/// Apply all scope and filter controls before building the canonical export document.
pub fn apply_export_request(
    request: &ExportRequest,
    messages: &[Message],
    runs: &[Run],
    dispatches: &[Dispatch],
    diagnostics: &[DiagnosticEvent],
) -> Result<ExportSelection, String> {
    validate_request(request)?;

    let selected_run_id = request.run_id;
    let selected_runs = runs
        .iter()
        .filter(|run| selected_run_id.is_none_or(|run_id| run.id == run_id))
        .cloned()
        .collect::<Vec<_>>();
    let selected_run_ids = selected_runs
        .iter()
        .map(|run| run.id)
        .collect::<BTreeSet<_>>();
    let run_dispatches = dispatches
        .iter()
        .filter(|dispatch| {
            selected_run_id.is_none_or(|_| selected_run_ids.contains(&dispatch.run_id))
        })
        .cloned()
        .collect::<Vec<_>>();
    let scoped_message_ids = message_ids_for_dispatches(&run_dispatches);
    let scoped_dispatch_ids = run_dispatches
        .iter()
        .map(|dispatch| dispatch.id)
        .collect::<BTreeSet<_>>();

    let start = request
        .time_range_iso
        .as_ref()
        .map(|(value, _)| parse_rfc3339(value, "start"))
        .transpose()?;
    let end = request
        .time_range_iso
        .as_ref()
        .map(|(_, value)| parse_rfc3339(value, "end"))
        .transpose()?;

    let mut selected_messages = messages
        .iter()
        .filter(|message| {
            let scope_match = match request.scope {
                ExportScopePreset::EntireWorkspace => true,
                ExportScopePreset::SingleProvider => {
                    request.participants.contains(&message.participant_id)
                        || (message.participant_id == ProviderId::User
                            && related_user_message(
                                message,
                                &request.participants,
                                &run_dispatches,
                            ))
                }
                ExportScopePreset::SingleRun => {
                    scoped_message_ids.contains(&message.id)
                        || message
                            .dispatch_id
                            .is_some_and(|id| scoped_dispatch_ids.contains(&id))
                }
                ExportScopePreset::SelectedRounds => message
                    .round
                    .is_some_and(|round| request.selected_rounds.contains(&round)),
                ExportScopePreset::SelectedMessages => {
                    request.selected_message_ids.contains(&message.id)
                }
                ExportScopePreset::ProviderOnlySubset => {
                    message.participant_id != ProviderId::User
                        && (request.participants.is_empty()
                            || request.participants.contains(&message.participant_id))
                }
                ExportScopePreset::DispatchSubset | ExportScopePreset::DiagnosticSubset => false,
            };

            let filters_match = (request.participants.is_empty()
                || request.participants.contains(&message.participant_id)
                || (request.scope == ExportScopePreset::SingleProvider
                    && message.participant_id == ProviderId::User))
                && (request.roles.is_empty() || request.roles.contains(&message.role))
                && (request.selected_rounds.is_empty()
                    || message
                        .round
                        .is_some_and(|round| request.selected_rounds.contains(&round)))
                && start.is_none_or(|value| message.timestamp >= value)
                && end.is_none_or(|value| message.timestamp <= value)
                && (request.tags.is_empty()
                    || request.tags.iter().all(|tag| message.tags.contains(tag)))
                && request.query.as_ref().is_none_or(|query| {
                    message
                        .body_text
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                })
                && (request.run_id.is_none()
                    || scoped_message_ids.contains(&message.id)
                    || message
                        .dispatch_id
                        .is_some_and(|id| scoped_dispatch_ids.contains(&id)))
                && matches_delivery_outcome(message, &request.delivery_outcomes, &run_dispatches);

            let matched = scope_match && filters_match;
            if request.invert_selection {
                !matched
            } else {
                matched
            }
        })
        .cloned()
        .collect::<Vec<_>>();
    selected_messages.sort_by_key(|message| message.timestamp);

    let selected_dispatches = if matches!(
        request.scope,
        ExportScopePreset::SingleRun | ExportScopePreset::DispatchSubset
    ) || request.include_flags.raw_payload_inclusion
    {
        run_dispatches
            .into_iter()
            .filter(|dispatch| {
                request.delivery_outcomes.is_empty()
                    || request.delivery_outcomes.contains(&dispatch.outcome)
            })
            .collect()
    } else {
        Vec::new()
    };
    let selected_diagnostics = if request.scope == ExportScopePreset::DiagnosticSubset {
        diagnostics.to_vec()
    } else {
        Vec::new()
    };

    Ok(ExportSelection {
        messages: selected_messages,
        runs: selected_runs,
        dispatches: selected_dispatches,
        diagnostics: selected_diagnostics,
    })
}

fn validate_request(request: &ExportRequest) -> Result<(), String> {
    match request.scope {
        ExportScopePreset::SingleProvider if request.participants.len() != 1 => {
            Err("single-provider export requires exactly one selected provider".to_owned())
        }
        ExportScopePreset::SingleRun if request.run_id.is_none() => {
            Err("single-run export requires a run id".to_owned())
        }
        ExportScopePreset::SelectedRounds if request.selected_rounds.is_empty() => {
            Err("selected-rounds export requires at least one round".to_owned())
        }
        ExportScopePreset::SelectedMessages if request.selected_message_ids.is_empty() => {
            Err("selected-messages export requires at least one message".to_owned())
        }
        _ => Ok(()),
    }
}

fn parse_rfc3339(value: &str, label: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| format!("invalid {label} export timestamp {value:?}: {error}"))
}

fn message_ids_for_dispatches(dispatches: &[Dispatch]) -> BTreeSet<chatmux_common::MessageId> {
    dispatches
        .iter()
        .flat_map(|dispatch| dispatch.source_message_ids.iter().copied())
        .collect::<BTreeSet<_>>()
}

fn related_user_message(
    message: &Message,
    providers: &BTreeSet<ProviderId>,
    dispatches: &[Dispatch],
) -> bool {
    dispatches.iter().any(|dispatch| {
        providers.contains(&dispatch.target_participant_id)
            && dispatch.source_message_ids.contains(&message.id)
    })
}

fn matches_delivery_outcome(
    message: &Message,
    outcomes: &[DispatchOutcome],
    dispatches: &[Dispatch],
) -> bool {
    if outcomes.is_empty() {
        return true;
    }
    dispatches.iter().any(|dispatch| {
        outcomes.contains(&dispatch.outcome)
            && (message.dispatch_id == Some(dispatch.id)
                || dispatch.source_message_ids.contains(&message.id))
    })
}

pub fn build_export_document(
    workspace: &Workspace,
    messages: &[Message],
    runs: &[Run],
    dispatches: &[Dispatch],
    diagnostics: &[DiagnosticEvent],
    options: &ExportBuildOptions,
) -> ExportDocument {
    let flags = effective_flags(&options.include_flags);
    let mut additional = BTreeMap::new();
    insert_if(
        &mut additional,
        flags.browser_name,
        "browser_name",
        &options.browser_name,
    );
    insert_if(
        &mut additional,
        flags.extension_version,
        "extension_version",
        &options.extension_version,
    );
    insert_if(
        &mut additional,
        flags.context_strategy_snapshot,
        "context_strategy_snapshot",
        &options.context_strategy_snapshot,
    );
    insert_if(
        &mut additional,
        flags.edge_policy_snapshot,
        "edge_policy_snapshot",
        &options.edge_policy_snapshot,
    );
    if flags.conversation_refs && !options.conversation_refs.is_empty() {
        additional.insert(
            "conversation_refs".to_owned(),
            options.conversation_refs.join(", "),
        );
    }
    if flags.model_labels && !options.model_labels.is_empty() {
        additional.insert("model_labels".to_owned(), options.model_labels.join(", "));
    }
    if flags.diagnostics_summary {
        additional.insert(
            "diagnostic_event_count".to_owned(),
            diagnostics.len().to_string(),
        );
    }
    additional.insert(
        "raw_payloads_included".to_owned(),
        (!dispatches.is_empty()).to_string(),
    );

    let provider_set = messages
        .iter()
        .map(|message| message.participant_id)
        .chain(
            dispatches
                .iter()
                .map(|dispatch| dispatch.target_participant_id),
        )
        .collect::<BTreeSet<_>>();
    let participants = provider_set
        .iter()
        .map(|provider| {
            (
                provider_key(*provider).to_owned(),
                provider.display_name().to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    ExportDocument {
        schema_version: EXPORT_SCHEMA_VERSION.to_owned(),
        metadata: ExportMetadata {
            title: flags.export_title.then(|| options.title.clone()),
            exported_at: flags.export_timestamp.then(Utc::now),
            workspace_name: flags.workspace_name.then(|| workspace.name.clone()),
            workspace_id: flags.workspace_id.then(|| workspace.id.0.to_string()),
            scope_type: flags.scope_type.then(|| {
                format!(
                    "{:?}",
                    options.scope.unwrap_or(ExportScopePreset::EntireWorkspace)
                )
            }),
            participant_labels: if flags.selected_participants {
                participants.values().cloned().collect()
            } else {
                Vec::new()
            },
            orchestration_mode: flags
                .orchestration_mode
                .then(|| runs.last().map(|run| format!("{:?}", run.mode)))
                .flatten(),
            run_id: flags
                .run_id
                .then(|| runs.last().map(|run| run.id.0.to_string()))
                .flatten(),
            round_range: flags.round_range.then(|| round_range(messages)).flatten(),
            message_count: flags.message_count.then_some(messages.len()),
            template_name: flags
                .template_used
                .then(|| options.template_name.clone())
                .flatten(),
            export_profile_name: flags
                .export_profile_name
                .then(|| options.export_profile_name.clone())
                .flatten(),
            tags: if flags.tags_and_notes {
                workspace.tags.clone()
            } else {
                Vec::new()
            },
            notes: flags
                .tags_and_notes
                .then(|| workspace.notes.clone())
                .flatten(),
            additional,
        },
        participants,
        messages: messages.to_vec(),
        runs: runs.to_vec(),
        dispatches: dispatches.to_vec(),
        diagnostics: diagnostics.to_vec(),
    }
}

fn insert_if(
    target: &mut BTreeMap<String, String>,
    include: bool,
    key: &str,
    value: &Option<String>,
) {
    if include && let Some(value) = value {
        target.insert(key.to_owned(), value.clone());
    }
}

pub fn default_metadata_flags() -> MetadataIncludeFlags {
    MetadataIncludeFlags {
        workspace_name: true,
        workspace_id: true,
        export_title: true,
        export_timestamp: true,
        scope_type: true,
        selected_participants: true,
        orchestration_mode: true,
        run_id: true,
        round_range: true,
        message_count: true,
        template_used: true,
        context_strategy_snapshot: true,
        edge_policy_snapshot: true,
        conversation_refs: true,
        model_labels: true,
        browser_name: true,
        extension_version: true,
        export_profile_name: true,
        tags_and_notes: true,
        diagnostics_summary: true,
        raw_payload_inclusion: false,
    }
}

fn effective_flags(flags: &MetadataIncludeFlags) -> MetadataIncludeFlags {
    flags.clone()
}

fn round_range(messages: &[Message]) -> Option<String> {
    let mut rounds = messages.iter().filter_map(|message| message.round);
    let first = rounds.next()?;
    let (minimum, maximum) = rounds.fold((first, first), |(minimum, maximum), round| {
        (minimum.min(round), maximum.max(round))
    });
    Some(if minimum == maximum {
        minimum.to_string()
    } else {
        format!("{minimum}-{maximum}")
    })
}

pub fn render_export(
    document: &ExportDocument,
    format: ExportFormat,
    layout: ExportLayout,
) -> String {
    render_document(document, format, layout, false)
        .unwrap_or_else(|error| format!("export rendering failed: {error}"))
}

pub fn render_document(
    document: &ExportDocument,
    format: ExportFormat,
    layout: ExportLayout,
    include_front_matter: bool,
) -> Result<String, String> {
    match format {
        ExportFormat::Markdown => render_markdown(document, layout, include_front_matter),
        ExportFormat::Json => {
            serde_json::to_string_pretty(document).map_err(|error| error.to_string())
        }
        ExportFormat::Toml => toml::to_string_pretty(document).map_err(|error| error.to_string()),
    }
}

pub fn render_markdown(
    document: &ExportDocument,
    layout: ExportLayout,
    include_front_matter: bool,
) -> Result<String, String> {
    let mut out = String::new();
    if include_front_matter {
        out.push_str("+++\n");
        out.push_str(
            &toml::to_string_pretty(&document.metadata).map_err(|error| error.to_string())?,
        );
        out.push_str("+++\n\n");
    }
    if let Some(title) = &document.metadata.title {
        out.push_str(&format!("# {title}\n\n"));
    }
    if !include_front_matter {
        append_metadata_summary(&mut out, &document.metadata);
    }

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
                out.push_str(&format!(
                    "## Round {}\n\n",
                    round.map_or_else(|| "Unassigned".to_owned(), |value| value.to_string())
                ));
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

    if !document.dispatches.is_empty() {
        out.push_str("## Dispatches\n\n");
        for dispatch in &document.dispatches {
            out.push_str(&format!(
                "### {} · Round {} · {:?}\n\n```text\n{}\n```\n\n",
                dispatch.target_participant_id.display_name(),
                dispatch.round_number,
                dispatch.outcome,
                dispatch.rendered_payload
            ));
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
    Ok(out)
}

fn append_metadata_summary(out: &mut String, metadata: &ExportMetadata) {
    if let Some(exported_at) = metadata.exported_at {
        out.push_str(&format!("- Exported At: {}\n", exported_at.to_rfc3339()));
    }
    if let Some(message_count) = metadata.message_count {
        out.push_str(&format!("- Message Count: {message_count}\n"));
    }
    if let Some(name) = &metadata.workspace_name {
        out.push_str(&format!("- Workspace: {name}\n"));
    }
    if !out.is_empty() {
        out.push('\n');
    }
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
    serde_json::to_string_pretty(document)
        .unwrap_or_else(|error| format!("{{\"error\":{error:?}}}"))
}

pub fn render_toml(document: &ExportDocument) -> String {
    toml::to_string_pretty(document).unwrap_or_else(|error| format!("error = {error:?}"))
}

pub fn render_filename(
    template: &str,
    substitutions: &BTreeMap<&str, String>,
) -> Result<String, String> {
    let owned = substitutions
        .iter()
        .map(|(key, value)| ((*key).to_owned(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    render_filename_specifiers(template, &owned)
}

pub fn render_filename_template(
    template: &str,
    workspace: Option<&Workspace>,
    format: ExportFormat,
) -> String {
    let extension = extension_for(format);
    let values = BTreeMap::from([
        (
            "workspace".to_owned(),
            workspace.map(|item| item.name.clone()).unwrap_or_default(),
        ),
        ("date".to_owned(), Utc::now().format("%Y-%m-%d").to_string()),
        ("time".to_owned(), Utc::now().format("%H-%M-%S").to_string()),
        ("format".to_owned(), extension.to_owned()),
    ]);
    let rendered = render_filename_specifiers(template, &values)
        .unwrap_or_else(|_| "chatmux-export".to_owned());
    with_extension(&rendered, extension)
}

pub fn render_filename_template_with_values(
    template: &str,
    workspace: Option<&Workspace>,
    format: ExportFormat,
    extra_values: &BTreeMap<String, String>,
) -> Result<String, String> {
    let extension = extension_for(format);
    let mut values = BTreeMap::from([
        (
            "workspace".to_owned(),
            workspace.map(|item| item.name.clone()).unwrap_or_default(),
        ),
        ("date".to_owned(), Utc::now().format("%Y-%m-%d").to_string()),
        ("time".to_owned(), Utc::now().format("%H-%M-%S").to_string()),
        ("timestamp".to_owned(), Utc::now().to_rfc3339()),
        ("format".to_owned(), extension.to_owned()),
    ]);
    values.extend(extra_values.clone());
    render_filename_specifiers(template, &values).map(|value| with_extension(&value, extension))
}

pub fn render_filename_specifiers(
    template: &str,
    values: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut rendered = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        rendered.push_str(&rest[..open]);
        let after_open = &rest[open + 1..];
        let Some(close) = after_open.find('}') else {
            return Err("filename template has an unmatched opening brace".to_owned());
        };
        let specifier = &after_open[..close];
        let (key_and_format, fallback) = specifier
            .split_once('|')
            .map_or((specifier, ""), |(key, fallback)| (key, fallback));
        let (key, formatter) = key_and_format
            .split_once(':')
            .map_or((key_and_format, None), |(key, formatter)| {
                (key, Some(formatter))
            });
        let mut value = values.get(key).cloned().unwrap_or_default();
        if value.is_empty() {
            value = fallback.to_owned();
        }
        if let Some(formatter) = formatter {
            value = apply_filename_formatter(key, &value, formatter)?;
        }
        rendered.push_str(&value);
        rest = &after_open[close + 1..];
    }
    rendered.push_str(rest);
    let slug = slugify(&rendered);
    if slug.is_empty() {
        return Err("filename template rendered an empty filename".to_owned());
    }
    Ok(slug.chars().take(180).collect())
}

fn apply_filename_formatter(key: &str, value: &str, formatter: &str) -> Result<String, String> {
    if matches!(key, "date" | "time" | "timestamp") {
        return Ok(Utc::now().format(formatter).to_string());
    }
    if let Some(limit) = formatter.strip_prefix('.') {
        let limit = limit
            .parse::<usize>()
            .map_err(|_| format!("invalid filename truncation formatter {formatter:?}"))?;
        return Ok(value.chars().take(limit).collect());
    }
    if formatter == "+" {
        return Ok(value
            .split(',')
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("+"));
    }
    Err(format!("unsupported filename formatter {formatter:?}"))
}

fn extension_for(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Markdown => "md",
        ExportFormat::Json => "json",
        ExportFormat::Toml => "toml",
    }
}

fn with_extension(filename: &str, extension: &str) -> String {
    let expected = format!(".{extension}");
    if filename.ends_with(&expected) {
        filename.to_owned()
    } else {
        format!("{filename}{expected}")
    }
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() || lower == '.' {
            out.push(lower);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches(['-', '.']).to_owned()
}

fn provider_key(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::User => "user",
        ProviderId::System => "system",
        ProviderId::Gpt => "gpt",
        ProviderId::Gemini => "gemini",
        ProviderId::Grok => "grok",
        ProviderId::Claude => "claude",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chatmux_common::{
        Block, CaptureConfidence, ContextStrategy, DispatchId, ExportProfileId, MessageId,
        MessageRole, OrchestrationMode, WorkspaceId,
    };

    fn workspace() -> Workspace {
        Workspace {
            id: WorkspaceId::new(),
            name: "My Workspace".to_owned(),
            archived: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            enabled_providers: BTreeSet::from([ProviderId::Gpt, ProviderId::Claude]),
            default_mode: OrchestrationMode::Broadcast,
            default_context_strategy: ContextStrategy::WorkspaceDefault,
            default_template_id: None,
            active_export_profile_ids: Vec::new(),
            tags: vec!["research".to_owned()],
            notes: Some("note".to_owned()),
        }
    }

    fn message(
        workspace_id: WorkspaceId,
        provider: ProviderId,
        role: MessageRole,
        round: u32,
        body: &str,
    ) -> Message {
        Message {
            id: MessageId::new(),
            workspace_id,
            participant_id: provider,
            role,
            round: Some(round),
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

    fn request(workspace_id: WorkspaceId, scope: ExportScopePreset) -> ExportRequest {
        ExportRequest {
            workspace_id,
            scope,
            format: ExportFormat::Markdown,
            layout: ExportLayout::Chronological,
            profile_id: None,
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
            include_flags: MetadataIncludeFlags::default(),
            include_front_matter: true,
            filename_template: None,
        }
    }

    #[test]
    fn single_provider_includes_related_user_turn_but_not_other_provider() {
        let workspace = workspace();
        let user = message(
            workspace.id,
            ProviderId::User,
            MessageRole::User,
            1,
            "question",
        );
        let gpt = message(
            workspace.id,
            ProviderId::Gpt,
            MessageRole::Assistant,
            1,
            "gpt",
        );
        let claude = message(
            workspace.id,
            ProviderId::Claude,
            MessageRole::Assistant,
            1,
            "claude",
        );
        let dispatch = Dispatch {
            id: DispatchId::new(),
            run_id: chatmux_common::RunId::new(),
            round_id: None,
            round_number: 1,
            target_participant_id: ProviderId::Gpt,
            source_message_ids: vec![user.id],
            template_id: None,
            rendered_payload: "question".to_owned(),
            sent_at: Some(Utc::now()),
            captured_at: Some(Utc::now()),
            outcome: DispatchOutcome::Delivered,
            error_detail: None,
            retry_count: 0,
        };
        let mut request = request(workspace.id, ExportScopePreset::SingleProvider);
        request.participants.insert(ProviderId::Gpt);

        let selected = apply_export_request(
            &request,
            &[user.clone(), gpt.clone(), claude],
            &[],
            &[dispatch],
            &[],
        )
        .expect("selection succeeds");

        assert_eq!(
            selected
                .messages
                .iter()
                .map(|item| item.id)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([user.id, gpt.id])
        );
    }

    #[test]
    fn arbitrary_message_selection_and_inversion_are_exact() {
        let workspace = workspace();
        let first = message(workspace.id, ProviderId::User, MessageRole::User, 1, "one");
        let second = message(
            workspace.id,
            ProviderId::Gpt,
            MessageRole::Assistant,
            1,
            "two",
        );
        let mut request = request(workspace.id, ExportScopePreset::SelectedMessages);
        request.selected_message_ids.insert(second.id);
        let selected =
            apply_export_request(&request, &[first.clone(), second.clone()], &[], &[], &[])
                .expect("selection succeeds");
        assert_eq!(selected.messages, vec![second.clone()]);

        request.invert_selection = true;
        let inverted = apply_export_request(&request, &[first.clone(), second], &[], &[], &[])
            .expect("inversion succeeds");
        assert_eq!(inverted.messages, vec![first]);
    }

    #[test]
    fn canonical_json_and_toml_share_schema_and_message_identity() {
        let workspace = workspace();
        let message = message(
            workspace.id,
            ProviderId::Gpt,
            MessageRole::Assistant,
            1,
            "hello",
        );
        let document = build_export_document(
            &workspace,
            std::slice::from_ref(&message),
            &[],
            &[],
            &[],
            &ExportBuildOptions {
                title: "Test Export".to_owned(),
                ..ExportBuildOptions::default()
            },
        );

        let json = render_document(
            &document,
            ExportFormat::Json,
            ExportLayout::Chronological,
            true,
        )
        .expect("JSON renders");
        let toml = render_document(
            &document,
            ExportFormat::Toml,
            ExportLayout::Chronological,
            true,
        )
        .expect("TOML renders");

        assert!(json.contains(EXPORT_SCHEMA_VERSION));
        assert!(toml.contains(EXPORT_SCHEMA_VERSION));
        assert!(json.contains(&message.id.0.to_string()));
        assert!(toml.contains(&message.id.0.to_string()));
    }

    #[test]
    fn markdown_front_matter_is_valid_toml_and_preserves_body_fences() {
        let workspace = workspace();
        let message = message(
            workspace.id,
            ProviderId::Gpt,
            MessageRole::Assistant,
            1,
            "```rust\nfn main() {}\n```",
        );
        let document = build_export_document(
            &workspace,
            &[message],
            &[],
            &[],
            &[],
            &ExportBuildOptions {
                title: "Test Export".to_owned(),
                ..ExportBuildOptions::default()
            },
        );
        let markdown = render_document(
            &document,
            ExportFormat::Markdown,
            ExportLayout::Chronological,
            true,
        )
        .expect("Markdown renders");
        let front_matter = markdown
            .strip_prefix("+++\n")
            .and_then(|body| body.split_once("+++\n\n"))
            .map(|(front_matter, _)| front_matter)
            .expect("front matter delimiters");
        toml::from_str::<ExportMetadata>(front_matter).expect("front matter is TOML");
        assert!(markdown.contains("```rust\nfn main() {}\n```"));
    }

    #[test]
    fn filename_specifiers_support_fallback_join_format_and_truncation() {
        let values = BTreeMap::from([
            ("workspace".to_owned(), "A Very Long Workspace".to_owned()),
            ("participants".to_owned(), "gpt, claude".to_owned()),
            ("empty".to_owned(), String::new()),
            ("date".to_owned(), String::new()),
        ]);
        let filename = render_filename_specifiers(
            "{workspace:.8}-{participants:+}-{empty|fallback}-{date:%Y}",
            &values,
        )
        .expect("filename renders");
        assert!(filename.starts_with("a-very-l-gpt-claude-fallback-"));
        assert!(filename.len() <= 180);
    }

    #[test]
    fn filename_template_adds_exact_format_extension() {
        let workspace = workspace();
        let rendered = render_filename_template(
            "{workspace}-{format}",
            Some(&workspace),
            ExportFormat::Markdown,
        );
        assert_eq!(rendered, "my-workspace-md.md");
    }

    #[test]
    fn profile_type_remains_serializable_for_saved_reuse() {
        let profile = ExportProfile {
            id: ExportProfileId::new(),
            workspace_id: WorkspaceId::new(),
            name: "Shareable".to_owned(),
            scope_preset: ExportScopePreset::EntireWorkspace,
            filter_preset: Default::default(),
            format: ExportFormat::Markdown,
            layout: ExportLayout::Chronological,
            include_flags: default_metadata_flags(),
            filename_template: "{workspace}-{date}".to_owned(),
            metadata_template: None,
            prefer_copy: true,
        };
        let json = serde_json::to_string(&profile).expect("profile serializes");
        let restored: ExportProfile = serde_json::from_str(&json).expect("profile deserializes");
        assert_eq!(restored, profile);
    }
}
