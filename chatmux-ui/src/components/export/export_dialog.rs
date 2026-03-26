//! Export dialog (§3.13).
//!
//! Sidebar: full-screen overlay. Full-tab: 640px centered modal.
//! Sections: scope selector, selection filters, format picker,
//! layout options, metadata toggles, filename template, profiles, actions.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::modal::Modal;
use crate::components::primitives::segmented_control::{Segment, SegmentedControl};

/// Export dialog component.
#[component]
pub fn ExportDialog(
    /// Whether the dialog is open.
    open: ReadSignal<bool>,
    /// Called to close.
    on_close: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    let (format, set_format) = signal("markdown".to_string());
    let (scope, set_scope) = signal("workspace".to_string());

    // TODO(backend): Call generate_export with the selected configuration.
    // TODO(backend): Call write_clipboard for the "Copy to Clipboard" action.

    view! {
        <Modal open=open on_close=on_close max_width=640>
            <div class="flex flex-col gap-6" style="max-height: 80vh; overflow-y: auto;">
                <h2 class="type-title text-primary">"Export"</h2>

                // §1 — Scope selector
                <section>
                    <h3 class="type-label text-secondary" style="margin-bottom: var(--space-3);">"Scope"</h3>
                    <div class="flex flex-col gap-2">
                        {["workspace", "provider", "run", "rounds", "messages", "dispatch", "diagnostics"]
                            .iter().map(|&value| {
                                let label = match value {
                                    "workspace" => "Entire Workspace",
                                    "provider" => "Single Provider",
                                    "run" => "Single Run",
                                    "rounds" => "Selected Rounds",
                                    "messages" => "Selected Messages",
                                    "dispatch" => "Dispatch Log",
                                    "diagnostics" => "Diagnostics",
                                    _ => value,
                                };
                                let v_check = value.to_string();
                                let v_change = value.to_string();
                                view! {
                                    <label class="flex items-center gap-3 cursor-pointer">
                                        <input
                                            type="radio"
                                            name="export-scope"
                                            value=value
                                            checked=move || scope.get() == v_check
                                            on:change=move |_| set_scope.set(v_change.clone())
                                        />
                                        <span class="type-body text-primary">{label}</span>
                                    </label>
                                }
                            }).collect_view()
                        }
                    </div>
                </section>

                // §3 — Format picker
                <section>
                    <h3 class="type-label text-secondary" style="margin-bottom: var(--space-3);">"Format"</h3>
                    <SegmentedControl
                        segments=vec![
                            Segment { value: "markdown".into(), label: "Markdown".into() },
                            Segment { value: "json".into(), label: "JSON".into() },
                            Segment { value: "toml".into(), label: "TOML".into() },
                        ]
                        selected=format
                        on_change=move |v| set_format.set(v)
                    />
                </section>

                // §5 — Metadata toggles (placeholder)
                <section>
                    <h3 class="type-label text-secondary" style="margin-bottom: var(--space-3);">"Metadata"</h3>
                    <p class="type-caption text-tertiary">
                        "Metadata toggle grid will be populated with checkbox fields for: \
                         workspace name, export title, timestamps, participants, mode, tags, etc."
                    </p>
                </section>

                // §6 — Filename template
                <section>
                    <h3 class="type-label text-secondary" style="margin-bottom: var(--space-3);">"Filename"</h3>
                    <input
                        class="type-body w-full"
                        type="text"
                        value="{workspace}-{date}-{format}"
                        style="\
                            padding: var(--space-3) var(--space-4); \
                            background: var(--surface-sunken); \
                            border: 1px solid var(--border-default); \
                            border-radius: var(--radius-md); \
                            color: var(--text-primary);"
                    />
                </section>

                // §8 — Actions
                <div class="flex justify-between items-center"
                     style="margin-top: var(--space-4); padding-top: var(--space-4); \
                            border-top: 1px solid var(--border-subtle);">
                    <Button variant=ButtonVariant::Secondary>
                        "Copy to Clipboard"
                    </Button>
                    <Button variant=ButtonVariant::Primary>
                        "Download"
                    </Button>
                </div>
            </div>
        </Modal>
    }
}
