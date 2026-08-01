//! Template manager (§3.12).
//!
//! Left column: template list. Right column: template editor.
//! Sidebar: sequential (list → editor). Full-tab: side-by-side.

use leptos::prelude::*;

use super::template_editor::TemplateEditor;
use super::template_list::TemplateList;
use crate::components::primitives::button::{Button, ButtonVariant};
use crate::layout::responsive::LayoutMode;
use crate::models::{Template, TemplateId, TemplateKind, WorkspaceId};

/// Template manager component.
#[component]
pub fn TemplateManager(
    /// Workspace that owns newly-created templates.
    workspace_id: Signal<Option<WorkspaceId>>,
    /// Available templates.
    templates: Signal<Vec<Template>>,
    /// Called to save a template.
    on_save: impl Fn(Template) + 'static + Copy + Send,
    /// Called to delete a custom template.
    on_delete: impl Fn(TemplateId) + 'static + Copy + Send,
) -> impl IntoView {
    let layout_mode = expect_context::<LayoutMode>();
    let (selected_id, set_selected_id) = signal(None::<TemplateId>);

    let selected_template = Signal::derive(move || {
        let id = selected_id.get()?;
        templates.get().into_iter().find(|t| t.id == id)
    });

    view! {
        <div class="template-manager flex h-full" style=match layout_mode {
            LayoutMode::Sidebar => "flex-direction: column;",
            LayoutMode::FullTab => "flex-direction: row;",
        }>
            // Template list
            <div style=match layout_mode {
                LayoutMode::Sidebar => "width: 100%;",
                LayoutMode::FullTab => "width: 280px; border-right: 1px solid var(--border-subtle); overflow-y: auto;",
            }>
                <div class="flex items-center justify-between p-4 border-b">
                    <span class="type-title text-primary">"Templates"</span>
                    <Button
                        variant=ButtonVariant::Primary
                        disabled=Signal::derive(move || workspace_id.get().is_none())
                        on_click=Box::new(move |_| {
                            let Some(workspace_id) = workspace_id.get_untracked() else {
                                return;
                            };
                            let template = Template {
                                id: TemplateId::new(),
                                workspace_id,
                                kind: TemplateKind::Custom,
                                name: "Custom package".to_owned(),
                                version: "1.0.0".to_owned(),
                                body_template: "{{provider_codename}}/{{role}}:\n{{body}}".to_owned(),
                                preamble: None,
                                metadata_template: None,
                                filename_template: None,
                            };
                            let id = template.id;
                            on_save(template);
                            set_selected_id.set(Some(id));
                        })
                    >
                        "+ Create"
                    </Button>
                </div>
                <TemplateList
                    templates=templates
                    selected=selected_id
                    on_select=move |id| set_selected_id.set(Some(id))
                />
            </div>

            // Template editor
            <div class="flex-1 overflow-y-auto p-5">
                {move || match selected_template.get() {
                    Some(tmpl) => view! {
                        <TemplateEditor
                            template=tmpl
                            on_save=on_save
                            on_delete=move |id| {
                                on_delete(id);
                                set_selected_id.set(None);
                            }
                            on_cancel=move || set_selected_id.set(None)
                        />
                    }.into_any(),
                    None => view! {
                        <div class="flex items-center justify-center h-full">
                            <p class="type-body text-secondary">"Select a template to edit"</p>
                        </div>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}
