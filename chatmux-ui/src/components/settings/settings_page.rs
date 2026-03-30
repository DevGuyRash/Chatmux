//! Settings page component (§3.15).
//!
//! Vertical scrolling page with sections: Appearance, Timing Defaults,
//! Per-Provider Overrides, Orchestration Defaults, Storage, Automation Safety,
//! Keyboard Shortcuts.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::number_input::NumberInput;
use crate::components::primitives::segmented_control::{Segment, SegmentedControl};
use crate::components::primitives::toggle::Toggle;
use crate::layout::responsive::LayoutMode;

/// Settings page component.
#[component]
pub fn SettingsPage() -> impl IntoView {
    let layout_mode = expect_context::<ReadSignal<LayoutMode>>();

    // TODO(backend): Load settings from storage.local on mount.
    // TODO(backend): Save settings to storage.local on change.

    // Local state for settings (will be synced with bridge)
    let (theme, set_theme) = signal("dark".to_string());
    let (surface, set_surface) = signal("sidebar".to_string());
    let (gen_timeout, set_gen_timeout) = signal(120.0);
    let (cooldown, set_cooldown) = signal(2.0);
    let (inter_round, set_inter_round) = signal(5.0);
    let (jitter_on, set_jitter_on) = signal(true);
    let (jitter_pct, set_jitter_pct) = signal(20.0);
    let (max_concurrent, set_max_concurrent) = signal(4.0);
    let (max_rounds, set_max_rounds) = signal(20.0);
    let (run_timeout, set_run_timeout) = signal(60.0);
    let (kill_switch, set_kill_switch) = signal(false);

    // TODO(backend): Get storage usage from bridge.
    let (storage_used, _set_used) = signal(0u64);
    let (storage_total, _set_total) = signal(0u64);

    view! {
        <div
            class="settings-page overflow-y-auto h-full"
            style=move || format!(
                "padding: var(--space-7); {} ",
                match layout_mode.get() {
                    LayoutMode::Sidebar => "",
                    LayoutMode::FullTab => "max-width: 680px; margin: 0 auto;",
                },
            )
        >
            <h1 class="type-title text-primary mb-7">
                "Settings"
            </h1>

            // §1 — Appearance
            <Section title="Appearance">
                <FieldRow label="Theme">
                    <SegmentedControl
                        segments=vec![
                            Segment { value: "dark".into(), label: "Dark".into() },
                            Segment { value: "light".into(), label: "Light".into() },
                            Segment { value: "system".into(), label: "System".into() },
                        ]
                        selected=theme
                        on_change=move |v| set_theme.set(v)
                    />
                </FieldRow>
                <FieldRow label="Default surface">
                    <SegmentedControl
                        segments=vec![
                            Segment { value: "sidebar".into(), label: "Sidebar".into() },
                            Segment { value: "full-tab".into(), label: "Full Tab".into() },
                        ]
                        selected=surface
                        on_change=move |v| set_surface.set(v)
                    />
                </FieldRow>
            </Section>

            // §2 — Timing Defaults
            <Section title="Timing Defaults">
                <FieldRow label="Generation timeout">
                    <NumberInput value=gen_timeout on_change=move |v| set_gen_timeout.set(v) min=5.0 suffix="s" />
                </FieldRow>
                <FieldRow label="Cooldown">
                    <NumberInput value=cooldown on_change=move |v| set_cooldown.set(v) min=0.0 suffix="s" />
                </FieldRow>
                <FieldRow label="Inter-round delay">
                    <NumberInput value=inter_round on_change=move |v| set_inter_round.set(v) min=0.0 suffix="s" />
                </FieldRow>
                <FieldRow label="Jitter">
                    <div class="flex items-center gap-3">
                        <Toggle checked=jitter_on on_change=move |v| set_jitter_on.set(v) />
                        <NumberInput value=jitter_pct on_change=move |v| set_jitter_pct.set(v) min=0.0 max=100.0 suffix="%" />
                    </div>
                </FieldRow>
                <FieldRow label="Max concurrent sends">
                    <NumberInput value=max_concurrent on_change=move |v| set_max_concurrent.set(v) min=1.0 max=10.0 />
                </FieldRow>
                <FieldRow label="Max rounds per run">
                    <NumberInput value=max_rounds on_change=move |v| set_max_rounds.set(v) min=1.0 />
                </FieldRow>
                <FieldRow label="Global run timeout">
                    <NumberInput value=run_timeout on_change=move |v| set_run_timeout.set(v) min=1.0 suffix="m" />
                </FieldRow>
            </Section>

            // §5 — Storage
            <Section title="Storage">
                <div class="flex flex-col gap-3">
                    <div class="flex items-center gap-3">
                        <div style="flex: 1; height: 8px; background: var(--surface-sunken); border-radius: var(--radius-full); overflow: hidden;">
                            <div style=move || {
                                let pct = if storage_total.get() > 0 {
                                    (storage_used.get() as f64 / storage_total.get() as f64 * 100.0).min(100.0)
                                } else { 0.0 };
                                format!("width: {}%; height: 100%; background: var(--accent-primary); border-radius: var(--radius-full);", pct)
                            } />
                        </div>
                        <span class="type-caption text-secondary">
                            {move || format!("{} / {} bytes", storage_used.get(), storage_total.get())}
                        </span>
                    </div>
                    <Button variant=ButtonVariant::Danger>
                        "Clear All Data"
                    </Button>
                </div>
            </Section>

            // §6 — Automation Safety
            <Section title="Automation Safety">
                <div
                    class="flex items-center justify-between p-5"
                    style=move || format!(
                        "border-radius: var(--radius-md); background: {};",
                        if kill_switch.get() { "var(--status-error-muted)" } else { "var(--surface-sunken)" },
                    )
                >
                    <div class="flex flex-col gap-1">
                        <span class="type-body-strong text-primary">"Kill Switch"</span>
                        <span class="type-caption text-secondary">"Immediately halt all orchestration activity"</span>
                    </div>
                    <Toggle
                        checked=kill_switch
                        on_change=move |v| {
                            set_kill_switch.set(v);
                            // TODO(backend): Call activate_kill_switch or deactivate_kill_switch
                        }
                        aria_label="Kill switch"
                    />
                </div>
            </Section>

            // §7 — Keyboard Shortcuts
            <Section title="Keyboard Shortcuts">
                <p class="type-body text-secondary">
                    "Keyboard shortcuts are configured through your browser's extension settings."
                </p>
                // TODO(backend): Load and display registered commands from the commands API
            </Section>
        </div>
    }
}

/// Section wrapper with title.
#[component]
fn Section(title: &'static str, children: Children) -> impl IntoView {
    let rendered = children();
    view! {
        <section class="mb-9">
            <h2 class="type-subtitle text-primary mb-5">
                {title}
            </h2>
            {rendered}
        </section>
    }
}

/// Field row — label + control.
#[component]
fn FieldRow(label: &'static str, children: Children) -> impl IntoView {
    let rendered = children();
    view! {
        <div
            class="flex items-center justify-between"
            style="padding: var(--space-3) 0; min-height: 40px;"
        >
            <span class="type-body text-primary">{label}</span>
            {rendered}
        </div>
    }
}
