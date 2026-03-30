//! Run controls bar (§3.7).
//!
//! Appears below workspace header when a run is active or can be started.
//! Sticky, does not scroll with messages.
//! State-dependent button visibility per §3.7 table.

use leptos::prelude::*;

use crate::components::primitives::badge::Badge;
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
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
    on_step: impl Fn() + 'static + Copy + Send,
    on_stop: impl Fn() + 'static + Copy + Send,
    on_abort: impl Fn() + 'static + Copy + Send,
    on_new_run: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    let layout_mode = expect_context::<ReadSignal<LayoutMode>>();
    let is_sidebar = Signal::derive(move || layout_mode.get() == LayoutMode::Sidebar);

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
                                on_click=Box::new(move |_| on_start())>
                            {if is_sidebar.get() { "▶" } else { "▶ Start Run" }}
                        </Button>
                    }.into_any(),

                    RunStatus::Running => view! {
                        <div class="flex gap-2">
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    on_click=Box::new(move |_| on_pause())>
                                {if is_sidebar.get() { "⏸" } else { "⏸ Pause" }}
                            </Button>
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    on_click=Box::new(move |_| on_step())>
                                {if is_sidebar.get() { "⏭" } else { "⏭ Step" }}
                            </Button>
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    on_click=Box::new(move |_| on_stop())>
                                {if is_sidebar.get() { "⏹" } else { "⏹ Stop" }}
                            </Button>
                            <Button variant=ButtonVariant::Danger size=ButtonSize::Small
                                    on_click=Box::new(move |_| on_abort())>
                                {if is_sidebar.get() { "⛔" } else { "⛔ Abort" }}
                            </Button>
                        </div>
                    }.into_any(),

                    RunStatus::Paused => view! {
                        <div class="flex gap-2">
                            <Button variant=ButtonVariant::Primary size=ButtonSize::Small
                                    on_click=Box::new(move |_| on_resume())>
                                {if is_sidebar.get() { "▶" } else { "▶ Resume" }}
                            </Button>
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    on_click=Box::new(move |_| on_step())>
                                {if is_sidebar.get() { "⏭" } else { "⏭ Step" }}
                            </Button>
                            <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                                    on_click=Box::new(move |_| on_stop())>
                                {if is_sidebar.get() { "⏹" } else { "⏹ Stop" }}
                            </Button>
                        </div>
                    }.into_any(),

                    RunStatus::Completed | RunStatus::Aborted => view! {
                        <Button variant=ButtonVariant::Primary size=ButtonSize::Small
                                on_click=Box::new(move |_| on_new_run())>
                            {if is_sidebar.get() { "▶ New" } else { "▶ New Run" }}
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
                <Badge>{move || match barrier_policy.get() {
                    BarrierPolicy::WaitForAll => "Wait for All",
                    BarrierPolicy::Quorum { .. } => "Quorum",
                    BarrierPolicy::FirstFinisher => "First Finisher",
                    BarrierPolicy::ManualAdvance => "Manual Advance",
                }}</Badge>
            </div>
        </div>
    }
}
