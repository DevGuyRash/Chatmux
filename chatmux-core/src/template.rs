//! Prompt template rendering helpers.

use chatmux_common::{Message, ProviderId, Template, TemplateKind, WorkspaceId};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct RenderedPackage {
    pub body: String,
    pub source_message_ids: Vec<chatmux_common::MessageId>,
    pub character_count: usize,
}

pub fn render_template(
    template: &Template,
    target: ProviderId,
    messages: &[Message],
    user_note: Option<&str>,
) -> RenderedPackage {
    let grouped = messages
        .iter()
        .map(|message| render_message_block(template, message))
        .collect::<Vec<_>>()
        .join("\n\n");

    let replacements = BTreeMap::from([
        ("{{target_provider}}", target.display_name().to_owned()),
        ("{{message_bundle}}", grouped.clone()),
        ("{{user_note}}", user_note.unwrap_or_default().to_owned()),
    ]);

    let mut body = if template.kind == TemplateKind::Custom
        && !template.body_template.contains("{{message_bundle}}")
        && !messages.is_empty()
    {
        grouped.clone()
    } else {
        template.body_template.clone()
    };
    for (needle, value) in replacements {
        body = body.replace(needle, &value);
    }
    if let Some(preamble) = &template.preamble {
        body = format!("{preamble}\n\n{body}");
    }
    body = body.trim_end().to_owned();

    RenderedPackage {
        character_count: body.chars().count(),
        body,
        source_message_ids: messages.iter().map(|message| message.id).collect(),
    }
}

fn render_message_block(template: &Template, message: &Message) -> String {
    let codename = provider_codename(message.participant_id);
    match template.kind {
        TemplateKind::BuiltinWrappedXml => {
            let tag = match message.role {
                chatmux_common::MessageRole::User => "user-input".to_owned(),
                chatmux_common::MessageRole::System => "system-note".to_owned(),
                _ => format!("{codename}-response"),
            };
            format!("<{tag}>\n{}\n</{tag}>", message.body_text)
        }
        TemplateKind::BuiltinMarkdownSections => format!(
            "## {} · {:?}\n\n{}",
            message.participant_id.display_name(),
            message.role,
            message.body_text
        ),
        TemplateKind::BuiltinPlainLabels => format!(
            "[{} | {:?}]\n{}",
            message.participant_id.display_name(),
            message.role,
            message.body_text
        ),
        TemplateKind::Custom => {
            let replacements = [
                ("{{provider_codename}}", codename.to_owned()),
                (
                    "{{provider_name}}",
                    message.participant_id.display_name().to_owned(),
                ),
                ("{{role}}", format!("{:?}", message.role)),
                ("{{timestamp}}", message.timestamp.to_rfc3339()),
                (
                    "{{round}}",
                    message
                        .round
                        .map(|round| round.to_string())
                        .unwrap_or_default(),
                ),
                ("{{message_id}}", message.id.0.to_string()),
                ("{{body}}", message.body_text.clone()),
            ];
            let mut block = template.body_template.clone();
            for (key, value) in replacements {
                block = block.replace(key, &value);
            }
            block
        }
    }
}

fn provider_codename(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::User => "user",
        ProviderId::System => "system",
        ProviderId::Gpt => "gpt",
        ProviderId::Gemini => "gemini",
        ProviderId::Grok => "grok",
        ProviderId::Claude => "claude",
    }
}

/// Built-in packaging families and preamble presets shipped with every workspace.
pub fn builtin_templates(workspace_id: WorkspaceId) -> Vec<Template> {
    let specs = [
        (
            TemplateKind::BuiltinWrappedXml,
            "Neutral · Wrapped XML",
            None,
        ),
        (
            TemplateKind::BuiltinMarkdownSections,
            "Neutral · Markdown sections",
            None,
        ),
        (
            TemplateKind::BuiltinPlainLabels,
            "Neutral · Plain labels",
            None,
        ),
        (
            TemplateKind::BuiltinWrappedXml,
            "Collaboration",
            Some("Collaborate constructively. Reconcile useful ideas and identify uncertainty."),
        ),
        (
            TemplateKind::BuiltinWrappedXml,
            "Debate",
            Some("Challenge the supplied positions rigorously and state which claims survive."),
        ),
        (
            TemplateKind::BuiltinWrappedXml,
            "Review / Critique",
            Some("Review the supplied work for correctness, omissions, failure modes, and excess."),
        ),
        (
            TemplateKind::BuiltinWrappedXml,
            "Synthesis",
            Some(
                "Synthesize the strongest supported answer from the supplied participant responses.",
            ),
        ),
    ];
    specs
        .into_iter()
        .map(|(kind, name, preamble)| Template {
            id: chatmux_common::TemplateId::new(),
            workspace_id,
            kind,
            name: name.to_owned(),
            version: "1.0.0".to_owned(),
            body_template: "{{message_bundle}}\n\n{{user_note}}".to_owned(),
            preamble: preamble.map(str::to_owned),
            metadata_template: None,
            filename_template: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chatmux_common::{Block, CaptureConfidence, MessageId, MessageRole};
    use chrono::Utc;

    fn message(provider: ProviderId, body: &str) -> Message {
        Message {
            id: MessageId::new(),
            workspace_id: WorkspaceId::new(),
            participant_id: provider,
            role: MessageRole::Assistant,
            round: Some(2),
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
    fn wrapped_template_uses_stable_provider_codename() {
        let template = builtin_templates(WorkspaceId::new())
            .into_iter()
            .next()
            .expect("wrapped built-in exists");
        let rendered = render_template(
            &template,
            ProviderId::Claude,
            &[message(ProviderId::Gpt, "answer")],
            None,
        );
        assert!(rendered.body.contains("<gpt-response>"));
        assert!(!rendered.body.contains("<chatgpt-response>"));
    }

    #[test]
    fn custom_template_expands_per_message_variables() {
        let template = Template {
            id: chatmux_common::TemplateId::new(),
            workspace_id: WorkspaceId::new(),
            kind: TemplateKind::Custom,
            name: "Custom".to_owned(),
            version: "1".to_owned(),
            body_template: "{{provider_codename}}/{{round}}: {{body}}".to_owned(),
            preamble: None,
            metadata_template: None,
            filename_template: None,
        };
        let rendered = render_template(
            &template,
            ProviderId::Claude,
            &[message(ProviderId::Gpt, "answer")],
            None,
        );
        assert_eq!(rendered.body, "gpt/2: answer");
    }
}
