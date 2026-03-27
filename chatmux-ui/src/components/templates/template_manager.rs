//! Template manager (§3.12).
//!
//! Left column: template list. Right column: template editor.
//! Sidebar: sequential (list → editor). Full-tab: side-by-side.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::layout::responsive::LayoutMode;
use crate::models::{Template, TemplateId};
use super::template_list::TemplateList;
use super::template_editor::TemplateEditor;

/// Template manager component.
#[component]
pub fn TemplateManager(
    /// Available templates.
    templates: Signal<Vec<Template>>,
    /// Called to save a template.
    on_save: impl Fn(Template) + 'static + Copy + Send,
) -> impl IntoView {
    let layout_mode = expect_context::<ReadSignal<LayoutMode>>();
    let (selected_id, set_selected_id) = signal(None::<TemplateId>);

    let selected_template = Signal::derive(move || {
        let id = selected_id.get()?;
        templates.get().into_iter().find(|t| t.id == id)
    });

    view! {
        <div class="template-manager flex h-full" style=move || match layout_mode.get() {
            LayoutMode::Sidebar => "flex-direction: column;",
            LayoutMode::FullTab => "flex-direction: row;",
        }>
            // Template list
            <div style=move || match layout_mode.get() {
                LayoutMode::Sidebar => "width: 100%;",
                LayoutMode::FullTab => "width: 280px; border-right: 1px solid var(--border-subtle); overflow-y: auto;",
            }>
                <div class="flex items-center justify-between p-4"
                     style="border-bottom: 1px solid var(--border-subtle);">
                    <span class="type-title text-primary">"Templates"</span>
                    <Button variant=ButtonVariant::Primary>
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
