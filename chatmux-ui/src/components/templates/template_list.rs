//! Template list (§3.12).
//!
//! Grouped by kind (Preamble, Body, Filename) with section headers.

use leptos::prelude::*;

use crate::components::primitives::badge::{Badge, BadgeVariant};
use crate::models::{Template, TemplateId, TemplateKind};

/// Template list component.
#[component]
pub fn TemplateList(
    /// Templates to display.
    templates: Signal<Vec<Template>>,
    /// Currently selected template ID.
    selected: ReadSignal<Option<TemplateId>>,
    /// Called when a template is selected.
    on_select: impl Fn(TemplateId) + 'static + Copy + Send,
) -> impl IntoView {
    let groups = move || {
        let tmpls = templates.get();
        let builtins: Vec<Template> = tmpls
            .iter()
            .filter(|t| t.kind != TemplateKind::Custom)
            .cloned()
            .collect();
        let custom: Vec<Template> = tmpls
            .iter()
            .filter(|t| t.kind == TemplateKind::Custom)
            .cloned()
            .collect();
        vec![
            ("Built-in Templates", builtins),
            ("Custom Templates", custom),
        ]
    };

    view! {
        <div class="template-list">
            {move || groups().into_iter().map(|(group_label, items)| {
                view! {
                    <div class="mb-6">
                        <h4 class="type-caption-strong text-secondary"
                            style="padding: var(--space-3) var(--space-5); text-transform: uppercase; letter-spacing: 0.05em;">
                            {group_label}
                        </h4>
                        {items.into_iter().map(|tmpl| {
                            let id = tmpl.id;
                            let is_selected = move || selected.get() == Some(id);

                            view! {
                                <button
                                    class="flex items-center gap-2 w-full text-left cursor-pointer select-none transition-colors"
                                    style=move || format!(
                                        "padding: var(--space-3) var(--space-5); \
                                         background: {}; border: none; \
                                         border-left: 3px solid {};",
                                        if is_selected() { "var(--surface-selected)" } else { "transparent" },
                                        if is_selected() { "var(--accent-primary)" } else { "transparent" },
                                    )
                                    on:click=move |_| on_select(id)
                                >
                                    <span class="type-body text-primary flex-1 truncate">{tmpl.name.clone()}</span>
                                    {(tmpl.kind != TemplateKind::Custom).then(|| view! {
                                        <Badge variant=BadgeVariant::Neutral>"Built-in"</Badge>
                                    })}
                                    <span class="type-caption text-tertiary">
                                        {format!("v{}", tmpl.version)}
                                    </span>
                                </button>
                            }
                        }).collect_view()}
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
