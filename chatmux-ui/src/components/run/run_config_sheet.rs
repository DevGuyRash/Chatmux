//! Run configuration sheet (§3.24).
//!
//! Appears when starting a new run. Sidebar: full-width overlay.
//! Full-tab: 720px centered modal. Contains: orchestration mode,
//! mode-specific config, participant set, barrier policy, timing,
//! stop conditions, routing summary.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::modal::Modal;
use crate::layout::responsive::LayoutMode;
use crate::models::OrchestrationMode;

/// Run configuration sheet.
#[component]
pub fn RunConfigSheet(
    /// Whether the sheet is open.
    open: ReadSignal<bool>,
    /// Called to close without starting.
    on_cancel: impl Fn() + 'static + Copy + Send,
    /// Called to start the run.
    on_start: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    let _layout_mode = expect_context::<ReadSignal<LayoutMode>>();
    let (selected_mode, set_selected_mode) = signal(OrchestrationMode::Broadcast);
    let (moderated, set_moderated) = signal(true);

    let modes_manual = [
        OrchestrationMode::Broadcast,
        OrchestrationMode::Directed,
        OrchestrationMode::RelayToOne,
        OrchestrationMode::RelayToMany,
        OrchestrationMode::DraftOnly,
        OrchestrationMode::CopyOnly,
    ];
    let modes_autonomous = [
        OrchestrationMode::Roundtable,
        OrchestrationMode::ModeratorJury,
        OrchestrationMode::ModeratedAutonomous,
        OrchestrationMode::RelayChain,
    ];

    view! {
        <Modal open=open on_close=on_cancel max_width=720>
            <div class="flex flex-col gap-7" style="max-height: 80vh; overflow-y: auto;">
                <h2 class="type-title text-primary">"Configure Run"</h2>

                // §1 — Orchestration Mode
                <section>
                    <h3 class="type-subtitle text-primary mb-4">
                        "Orchestration Mode"
                    </h3>

                    // Manual modes
                    <p class="type-caption text-secondary mb-3">"Manual"</p>
                    <div class="flex flex-col gap-2 mb-5">
                        {modes_manual.iter().map(|&mode| {
                            view! { <ModeCard mode=mode selected=selected_mode on_select=move |m| set_selected_mode.set(m) /> }
                        }).collect_view()}
                    </div>

                    // Autonomous modes
                    <p class="type-caption text-secondary mb-3">"Autonomous"</p>
                    <div class="flex flex-col gap-2">
                        {modes_autonomous.iter().map(|&mode| {
                            view! { <ModeCard mode=mode selected=selected_mode on_select=move |m| set_selected_mode.set(m) /> }
                        }).collect_view()}
                    </div>

                    // Moderated toggle (visible for autonomous modes)
                    {move || mode_is_autonomous(selected_mode.get()).then(|| view! {
                        <div class="flex items-center gap-3 mt-4">
                            <crate::components::primitives::toggle::Toggle
                                checked=moderated
                                on_change=move |v| set_moderated.set(v)
                                aria_label="Moderated mode"
                            />
                            <span class="type-body text-primary">"Moderated — pause between rounds for review"</span>
                        </div>
                    })}
                </section>

                // TODO: §2 Mode-specific config, §3 Participant set, §4 Barrier policy,
                // §5 Timing controls, §6 Stop conditions, §7 Routing summary
                // These will be implemented using the same primitives pattern.
                <section>
                    <p class="type-caption text-tertiary">
                        "Additional configuration sections (participant set, barrier policy, timing, stop conditions, routing) will be populated as backend integration matures."
                    </p>
                </section>

                // §8 — Action row
                <div class="flex justify-end gap-3">
                    <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| on_cancel())>
                        "Cancel"
                    </Button>
                    <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| on_start())>
                        "Start Run"
                    </Button>
                </div>
            </div>
        </Modal>
    }
}

/// Mode selection card.
#[component]
fn ModeCard(
    mode: OrchestrationMode,
    selected: ReadSignal<OrchestrationMode>,
    on_select: impl Fn(OrchestrationMode) + 'static + Copy,
) -> impl IntoView {
    view! {
        <button
            class="flex flex-col gap-1 w-full cursor-pointer select-none text-left transition-colors"
            style=move || format!(
                "padding: var(--space-4) var(--space-5); \
                 border-radius: var(--radius-md); \
                 border: 1px solid {}; \
                 background: {};",
                if selected.get() == mode { "var(--border-accent)" } else { "var(--border-default)" },
                if selected.get() == mode { "var(--surface-selected)" } else { "var(--surface-raised)" },
            )
            on:click=move |_| on_select(mode)
        >
            <span class="type-subtitle text-primary">{mode_label(mode)}</span>
            <span class="type-body text-secondary">{mode_description(mode)}</span>
        </button>
    }
}

fn mode_label(m: OrchestrationMode) -> &'static str {
    match m {
        OrchestrationMode::Broadcast => "Broadcast",
        OrchestrationMode::Directed => "Directed",
        OrchestrationMode::RelayToOne => "Relay to One",
        OrchestrationMode::RelayToMany => "Relay to Many",
        OrchestrationMode::DraftOnly => "Draft Only",
        OrchestrationMode::CopyOnly => "Copy Only",
        OrchestrationMode::Roundtable => "Roundtable",
        OrchestrationMode::ModeratorJury => "Moderator / Jury",
        OrchestrationMode::RelayChain => "Relay Chain",
        OrchestrationMode::ModeratedAutonomous => "Moderated Autonomous",
    }
}

fn mode_description(m: OrchestrationMode) -> &'static str {
    match m {
        OrchestrationMode::Broadcast => "Send to all participants simultaneously.",
        OrchestrationMode::Directed => "Send to one specific participant.",
        OrchestrationMode::RelayToOne => "Forward output of one participant to another.",
        OrchestrationMode::RelayToMany => "Forward output of one participant to several.",
        OrchestrationMode::DraftOnly => "Compose a draft without sending.",
        OrchestrationMode::CopyOnly => "Copy content without routing.",
        OrchestrationMode::Roundtable => "Participants respond in turn, autonomously.",
        OrchestrationMode::ModeratorJury => "A moderator routes turns between participants.",
        OrchestrationMode::RelayChain => "Output chains through a sequence of participants.",
        OrchestrationMode::ModeratedAutonomous => "Autonomous with moderation checkpoints.",
    }
}

fn mode_is_autonomous(m: OrchestrationMode) -> bool {
    matches!(
        m,
        OrchestrationMode::Roundtable
            | OrchestrationMode::ModeratorJury
            | OrchestrationMode::RelayChain
            | OrchestrationMode::ModeratedAutonomous
    )
}
