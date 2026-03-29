//! Workspace header component (§3.2).
//!
//! Top of the active workspace view, always visible, never scrolls.
//! Sidebar: two rows stacked (~80px). Full-tab: single row (~56px).

use leptos::prelude::*;

use crate::components::primitives::badge::Badge;
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::icon::{Icon, IconKind};
use crate::layout::responsive::LayoutMode;
use crate::models::Workspace;
use crate::models::{ContextStrategy, OrchestrationMode, Run, RunStatus};

fn orchestration_mode_label(mode: OrchestrationMode) -> &'static str {
    match mode {
        OrchestrationMode::Broadcast => "Broadcast",
        OrchestrationMode::Directed => "Directed",
        OrchestrationMode::RelayToOne => "Relay to One",
        OrchestrationMode::RelayToMany => "Relay to Many",
        OrchestrationMode::DraftOnly => "Draft Only",
        OrchestrationMode::CopyOnly => "Copy Only",
        OrchestrationMode::Roundtable => "Roundtable",
        OrchestrationMode::ModeratorJury => "Moderator Jury",
        OrchestrationMode::RelayChain => "Relay Chain",
        OrchestrationMode::ModeratedAutonomous => "Moderated Autonomous",
    }
}

fn context_strategy_label(strategy: &ContextStrategy) -> &'static str {
    match strategy {
        ContextStrategy::WorkspaceDefault => "Workspace Default",
        ContextStrategy::FullHistory => "Full History",
        ContextStrategy::LastN { .. } => "Last N",
        ContextStrategy::SpecificRange { .. } => "Specific Range",
        ContextStrategy::PinnedSummary { .. } => "Pinned Summary",
        ContextStrategy::None => "None",
    }
}

/// Workspace header component.
#[component]
pub fn WorkspaceHeader(
    /// Workspace data.
    workspace: Workspace,
    /// Active run, if any.
    run: Option<Run>,
    /// Called when back button is clicked (sidebar only).
    on_back: impl Fn() + 'static + Copy + Send,
    /// Called when provider settings should be shown.
    on_manage_providers: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    let layout_mode = expect_context::<ReadSignal<LayoutMode>>();
    let sidebar_workspace = workspace.clone();
    let sidebar_run = run.clone();
    let full_tab_workspace = workspace;
    let full_tab_run = run;

    view! {
        <header
            class="workspace-header select-none"
            style=move || format!(
                "background: var(--surface-raised); \
                 border-bottom: 1px solid var(--border-subtle); \
                 padding: var(--space-5) var(--space-6); \
                 min-height: {};",
                match layout_mode.get() {
                    LayoutMode::Sidebar => "80px",
                    LayoutMode::FullTab => "56px",
                },
            )
        >
            {move || match layout_mode.get() {
                LayoutMode::Sidebar => view! {
                    <SidebarHeader
                        workspace=sidebar_workspace.clone()
                        run=sidebar_run.clone()
                        on_back=on_back
                        on_manage_providers=on_manage_providers
                    />
                }.into_any(),
                LayoutMode::FullTab => view! {
                    <FullTabHeader
                        workspace=full_tab_workspace.clone()
                        run=full_tab_run.clone()
                        on_back=on_back
                        on_manage_providers=on_manage_providers
                    />
                }.into_any(),
            }}
        </header>
    }
}

/// Sidebar header: two rows stacked.
#[component]
fn SidebarHeader(
    workspace: Workspace,
    run: Option<Run>,
    on_back: impl Fn() + 'static + Copy + Send,
    on_manage_providers: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-3">
            // Row 1: Back + name + overflow
            <div class="flex items-center gap-3">
                <Button
                    variant=ButtonVariant::Icon
                    size=ButtonSize::Small
                    aria_label="Back to workspace list".to_string()
                    on_click=Box::new(move |_| on_back())
                >
                    <Icon kind=IconKind::ArrowLeft size=18 />
                </Button>
                <span class="type-title text-primary truncate" style="max-width: 220px;">
                    {workspace.name.clone()}
                </span>
            </div>

            // Row 2: Mode + Strategy + Run status
            <div class="flex items-center gap-2 flex-wrap">
                <Badge>{orchestration_mode_label(workspace.default_mode)}</Badge>
                <Badge>{context_strategy_label(&workspace.default_context_strategy)}</Badge>
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Small
                    on_click=Box::new(move |_| on_manage_providers())
                >
                    "Providers"
                </Button>
                {run.map(|r| view! { <RunStatusIndicator run=r /> })}
            </div>
        </div>
    }
}

/// Full-tab header: single row with back button.
#[component]
fn FullTabHeader(
    workspace: Workspace,
    run: Option<Run>,
    on_back: impl Fn() + 'static + Copy + Send,
    on_manage_providers: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between">
            // Left: back + name
            <div class="flex items-center gap-3">
                <Button
                    variant=ButtonVariant::Icon
                    size=ButtonSize::Small
                    aria_label="Back to workspace list".to_string()
                    on_click=Box::new(move |_| on_back())
                >
                    <Icon kind=IconKind::ArrowLeft size=18 />
                </Button>
                <span class="type-display text-primary">
                    {workspace.name.clone()}
                </span>
            </div>

            // Center: mode + strategy
            <div class="flex items-center gap-3">
                <Badge>{orchestration_mode_label(workspace.default_mode)}</Badge>
                <Badge>{context_strategy_label(&workspace.default_context_strategy)}</Badge>
            </div>

            // Right: run status
            <div class="flex items-center gap-3">
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Small
                    on_click=Box::new(move |_| on_manage_providers())
                >
                    "Providers"
                </Button>
                {run.map(|r| view! { <RunStatusIndicator run=r /> })}
            </div>
        </div>
    }
}

/// Run status indicator (dot + label).
#[component]
fn RunStatusIndicator(run: Run) -> impl IntoView {
    let (dot_color, label) = match run.status {
        RunStatus::Created => return view! {}.into_any(),
        RunStatus::Running => (
            "var(--status-success-solid)",
            "Running".to_string(),
        ),
        RunStatus::Paused => (
            "var(--status-warning-solid)",
            "Paused".to_string(),
        ),
        RunStatus::Completed => (
            "var(--status-info-solid)",
            "Completed".to_string(),
        ),
        RunStatus::Aborted => (
            "var(--status-error-solid)",
            "Aborted".to_string(),
        ),
    };

    view! {
        <div class="flex items-center gap-2">
            <span style=format!(
                "display: inline-block; width: 8px; height: 8px; \
                 border-radius: var(--radius-full); background: {};{}",
                dot_color,
                if run.status == RunStatus::Running { " animation: pulse 2s infinite;" } else { "" },
            ) />
            <span class="type-caption-strong text-primary">{label}</span>
        </div>
    }
    .into_any()
}
