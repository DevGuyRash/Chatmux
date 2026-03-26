//! Prompt template rendering helpers.

use chatmux_common::{Message, ProviderId, Template};
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
        .map(|message| {
            format!(
                "<{provider}-response>\n{body}\n</{provider}-response>",
                provider = message.participant_id.display_name().to_lowercase(),
                body = message.body_text
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let replacements = BTreeMap::from([
        ("{{target_provider}}", target.display_name().to_owned()),
        ("{{message_bundle}}", grouped.clone()),
        ("{{user_note}}", user_note.unwrap_or_default().to_owned()),
    ]);

    let mut body = template.body_template.clone();
    for (needle, value) in replacements {
        body = body.replace(needle, &value);
    }
    if let Some(preamble) = &template.preamble {
        body = format!("{preamble}\n\n{body}");
    }

    RenderedPackage {
        character_count: body.chars().count(),
        body,
        source_message_ids: messages.iter().map(|message| message.id).collect(),
    }
}
