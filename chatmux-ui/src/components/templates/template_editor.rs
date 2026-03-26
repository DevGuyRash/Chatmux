//! Template editor (§3.12).
//!
//! Template name input, kind dropdown, format family selector (body only),
//! template body text area (font-mono), variable reference, live preview.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::text_input::TextInput;
use crate::components::primitives::text_area::TextArea;
use crate::models::{Template, TemplateKind};

/// Template editor component.
#[component]
pub fn TemplateEditor(
    /// The template being edited.
    template: Template,
    /// Called to save.
    on_save: impl Fn(Template) + 'static + Copy + Send,
    /// Called to cancel.
    on_cancel: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    let (name, set_name) = signal(template.name.clone());
    let (body, set_body) = signal(template.body_template.clone());
    let is_builtin = template.kind != TemplateKind::Custom;

    view! {
        <div class="template-editor flex flex-col gap-5">
            // Name input
            <div>
                <label class="type-label text-secondary" style="display: block; margin-bottom: var(--space-2);">
                    "Template Name"
                </label>
                <TextInput
                    value=name
                    on_input=move |v| set_name.set(v)
                    placeholder="Template name"
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
                <label class="type-label text-secondary" style="display: block; margin-bottom: var(--space-2);">
                    "Template Body"
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
            <details style="margin-top: var(--space-2);">
                <summary class="type-caption text-link cursor-pointer">"Available Variables"</summary>
                <div class="type-code-small text-secondary surface-sunken p-4 rounded-md"
                     style="margin-top: var(--space-2);">
                    "{{provider_name}}, {{timestamp}}, {{body}}, {{round}}, {{run_id}}, {{workspace_name}}, {{message_id}}"
                </div>
            </details>

            // Live preview
            <div>
                <label class="type-label text-secondary" style="display: block; margin-bottom: var(--space-2);">
                    "Preview"
                </label>
                <div
                    class="type-code surface-sunken p-4 rounded-md"
                    style="min-height: 60px; white-space: pre-wrap; word-break: break-word;"
                >
                    {move || body.get()}
                </div>
            </div>

            // Actions
            {(!is_builtin).then(|| view! {
                <div class="flex gap-3 justify-end">
                    <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| on_cancel())>
                        "Cancel"
                    </Button>
                    <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                        let mut updated = template.clone();
                        updated.name = name.get_untracked();
                        updated.body_template = body.get_untracked();
                        on_save(updated);
                    })>
                        "Save"
                    </Button>
                </div>
            })}
        </div>
    }
}
