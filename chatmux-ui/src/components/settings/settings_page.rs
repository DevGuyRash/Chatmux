//! Settings page component (§3.15).
//!
//! Vertical scrolling page with sections: Appearance, Timing Defaults,
//! Per-Provider Overrides, Orchestration Defaults, Storage, Automation Safety,
//! Keyboard Shortcuts.

use leptos::prelude::*;

use crate::bridge::storage::SurfacePreference;
use crate::bridge::{messaging, storage};
use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::number_input::NumberInput;
use crate::components::primitives::segmented_control::{Segment, SegmentedControl};
use crate::components::primitives::toggle::Toggle;
use crate::layout::responsive::LayoutMode;
use crate::state::{
    app_state::AppState, binding_state::BindingState, diagnostics_state::DiagnosticsState,
    message_state::MessageState, run_state::ActiveRunState, workspace_state::WorkspaceListState,
};
use crate::theme::{ThemePreference, use_theme};

/// Settings page component.
#[component]
pub fn SettingsPage() -> impl IntoView {
    let layout_mode = expect_context::<LayoutMode>();
    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();
    let theme_context = use_theme();

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
    // These values are written by extension API futures that can complete
    // after a responsive shell transition unmounts this page. Arc signals
    // remain valid until those futures release their final clone.
    let storage_used = ArcRwSignal::new(0u64);
    let storage_total = ArcRwSignal::new(0u64);
    let (storage_loaded, set_storage_loaded) = signal(false);
    let (confirm_clear, set_confirm_clear) = signal(false);
    let (archive_body, set_archive_body) = signal(String::new());
    let shortcuts = ArcRwSignal::new(Vec::<(String, String, Option<String>)>::new());
    let (shortcuts_loaded, set_shortcuts_loaded) = signal(false);

    Effect::new(move |_| {
        let settings = app_state.ui_settings.get();
        set_theme.set(theme_preference_value(settings.theme).to_owned());
        set_surface.set(surface_preference_value(settings.surface_preference).to_owned());
        set_gen_timeout.set(settings.timing.per_provider_generation_timeout_secs as f64);
        set_cooldown.set(settings.timing.per_provider_cooldown_secs as f64);
        set_inter_round.set(settings.timing.inter_round_delay_secs as f64);
        set_jitter_on.set(settings.timing.jitter_percent > 0);
        set_jitter_pct.set(settings.timing.jitter_percent as f64);
        set_max_concurrent.set(settings.timing.max_concurrent_sends as f64);
        set_max_rounds.set(settings.timing.max_rounds.unwrap_or(20) as f64);
        set_run_timeout.set(
            settings
                .timing
                .global_run_timeout_secs
                .map(|seconds| seconds as f64 / 60.0)
                .unwrap_or(60.0),
        );
    });

    let shortcuts_for_load = shortcuts.clone();
    Effect::new(move |_| {
        if shortcuts_loaded.get() {
            return;
        }
        set_shortcuts_loaded.set(true);
        let shortcuts = shortcuts_for_load.clone();
        leptos::task::spawn_local_scoped_with_cancellation(async move {
            let commands = crate::bridge::commands::get_commands()
                .await
                .into_iter()
                .map(|command| (command.name, command.description, command.shortcut))
                .collect();
            shortcuts.set(commands);
        });
    });

    let storage_used_for_load = storage_used.clone();
    let storage_total_for_load = storage_total.clone();
    Effect::new(move |_| {
        if storage_loaded.get() {
            return;
        }
        set_storage_loaded.set(true);
        let storage_used = storage_used_for_load.clone();
        let storage_total = storage_total_for_load.clone();
        leptos::task::spawn_local_scoped_with_cancellation(async move {
            let (used, total) = storage::get_storage_usage().await;
            storage_used.set(used);
            storage_total.set(total);
        });
    });

    let storage_used_for_bar = storage_used.clone();
    let storage_total_for_bar = storage_total.clone();
    let storage_used_for_label = storage_used.clone();
    let storage_total_for_label = storage_total.clone();
    let shortcuts_for_view = shortcuts.clone();

    view! {
        <div
            class="settings-page overflow-y-auto h-full"
            style=format!(
                "padding: var(--space-7); {} ",
                match layout_mode {
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
                        on_change=move |value| {
                            let preference = match value.as_str() {
                                "light" => ThemePreference::Light,
                                "system" => ThemePreference::System,
                                _ => ThemePreference::Dark,
                            };
                            theme_context.set_preference.set(preference);
                            app_state.set_ui_settings.update(|settings| settings.theme = preference);
                        }
                    />
                </FieldRow>
                <FieldRow label="Default surface">
                    <SegmentedControl
                        segments=vec![
                            Segment { value: "sidebar".into(), label: "Sidebar".into() },
                            Segment { value: "full-tab".into(), label: "Full tab".into() },
                        ]
                        selected=surface
                        on_change=move |value| {
                            let preference = if value == "full-tab" {
                                SurfacePreference::FullTab
                            } else {
                                SurfacePreference::Sidebar
                            };
                            app_state.set_ui_settings.update(|settings| {
                                settings.surface_preference = preference;
                            });
                        }
                    />
                </FieldRow>
            </Section>

            // §2 — Timing Defaults
            <Section title="Timing Defaults">
                <FieldRow label="Generation timeout">
                    <NumberInput value=gen_timeout on_change=move |value| app_state.set_ui_settings.update(|settings| settings.timing.per_provider_generation_timeout_secs = value.round() as u64) min=5.0 suffix="s" />
                </FieldRow>
                <FieldRow label="Cooldown">
                    <NumberInput value=cooldown on_change=move |value| app_state.set_ui_settings.update(|settings| settings.timing.per_provider_cooldown_secs = value.round() as u64) min=0.0 suffix="s" />
                </FieldRow>
                <FieldRow label="Inter-round delay">
                    <NumberInput value=inter_round on_change=move |value| app_state.set_ui_settings.update(|settings| settings.timing.inter_round_delay_secs = value.round() as u64) min=0.0 suffix="s" />
                </FieldRow>
                <FieldRow label="Jitter">
                    <div class="flex items-center gap-3">
                        <Toggle checked=jitter_on on_change=move |enabled| app_state.set_ui_settings.update(|settings| {
                            settings.timing.jitter_percent = if enabled {
                                settings.timing.jitter_percent.max(20)
                            } else {
                                0
                            };
                        }) />
                        <NumberInput value=jitter_pct on_change=move |value| app_state.set_ui_settings.update(|settings| settings.timing.jitter_percent = value.round() as u8) min=0.0 max=100.0 suffix="%" />
                    </div>
                </FieldRow>
                <FieldRow label="Max concurrent sends">
                    <NumberInput value=max_concurrent on_change=move |value| app_state.set_ui_settings.update(|settings| settings.timing.max_concurrent_sends = value.round() as usize) min=1.0 max=10.0 />
                </FieldRow>
                <FieldRow label="Max rounds per run">
                    <NumberInput value=max_rounds on_change=move |value| app_state.set_ui_settings.update(|settings| settings.timing.max_rounds = Some(value.round() as u32)) min=1.0 />
                </FieldRow>
                <FieldRow label="Global run timeout">
                    <NumberInput value=run_timeout on_change=move |value| app_state.set_ui_settings.update(|settings| settings.timing.global_run_timeout_secs = Some((value * 60.0).round() as u64)) min=1.0 suffix="m" />
                </FieldRow>
            </Section>

            <Section title="Workspace portability">
                <div class="flex flex-col gap-3">
                    <p class="type-caption text-secondary">
                        "Export a complete local workspace backup, or paste a Chatmux workspace archive to import it as a safe paused copy. Browser tab bindings are intentionally not transferred."
                    </p>
                    <textarea
                        class="type-body text-primary surface-sunken border rounded-md"
                        style="min-height: 8rem; padding: var(--space-4); resize: vertical;"
                        aria-label="Workspace archive JSON"
                        placeholder="Paste workspace archive JSON here…"
                        prop:value=move || archive_body.get()
                        on:input=move |event| set_archive_body.set(event_target_value(&event))
                    />
                    <div class="flex items-center gap-3 flex-wrap">
                        <Button variant=ButtonVariant::Secondary on_click=Box::new(move |_| {
                            let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else { return; };
                            leptos::task::spawn_local(async move {
                                let result = messaging::export_workspace_archive(workspace_id).await;
                                // Clipboard access can be denied by the browser.
                                // The outcome has to reflect what actually
                                // happened: the archive is in the box either way,
                                // but claiming a copy that never landed sends the
                                // user to paste nothing.
                                let mut copied = false;
                                if let Ok(events) = &result
                                    && let Some(body) = events.iter().find_map(|event| match event {
                                        crate::models::UiEvent::ExportRendered { body, .. } => Some(body.clone()),
                                        _ => None,
                                    })
                                {
                                    set_archive_body.set(body.clone());
                                    copied = crate::bridge::clipboard::write_clipboard(&body).await;
                                }
                                crate::state::controller::dispatch_user_command_result(
                                    app_state, workspace_state, run_state, binding_state,
                                    message_state, diagnostics_state,
                                    if copied {
                                        "Workspace archive copied to the clipboard."
                                    } else {
                                        "Workspace archive ready in the box below — copy it manually."
                                    },
                                    "Couldn't prepare the workspace archive:",
                                    result,
                                );
                            });
                        })>"Export active workspace"</Button>
                        <Button variant=ButtonVariant::Primary on_click=Box::new(move |_| {
                            let body = normalize_workspace_archive_input(&archive_body.get_untracked());
                            if body.trim().is_empty() { return; }
                            leptos::task::spawn_local(async move {
                                let confirmed = crate::state::controller::dispatch_user_command_result(
                                    app_state, workspace_state, run_state, binding_state,
                                    message_state, diagnostics_state,
                                    "Workspace imported as a safe paused copy.",
                                    "Couldn't import the workspace:",
                                    messaging::import_workspace_archive(body).await,
                                );
                                if confirmed { set_archive_body.set(String::new()); }
                            });
                        })>"Import workspace"</Button>
                    </div>
                </div>
            </Section>

            // §5 — Storage
            <Section title="Storage">
                <div class="flex flex-col gap-3">
                    <div class="flex items-center gap-3">
                        <div style="flex: 1; height: 8px; background: var(--surface-sunken); border-radius: var(--radius-full); overflow: hidden;">
                            <div style=move || {
                                let total = storage_total_for_bar.get();
                                let pct = if total > 0 {
                                    (storage_used_for_bar.get() as f64 / total as f64 * 100.0).min(100.0)
                                } else { 0.0 };
                                format!("width: {}%; height: 100%; background: var(--accent-primary); border-radius: var(--radius-full);", pct)
                            } />
                        </div>
                        <span class="type-caption text-secondary">
                            {move || if storage_total_for_label.get() > 0 {
                                format!("{} / {} bytes", storage_used_for_label.get(), storage_total_for_label.get())
                            } else {
                                format!("{} bytes used", storage_used_for_label.get())
                            }}
                        </span>
                    </div>
                    <Button
                        variant=ButtonVariant::Danger
                        disabled=Signal::derive(move || app_state.active_workspace_id.get().is_none())
                        on_click=Box::new(move |_| {
                            if !confirm_clear.get_untracked() {
                                set_confirm_clear.set(true);
                                return;
                            }
                            let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else {
                                return;
                            };
                            leptos::task::spawn_local(async move {
                                crate::state::controller::dispatch_user_command_result(
                                    app_state,
                                    workspace_state,
                                    run_state,
                                    binding_state,
                                    message_state,
                                    diagnostics_state,
                                    "Conversation history cleared.",
                                    "Couldn't clear conversation history:",
                                    messaging::clear_workspace_data(workspace_id).await,
                                );
                                set_confirm_clear.set(false);
                            });
                        })
                    >
                        // Both states name the same operation. An arming label
                        // that changes the noun ("Clear workspace") describes a
                        // different, larger action than the one that runs.
                        {move || if confirm_clear.get() {
                            "Confirm — clear conversation history"
                        } else {
                            "Clear conversation history"
                        }}
                    </Button>
                    <p class="type-caption text-tertiary mt-3">
                        "Removes messages, runs and diagnostics for the active workspace. Provider bindings, routing policies, templates and export profiles are kept."
                    </p>
                </div>
            </Section>

            // §6 — Automation Safety
            <Section title="Automation Safety">
                <div
                    class="flex items-center justify-between p-5"
                    style=move || format!(
                        "border-radius: var(--radius-md); background: {};",
                        if app_state.kill_switch_active.get() { "var(--status-error-muted)" } else { "var(--surface-sunken)" },
                    )
                >
                    <div class="flex flex-col gap-1">
                        <span class="type-body-strong text-primary">"Kill Switch"</span>
                        <span class="type-caption text-secondary">"Immediately halt all orchestration activity"</span>
                    </div>
                    <Toggle
                        checked=app_state.kill_switch_active
                        on_change=move |active| {
                            leptos::task::spawn_local(async move {
                                crate::state::controller::dispatch_user_command_result(
                                    app_state,
                                    workspace_state,
                                    run_state,
                                    binding_state,
                                    message_state,
                                    diagnostics_state,
                                    if active { "Kill switch activated." } else { "Kill switch deactivated." },
                                    if active { "Couldn't activate the kill switch:" } else { "Couldn't deactivate the kill switch:" },
                                    messaging::set_kill_switch(active).await,
                                );
                            });
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
                <div class="flex flex-col gap-2 mt-3">
                    {move || shortcuts_for_view.get().into_iter().map(|(name, description, shortcut)| view! {
                        <div class="flex items-center justify-between gap-4 surface-sunken rounded-md p-3">
                            <div class="flex flex-col gap-1">
                                <span class="type-body-strong text-primary">{description}</span>
                                <span class="type-caption text-tertiary">{name}</span>
                            </div>
                            <kbd class="type-caption text-primary border rounded-sm px-2 py-1">
                                {shortcut.unwrap_or_else(|| "Not assigned".to_owned())}
                            </kbd>
                        </div>
                    }).collect_view()}
                </div>
            </Section>
        </div>
    }
}

fn normalize_workspace_archive_input(value: &str) -> String {
    let trimmed = value.trim();
    match (trimmed.find('{'), trimmed.rfind('}')) {
        (Some(start), Some(end)) if start <= end => trimmed[start..=end].to_owned(),
        _ => trimmed.to_owned(),
    }
}

fn theme_preference_value(preference: ThemePreference) -> &'static str {
    match preference {
        ThemePreference::Dark => "dark",
        ThemePreference::Light => "light",
        ThemePreference::System => "system",
    }
}

fn surface_preference_value(preference: SurfacePreference) -> &'static str {
    match preference {
        SurfacePreference::Sidebar => "sidebar",
        SurfacePreference::FullTab => "full-tab",
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
