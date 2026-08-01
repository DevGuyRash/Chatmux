//! Run controls bar (§3.7).
//!
//! Appears below workspace header when a run is active or can be started.
//! Sticky, does not scroll with messages.
//! State-dependent button visibility per §3.7 table.

use leptos::prelude::*;

use crate::components::primitives::badge::Badge;
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::icon::{Icon, IconKind};
use crate::layout::responsive::LayoutMode;
use crate::models::{BarrierPolicy, RunStatus};

/// Run controls bar component.
#[component]
pub fn RunControlsBar(
    /// Current run state.
    run_state: Signal<RunStatus>,
    /// Current round number.
    current_round: Signal<u32>,
    /// Maximum rounds (if set).
    max_rounds: Signal<Option<u32>>,
    /// Active barrier policy.
    barrier_policy: Signal<BarrierPolicy>,
    /// Callbacks for run actions.
    on_start: impl Fn() + 'static + Copy + Send,
    on_pause: impl Fn() + 'static + Copy + Send,
    on_resume: impl Fn() + 'static + Copy + Send,
    on_edit_packages: impl Fn() + 'static + Copy + Send,
    on_step: impl Fn() + 'static + Copy + Send,
    on_stop: impl Fn() + 'static + Copy + Send,
    on_abort: impl Fn() + 'static + Copy + Send,
    on_new_run: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    let is_sidebar = expect_context::<LayoutMode>() == LayoutMode::Sidebar;

    view! {
        <div
            class="run-controls-bar flex items-center justify-between surface-raised select-none border-t border-b"
            style="\
                padding: var(--space-4) var(--space-5); \
                min-height: 44px;"
        >
            // Left: action buttons (state-dependent)
            <div class="flex items-center gap-2">
                {move || match run_state.get() {
                    RunStatus::Created => view! {
                        <Button variant=ButtonVariant::Primary size=ButtonSize::Small
                                aria_label="Start run".to_owned()
                                on_click=Box::new(move |_| on_start())>
                            <Icon kind=IconKind::Play size=14 />
                            {(!is_sidebar).then_some("Start run")}
                        </Button>
                    }.into_any(),

                    RunStatus::Running => view! {
                        <div class="flex gap-2">
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    aria_label="Pause run".to_owned()
                                    on_click=Box::new(move |_| on_pause())>
                                <Icon kind=IconKind::Pause size=14 />
                                {(!is_sidebar).then_some("Pause")}
                            </Button>
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    aria_label="Advance one run step".to_owned()
                                    on_click=Box::new(move |_| on_step())>
                                <Icon kind=IconKind::Step size=14 />
                                {(!is_sidebar).then_some("Step")}
                            </Button>
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    aria_label="Stop run".to_owned()
                                    on_click=Box::new(move |_| on_stop())>
                                <Icon kind=IconKind::Stop size=14 />
                                {(!is_sidebar).then_some("Stop")}
                            </Button>
                            <Button variant=ButtonVariant::Danger size=ButtonSize::Small
                                    aria_label="Abort run immediately".to_owned()
                                    on_click=Box::new(move |_| on_abort())>
                                <Icon kind=IconKind::StopOctagon size=14 />
                                {(!is_sidebar).then_some("Abort")}
                            </Button>
                        </div>
                    }.into_any(),

                    RunStatus::Paused => view! {
                        <div class="flex gap-2">
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    aria_label="Review and edit next-round packages".to_owned()
                                    on_click=Box::new(move |_| on_edit_packages())>
                                <Icon kind=IconKind::Pencil size=14 />
                                {(!is_sidebar).then_some("Edit packages")}
                            </Button>
                            <Button variant=ButtonVariant::Primary size=ButtonSize::Small
                                    aria_label="Resume run".to_owned()
                                    on_click=Box::new(move |_| on_resume())>
                                <Icon kind=IconKind::Play size=14 />
                                {(!is_sidebar).then_some("Resume")}
                            </Button>
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    aria_label="Advance one run step".to_owned()
                                    on_click=Box::new(move |_| on_step())>
                                <Icon kind=IconKind::Step size=14 />
                                {(!is_sidebar).then_some("Step")}
                            </Button>
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    aria_label="Stop run".to_owned()
                                    on_click=Box::new(move |_| on_stop())>
                                <Icon kind=IconKind::Stop size=14 />
                                {(!is_sidebar).then_some("Stop")}
                            </Button>
                            <Button variant=ButtonVariant::Danger size=ButtonSize::Small
                                    aria_label="Abort run immediately".to_owned()
                                    on_click=Box::new(move |_| on_abort())>
                                <Icon kind=IconKind::StopOctagon size=14 />
                                {(!is_sidebar).then_some("Abort")}
                            </Button>
                        </div>
                    }.into_any(),

                    RunStatus::Completed | RunStatus::Aborted => view! {
                        <Button variant=ButtonVariant::Primary size=ButtonSize::Small
                                aria_label="Start a new run".to_owned()
                                on_click=Box::new(move |_| on_new_run())>
                            <Icon kind=IconKind::Play size=14 />
                            {if is_sidebar { "New" } else { "New run" }}
                        </Button>
                    }.into_any(),
                }}
            </div>

            // Center: round counter + barrier policy
            <div class="flex items-center gap-3">
                <span class="type-body-strong text-primary">
                    {move || {
                        let round = current_round.get();
                        match max_rounds.get() {
                            Some(max) => format!("Round {round} / {max}"),
                            None => format!("Round {round}"),
                        }
                    }}
                </span>
                <Badge>{move || crate::models::view_models::barrier_policy_label(&barrier_policy.get())}</Badge>
            </div>
        </div>
    }
}
