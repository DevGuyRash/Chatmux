//! Template editor (§3.12).
//!
//! Template name input, kind dropdown, format family selector (body only),
//! template body text area (font-mono), variable reference, live preview.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::text_area::TextArea;
use crate::components::primitives::text_input::TextInput;
use crate::models::{Template, TemplateId, TemplateKind};

/// Template editor component.
#[component]
pub fn TemplateEditor(
    /// The template being edited.
    template: Template,
    /// Called to save.
    on_save: impl Fn(Template) + 'static + Copy + Send,
    /// Called to delete a custom template.
    on_delete: impl Fn(TemplateId) + 'static + Copy + Send,
    /// Called to cancel.
    on_cancel: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    let (name, set_name) = signal(template.name.clone());
    let (body, set_body) = signal(template.body_template.clone());
    let (preamble, set_preamble) = signal(template.preamble.clone().unwrap_or_default());
    let is_builtin = template.kind != TemplateKind::Custom;
    let kind = template.kind.clone();

    // A preview that echoed the body back substituted nothing, so it showed the
    // operator exactly what they had already typed one box above and told them
    // nothing about what would actually be sent. This renders sample messages
    // through the same rules the packager applies.
    let preview = Signal::derive(move || render_preview(&kind, &body.get(), &preamble.get()));

    view! {
        <div class="template-editor flex flex-col gap-5">
            // Name input
            <div>
                <label class="type-label text-secondary mb-2 block">
                    "Template name"
                </label>
                <TextInput
                    value=name
                    on_input=move |v| set_name.set(v)
                    placeholder="Template name"
                    disabled=is_builtin
                />
            </div>

            <div>
                <label class="type-label text-secondary mb-2 block">
                    "Optional preamble"
                </label>
                <TextArea
                    value=preamble
                    on_input=move |v| set_preamble.set(v)
                    placeholder="Instructions placed before the packaged messages…"
                    monospace=false
                    min_rows=2
                    max_rows=8
                    disabled=is_builtin
                />
            </div>

            // Kind (read-only for now)
            <div class="type-caption text-secondary">
                {format!("Kind: {}", match template.kind {
                    TemplateKind::BuiltinWrappedXml => "Built-in (Wrapped XML)",
                    TemplateKind::BuiltinMarkdownSections => "Built-in (Markdown Sections)",
                    TemplateKind::BuiltinPlainLabels => "Built-in (Plain Labels)",
                    TemplateKind::Custom => "Custom",
                })}
            </div>

            // Body editor
            <div>
                <label class="type-label text-secondary mb-2 block">
                    "Template body"
                </label>
                <TextArea
                    value=body
                    on_input=move |v| set_body.set(v)
                    placeholder="Template content…"
                    monospace=true
                    min_rows=6
                    max_rows=20
                    disabled=is_builtin
                />
            </div>

            // Variable reference (collapsible)
            <details class="mt-2">
                <summary class="type-caption text-link cursor-pointer">"Available variables"</summary>
                <div class="type-code-small text-secondary surface-sunken p-4 rounded-md mt-2">
                    "{{provider_codename}}, {{provider_name}}, {{role}}, {{timestamp}}, {{body}}, {{round}}, {{message_id}}, {{message_bundle}}, {{target_provider}}, {{user_note}}"
                </div>
            </details>

            // Live preview
            <div>
                <label class="type-label text-secondary mb-2 block">
                    "Preview with sample messages"
                </label>
                <div
                    class="type-code surface-sunken p-4 rounded-md whitespace-pre-wrap break-words"
                    style="min-height: 60px;"
                >
                    {move || preview.get()}
                </div>
                <p class="type-caption text-tertiary mt-2">
                    "Sample content, rendered the way this template packages a real round."
                </p>
            </div>

            // Actions
            {(!is_builtin).then(|| view! {
                <div class="flex gap-3 justify-end">
                    <Button variant=ButtonVariant::Danger on_click=Box::new(move |_| {
                        on_delete(template.id);
                    })>
                        "Delete"
                    </Button>
                    <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| on_cancel())>
                        "Cancel"
                    </Button>
                    <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                        let mut updated = template.clone();
                        updated.name = name.get_untracked();
                        updated.body_template = body.get_untracked();
                        updated.preamble = match preamble.get_untracked().trim() {
                            "" => None,
                            value => Some(value.to_owned()),
                        };
                        on_save(updated);
                    })>
                        "Save"
                    </Button>
                </div>
            })}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Preview rendering
// ---------------------------------------------------------------------------
//
// Mirrors the packaging rules in `chatmux-core::template` against a fixed
// sample round. The UI crate does not depend on `chatmux-core`, so this cannot
// call the real renderer; if the packaging rules there change, this must follow.

/// One sample turn: the participant's display name, its codename, its role tag
/// and its body.
const SAMPLE_TURNS: &[(&str, &str, &str, &str)] = &[
    (
        "You",
        "user",
        "User",
        "Compare the two approaches and say which you'd ship.",
    ),
    (
        "ChatGPT",
        "gpt",
        "Assistant",
        "The first is simpler to operate; the second scales further.",
    ),
    (
        "Claude",
        "claude",
        "Assistant",
        "Agreed on the tradeoff, but the second one's failure mode is worse.",
    ),
];

const SAMPLE_TARGET: &str = "Gemini";
const SAMPLE_USER_NOTE: &str = "Focus on the operational cost.";

fn render_sample_block(
    kind: &TemplateKind,
    name: &str,
    codename: &str,
    role: &str,
    body: &str,
) -> String {
    match kind {
        TemplateKind::BuiltinWrappedXml => {
            let tag = match role {
                "User" => "user-input".to_owned(),
                "System" => "system-note".to_owned(),
                _ => format!("{codename}-response"),
            };
            format!("<{tag}>\n{body}\n</{tag}>")
        }
        TemplateKind::BuiltinMarkdownSections => format!("## {name} · {role}\n\n{body}"),
        TemplateKind::BuiltinPlainLabels => format!("[{name} | {role}]\n{body}"),
        TemplateKind::Custom => format!("[{name} | {role}]\n{body}"),
    }
}

fn render_preview(kind: &TemplateKind, body_template: &str, preamble: &str) -> String {
    let bundle = SAMPLE_TURNS
        .iter()
        .map(|(name, codename, role, body)| render_sample_block(kind, name, codename, role, body))
        .collect::<Vec<_>>()
        .join("\n\n");

    // Same fallback the packager uses: a custom template that never places the
    // bundle still receives it, otherwise the round would be dropped silently.
    let mut rendered =
        if *kind == TemplateKind::Custom && !body_template.contains("{{message_bundle}}") {
            bundle.clone()
        } else {
            body_template.to_owned()
        };

    for (needle, value) in [
        ("{{target_provider}}", SAMPLE_TARGET),
        ("{{message_bundle}}", bundle.as_str()),
        ("{{user_note}}", SAMPLE_USER_NOTE),
    ] {
        rendered = rendered.replace(needle, value);
    }

    if !preamble.trim().is_empty() {
        rendered = format!("{}\n\n{rendered}", preamble.trim());
    }

    let rendered = rendered.trim_end().to_owned();
    if rendered.is_empty() {
        "Nothing to preview yet — this template renders an empty package.".to_owned()
    } else {
        rendered
    }
}
