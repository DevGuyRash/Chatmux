//! Individual workspace list row (§3.1).
//!
//! Two-line layout: workspace name on top, metadata row below with
//! provider dots, orchestration mode badge, and relative time.
//! States: Default, Hover, Active (selected), Archived.

use leptos::prelude::*;

use crate::components::primitives::badge::{Badge, BadgeVariant};
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::icon::{Icon, IconKind};
use crate::components::provider::Provider;
use crate::models::{OrchestrationMode, ProviderId, Workspace};

/// A single workspace row in the workspace list.
#[component]
pub fn WorkspaceRow(
    /// Workspace data.
    workspace: Workspace,
    /// Click handler.
    on_click: impl Fn() + 'static + Send,
    /// Delete handler — called when the user clicks the delete button.
    on_delete: impl Fn() + 'static + Send,
    /// Rename handler.
    on_rename: impl Fn() + 'static + Send,
    /// Duplicate handler.
    on_duplicate: impl Fn() + 'static + Send,
    /// Archive or restore handler.
    on_archive: impl Fn() + 'static + Send,
) -> impl IntoView {
    let is_archived = workspace.archived;
    let mode_label = orchestration_mode_short(&workspace.default_mode);
    let time_label = relative_time(workspace.updated_at);

    // Collect provider dots for the 4 external providers (skip User/System).
    let provider_dots: Vec<_> = [
        (ProviderId::Gpt, Provider::Gpt),
        (ProviderId::Gemini, Provider::Gemini),
        (ProviderId::Grok, Provider::Grok),
        (ProviderId::Claude, Provider::Claude),
    ]
    .into_iter()
    .filter(|(id, _)| workspace.enabled_providers.contains(id))
    .map(|(_, provider)| provider)
    .collect();

    view! {
        <div
            class="workspace-row flex items-center justify-between w-full cursor-pointer select-none transition-colors border-b"
            style=format!(
                "padding: var(--space-5) var(--space-6); \
                 background: var(--surface-raised); \
                 text-align: left; border: none; \
                 color: {};",
                if is_archived { "var(--text-tertiary)" } else { "var(--text-primary)" },
            )
        >
            <div style="min-width: 0; flex: 1;">
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Large
                    full_width=true
                    on_click=Box::new(move |_| on_click())
                >
                    <span class="flex items-center justify-between w-full">
                        <span class="flex flex-col gap-1" style="min-width: 0; flex: 1;">
                            <span class="type-body-strong truncate" style=format!(
                                "{}",
                                if is_archived { "font-style: italic;" } else { "" },
                            )>
                                {workspace.name}
                            </span>
                            <span class="flex items-center gap-3">
                                {(!provider_dots.is_empty()).then(|| view! {
                                    <span class="flex items-center gap-1">
                                        {provider_dots.into_iter().map(|provider| {
                                            view! {
                                                <span
                                                    style=format!(
                                                        "display: inline-block; width: 6px; height: 6px; border-radius: var(--radius-full); background: {};",
                                                        provider.solid_color(),
                                                    )
                                                    title=provider.label()
                                                />
                                            }
                                        }).collect_view()}
                                    </span>
                                })}
                                <span class="type-caption text-tertiary">{time_label}</span>
                            </span>
                        </span>
                        <span class="flex items-center gap-2" style="flex-shrink: 0;">
                            <Badge>{mode_label}</Badge>
                            {is_archived.then(|| view! {
                                <Badge variant=BadgeVariant::Neutral>"Archived"</Badge>
                            })}
                        </span>
                    </span>
                </Button>
            </div>
            <div class="flex items-center gap-2" style="flex-shrink: 0;">
                <span class="workspace-row__actions flex items-center gap-1">
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        aria_label="Rename workspace".to_string()
                        on_click=Box::new(move |_| on_rename())
                    >
                        <Icon kind=IconKind::Pencil size=16 />
                    </Button>
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        aria_label="Duplicate workspace".to_string()
                        on_click=Box::new(move |_| on_duplicate())
                    >
                        <Icon kind=IconKind::Duplicate size=16 />
                    </Button>
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        aria_label=if is_archived { "Restore workspace".to_string() } else { "Archive workspace".to_string() }
                        on_click=Box::new(move |_| on_archive())
                    >
                        <Icon kind=IconKind::ArchiveBox size=16 />
                    </Button>
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        aria_label="Delete workspace".to_string()
                        on_click=Box::new(move |_| on_delete())
                    >
                        <Icon kind=IconKind::Trash size=16 color="var(--status-error-solid)".to_string() />
                    </Button>
                </span>
            </div>
        </div>
    }
}

/// Short label for orchestration mode (used in compact badge).
fn orchestration_mode_short(mode: &OrchestrationMode) -> &'static str {
    match mode {
        OrchestrationMode::Broadcast => "Broadcast",
        OrchestrationMode::Directed => "Directed",
        OrchestrationMode::RelayToOne => "Relay",
        OrchestrationMode::RelayToMany => "Relay",
        OrchestrationMode::DraftOnly => "Draft",
        OrchestrationMode::CopyOnly => "Copy",
        OrchestrationMode::Roundtable => "Roundtable",
        OrchestrationMode::ModeratorJury => "Jury",
        OrchestrationMode::RelayChain => "Chain",
        OrchestrationMode::ModeratedAutonomous => "Auto",
    }
}

/// Convert a UTC timestamp to a human-readable relative time string.
fn relative_time(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 30 {
        format!("{}d ago", duration.num_days())
    } else if duration.num_days() < 365 {
        format!("{}mo ago", duration.num_days() / 30)
    } else {
        format!("{}y ago", duration.num_days() / 365)
    }
}
