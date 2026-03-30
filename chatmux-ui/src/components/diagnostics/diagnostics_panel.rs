//! Rich diagnostics explorer.

use leptos::prelude::*;
use regex::{Regex, RegexBuilder};

use crate::bridge::{
    clipboard::{download_text, write_clipboard},
    messaging,
};
use crate::components::primitives::{
    button::{Button, ButtonSize, ButtonVariant},
    chip::Chip,
    empty_state::EmptyState,
    segmented_control::{Segment, SegmentedControl},
    text_input::TextInput,
    toggle::Toggle,
};
use crate::models::{
    DiagnosticEvent, DiagnosticLevel, DiagnosticsDetailLevel, DiagnosticsQuery,
    DiagnosticsSearchMode,
};
use crate::state::{
    app_state::AppState,
    binding_state::BindingState,
    controller::dispatch_command_result,
    diagnostics_state::DiagnosticsState,
    message_state::MessageState,
    run_state::ActiveRunState,
    workspace_state::WorkspaceListState,
};

use super::event_row::EventRow;

#[component]
pub fn DiagnosticsPanel() -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let workspace_state = expect_context::<WorkspaceListState>();
    let run_state = expect_context::<ActiveRunState>();
    let binding_state = expect_context::<BindingState>();
    let message_state = expect_context::<MessageState>();
    let diagnostics_state = expect_context::<DiagnosticsState>();

    let (scope_mode, set_scope_mode) = signal("workspace".to_owned());
    let (display_mode, set_display_mode) = signal("readable".to_owned());
    let (detail_mode, set_detail_mode) = signal("standard".to_owned());
    let (live, set_live) = signal(true);
    let (search_query, set_search_query) = signal(String::new());
    let (regex_mode, set_regex_mode) = signal(false);
    let (case_sensitive, set_case_sensitive) = signal(false);
    let (context_before, set_context_before) = signal("0".to_owned());
    let (context_after, set_context_after) = signal("0".to_owned());
    let (include_critical, set_include_critical) = signal(true);
    let (include_warning, set_include_warning) = signal(true);
    let (include_info, set_include_info) = signal(true);
    let (include_debug, set_include_debug) = signal(true);
    let (source_filter, set_source_filter) = signal("all".to_owned());
    let (provider_filter, set_provider_filter) = signal("all".to_owned());
    let (view_events, set_view_events) = signal(Vec::<DiagnosticEvent>::new());
    let (selected_event_id, set_selected_event_id) = signal(None::<crate::models::DiagnosticEventId>);
    let (refresh_key, set_refresh_key) = signal(0u32);

    Effect::new(move |_| {
        let _ = diagnostics_state.events.get();
        if live.get() {
            set_view_events.set(diagnostics_state.events.get());
        }
    });

    Effect::new(move |_| {
        let _ = refresh_key.get();
        let _ = app_state.active_workspace_id.get();
        let _ = scope_mode.get();
        let workspace_id = if scope_mode.get_untracked() == "workspace" {
            app_state.active_workspace_id.get_untracked()
        } else {
            None
        };
        let query = DiagnosticsQuery {
            workspace_id,
            include_global: scope_mode.get_untracked() == "global",
            levels: selected_levels(
                include_critical.get_untracked(),
                include_warning.get_untracked(),
                include_info.get_untracked(),
                include_debug.get_untracked(),
            ),
            sources: Vec::new(),
            providers: Vec::new(),
            text_query: None,
            search_mode: DiagnosticsSearchMode::Plain,
            case_sensitive: false,
            context_before: 0,
            context_after: 0,
            detail_level: selected_detail_level(&detail_mode.get_untracked()),
            include_artifacts: true,
            limit: Some(500),
        };
        leptos::task::spawn_local(async move {
            dispatch_command_result(
                app_state,
                workspace_state,
                run_state,
                binding_state,
                message_state,
                diagnostics_state,
                messaging::request_diagnostics_snapshot(query).await,
            );
            if !live.get_untracked() {
                set_view_events.set(diagnostics_state.events.get_untracked());
            }
        });
    });

    Effect::new(move |_| {
        let visible = filtered_events(
            &view_events.get(),
            &search_query.get(),
            regex_mode.get(),
            case_sensitive.get(),
            &context_before.get(),
            &context_after.get(),
            selected_levels(
                include_critical.get(),
                include_warning.get(),
                include_info.get(),
                include_debug.get(),
            ),
            &source_filter.get(),
            &provider_filter.get(),
        );

        let selected_id = selected_event_id.get();
        if visible.is_empty() {
            set_selected_event_id.set(None);
        } else if selected_id.is_none()
            || !visible.iter().any(|event| Some(event.id) == selected_id)
        {
            set_selected_event_id.set(Some(visible[0].id));
        }
    });

    let visible_events = Signal::derive(move || {
        filtered_events(
            &view_events.get(),
            &search_query.get(),
            regex_mode.get(),
            case_sensitive.get(),
            &context_before.get(),
            &context_after.get(),
            selected_levels(
                include_critical.get(),
                include_warning.get(),
                include_info.get(),
                include_debug.get(),
            ),
            &source_filter.get(),
            &provider_filter.get(),
        )
    });

    let selected_event = Signal::derive(move || {
        let selected_id = selected_event_id.get();
        visible_events
            .get()
            .into_iter()
            .find(|event| Some(event.id) == selected_id)
    });

    let regex_error = Signal::derive(move || {
        validate_regex(&search_query.get(), regex_mode.get(), case_sensitive.get())
    });

    let copy_payload = {
        let visible_events = visible_events;
        let selected_event = selected_event;
        move || {
            let body = render_export_payload(
                &display_mode.get_untracked(),
                &detail_mode.get_untracked(),
                selected_event.get(),
                &visible_events.get_untracked(),
            );
            leptos::task::spawn_local(async move {
                let _ = write_clipboard(&body).await;
            });
        }
    };

    let download_payload = {
        let visible_events = visible_events;
        let selected_event = selected_event;
        move || {
            let display = display_mode.get_untracked();
            let detail = detail_mode.get_untracked();
            let body = render_export_payload(
                &display,
                &detail,
                selected_event.get(),
                &visible_events.get_untracked(),
            );
            let (filename, mime_type) = if display == "event_data" {
                ("chatmux-diagnostics.ndjson".to_owned(), "application/x-ndjson".to_owned())
            } else {
                ("chatmux-diagnostics.txt".to_owned(), "text/plain".to_owned())
            };
            leptos::task::spawn_local(async move {
                let _ = download_text(&filename, &mime_type, &body).await;
            });
        }
    };

    view! {
        <div class="diagnostics-panel flex flex-col h-full" style="min-height: 0;">
            <div
                class="flex flex-col gap-4"
                style="padding: var(--space-5); border-bottom: 1px solid var(--border-subtle); \
                       background: linear-gradient(180deg, var(--surface-raised), var(--surface-sunken));"
            >
                <div class="flex items-center justify-between gap-4 flex-wrap">
                    <div class="flex flex-col gap-1">
                        <h2 class="type-title text-primary">"Diagnostics Explorer"</h2>
                        <p class="type-body text-secondary">
                            "Rich live diagnostics with human-readable sections, structured event data, regex search, copy, and download."
                        </p>
                    </div>

                    <div class="flex items-center gap-2 flex-wrap">
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Small
                            on_click=Box::new(move |_| set_refresh_key.update(|value| *value += 1))
                        >
                            "Refresh"
                        </Button>
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Small
                            on_click=Box::new(move |_| copy_payload())
                        >
                            "Copy"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Small
                            on_click=Box::new(move |_| download_payload())
                        >
                            "Download All"
                        </Button>
                    </div>
                </div>

                <div class="flex items-center gap-4 flex-wrap">
                    <SegmentedControl
                        aria_label="Diagnostics scope".to_string()
                        segments=vec![
                            Segment { value: "workspace".into(), label: "Workspace".into() },
                            Segment { value: "global".into(), label: "All Workspaces".into() },
                        ]
                        selected=scope_mode
                        on_change=move |value| set_scope_mode.set(value)
                    />
                    <SegmentedControl
                        aria_label="Diagnostics display mode".to_string()
                        segments=vec![
                            Segment { value: "readable".into(), label: "Readable".into() },
                            Segment { value: "event_data".into(), label: "Event Data".into() },
                        ]
                        selected=display_mode
                        on_change=move |value| set_display_mode.set(value)
                    />
                    <SegmentedControl
                        aria_label="Diagnostics detail mode".to_string()
                        segments=vec![
                            Segment { value: "overview".into(), label: "Overview".into() },
                            Segment { value: "standard".into(), label: "Standard".into() },
                            Segment { value: "verbose".into(), label: "Verbose".into() },
                        ]
                        selected=detail_mode
                        on_change=move |value| set_detail_mode.set(value)
                    />
                    <div class="flex items-center gap-2">
                        <span class="type-caption text-secondary">"Live"</span>
                        <Toggle checked=live on_change=move |value| {
                            set_live.set(value);
                            if !value {
                                set_view_events.set(diagnostics_state.events.get_untracked());
                            } else {
                                set_view_events.set(diagnostics_state.events.get_untracked());
                            }
                        } />
                    </div>
                </div>

                <div class="grid gap-3" style="grid-template-columns: minmax(0, 2fr) repeat(4, minmax(0, 1fr));">
                    <div style="grid-column: 1 / span 2;">
                        <TextInput
                            value=search_query
                            on_input=move |value| set_search_query.set(value)
                            placeholder="Search diagnostics..."
                            aria_label="Search diagnostics".to_string()
                        />
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="type-caption text-secondary">"Regex"</span>
                        <Toggle checked=regex_mode on_change=move |value| set_regex_mode.set(value) />
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="type-caption text-secondary">"Case"</span>
                        <Toggle checked=case_sensitive on_change=move |value| set_case_sensitive.set(value) />
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="type-caption text-secondary">"Before"</span>
                        <input
                            class="type-caption"
                            type="number"
                            min="0"
                            prop:value=move || context_before.get()
                            style="width: 72px; padding: var(--space-2); background: var(--surface-sunken); \
                                   border: 1px solid var(--border-default); border-radius: var(--radius-md); color: var(--text-primary);"
                            on:input=move |ev| set_context_before.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="type-caption text-secondary">"After"</span>
                        <input
                            class="type-caption"
                            type="number"
                            min="0"
                            prop:value=move || context_after.get()
                            style="width: 72px; padding: var(--space-2); background: var(--surface-sunken); \
                                   border: 1px solid var(--border-default); border-radius: var(--radius-md); color: var(--text-primary);"
                            on:input=move |ev| set_context_after.set(event_target_value(&ev))
                        />
                    </div>
                </div>

                {move || regex_error.get().map(|message| view! {
                    <div
                        class="type-caption"
                        style="padding: var(--space-3); border-radius: var(--radius-md); \
                               background: var(--status-error-muted); color: var(--status-error-text);"
                    >
                        {message}
                    </div>
                })}

                <div class="flex items-center gap-2 flex-wrap">
                    <Chip label="Critical".into() selected=include_critical
                        on_click=move || set_include_critical.update(|v| *v = !*v)
                        selected_bg="var(--status-error-muted)".to_string()
                        selected_border="var(--status-error-border)".to_string() />
                    <Chip label="Warning".into() selected=include_warning
                        on_click=move || set_include_warning.update(|v| *v = !*v)
                        selected_bg="var(--status-warning-muted)".to_string()
                        selected_border="var(--status-warning-border)".to_string() />
                    <Chip label="Info".into() selected=include_info
                        on_click=move || set_include_info.update(|v| *v = !*v)
                        selected_bg="var(--status-info-muted)".to_string()
                        selected_border="var(--status-info-border)".to_string() />
                    <Chip label="Debug".into() selected=include_debug
                        on_click=move || set_include_debug.update(|v| *v = !*v) />
                </div>

                <div class="flex items-center gap-3 flex-wrap">
                    <SummaryPill label="Total" value=move || diagnostics_state.summary.get().total />
                    <SummaryPill label="Critical" value=move || diagnostics_state.summary.get().critical />
                    <SummaryPill label="Warning" value=move || diagnostics_state.summary.get().warning />
                    <SummaryPill label="Info" value=move || diagnostics_state.summary.get().info />
                    <SummaryPill label="Debug" value=move || diagnostics_state.summary.get().debug />
                    <span class="type-caption text-tertiary">
                        {move || format!("Visible: {}", visible_events.get().len())}
                    </span>
                </div>

                <div class="flex items-center gap-2 flex-wrap">
                    <span class="type-caption text-secondary">"Source"</span>
                    {source_options(&view_events.get()).into_iter().map(|source| {
                        let source_check = source.clone();
                        let source_click = source.clone();
                        view! {
                            <button
                                class="type-caption"
                                style=move || format!(
                                    "padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); \
                                     border: 1px solid {}; background: {}; color: {}; cursor: pointer;",
                                    if source_filter.get() == source_check { "var(--accent-primary)" } else { "var(--border-subtle)" },
                                    if source_filter.get() == source_check { "var(--surface-raised)" } else { "var(--surface-sunken)" },
                                    if source_filter.get() == source_check { "var(--text-primary)" } else { "var(--text-secondary)" },
                                )
                                on:click=move |_| set_source_filter.set(source_click.clone())
                            >
                                {source.clone()}
                            </button>
                        }
                    }).collect_view()}
                </div>

                <div class="flex items-center gap-2 flex-wrap">
                    <span class="type-caption text-secondary">"Provider"</span>
                    {provider_options(&view_events.get()).into_iter().map(|provider| {
                        let provider_check = provider.clone();
                        let provider_click = provider.clone();
                        view! {
                            <button
                                class="type-caption"
                                style=move || format!(
                                    "padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); \
                                     border: 1px solid {}; background: {}; color: {}; cursor: pointer;",
                                    if provider_filter.get() == provider_check { "var(--accent-primary)" } else { "var(--border-subtle)" },
                                    if provider_filter.get() == provider_check { "var(--surface-raised)" } else { "var(--surface-sunken)" },
                                    if provider_filter.get() == provider_check { "var(--text-primary)" } else { "var(--text-secondary)" },
                                )
                                on:click=move |_| set_provider_filter.set(provider_click.clone())
                            >
                                {provider.clone()}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="flex-1 grid" style="grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr); min-height: 0;">
                <div class="flex flex-col gap-3 overflow-y-auto" style="padding: var(--space-5);">
                    {move || {
                        let events = visible_events.get();
                        if view_events.get().is_empty() {
                            view! {
                                <EmptyState
                                    heading="All clear"
                                    description="No diagnostic events have been recorded."
                                    icon=crate::components::primitives::icon::IconKind::ShieldCheck
                                />
                            }.into_any()
                        } else if events.is_empty() {
                            view! {
                                <EmptyState
                                    heading="No matches"
                                    description="No diagnostics match the current filters or search."
                                    icon=crate::components::primitives::icon::IconKind::Search
                                />
                            }.into_any()
                        } else {
                            view! {
                                <>
                                    {events.into_iter().map(|event| {
                                        let event_id = event.id;
                                        view! {
                                            <EventRow
                                                event=event
                                                selected=Signal::derive(move || selected_event_id.get() == Some(event_id))
                                                query=search_query.get()
                                                regex_mode=regex_mode.get()
                                                case_sensitive=case_sensitive.get()
                                                on_select=Box::new(move || set_selected_event_id.set(Some(event_id)))
                                            />
                                        }
                                    }).collect_view()}
                                </>
                            }.into_any()
                        }
                    }}
                </div>

                <div
                    class="flex flex-col overflow-y-auto"
                    style="border-left: 1px solid var(--border-subtle); padding: var(--space-5); background: var(--surface-raised);"
                >
                    {move || {
                        if let Some(event) = selected_event.get() {
                            view! {
                                <DiagnosticDetail
                                    event=event
                                    display_mode=display_mode.get()
                                    detail_mode=detail_mode.get()
                                    query=search_query.get()
                                    regex_mode=regex_mode.get()
                                    case_sensitive=case_sensitive.get()
                                />
                            }.into_any()
                        } else {
                            view! {
                                <EmptyState
                                    heading="Select an event"
                                    description="Choose a diagnostic event to inspect its rich detail."
                                    icon=crate::components::primitives::icon::IconKind::Eye
                                />
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn SummaryPill(label: &'static str, value: impl Fn() -> u32 + 'static + Copy + Send) -> impl IntoView {
    view! {
        <div
            class="flex items-center gap-2"
            style="padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); \
                   background: var(--surface-sunken); border: 1px solid var(--border-subtle);"
        >
            <span class="type-caption text-secondary">{label}</span>
            <span class="type-caption-strong text-primary">{move || value().to_string()}</span>
        </div>
    }
}

#[component]
fn DiagnosticDetail(
    event: DiagnosticEvent,
    display_mode: String,
    detail_mode: String,
    query: String,
    regex_mode: bool,
    case_sensitive: bool,
) -> impl IntoView {
    let detail_level = selected_detail_level(&detail_mode);
    let raw_json = serde_json::to_string_pretty(&event).unwrap_or_else(|_| "{}".to_owned());
    let attributes = event.attributes.clone();
    let detail_text = event.detail.clone();
    let detail_rows = detail_rows(&event, detail_level);
    let attributes_json = serde_json::to_string_pretty(&attributes).unwrap_or_else(|_| "{}".to_owned());
    let query_for_raw = query.clone();
    let query_for_detail = query.clone();

    view! {
        <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
                <div class="flex items-center gap-2 flex-wrap">
                    <span class="type-caption text-secondary">{event.timestamp.to_rfc3339()}</span>
                    <span class="type-caption text-secondary">{format!("{:?}", event.source)}</span>
                    <span class="type-code-small text-tertiary" style="font-family: var(--font-mono);">
                        {event.code.clone()}
                    </span>
                </div>
                <h3 class="type-title text-primary">{event.title.clone()}</h3>
                <p class="type-body text-secondary">{event.summary.clone()}</p>
            </div>

            {if display_mode == "event_data" {
                view! {
                    <DetailSection title="Structured Event">
                        <pre
                            class="type-code-small"
                            style="margin: 0; white-space: pre-wrap; word-break: break-word; color: var(--text-primary);"
                        >
                            {highlight_text(raw_json, query_for_raw, regex_mode, case_sensitive)}
                        </pre>
                    </DetailSection>
                }.into_any()
            } else {
                view! {
                    <>
                        <DetailSection title="Readable Detail">
                            <div class="type-body text-primary" style="white-space: pre-wrap; word-break: break-word;">
                                {highlight_text(detail_text, query_for_detail, regex_mode, case_sensitive)}
                            </div>
                        </DetailSection>
                        <DetailSection title="Structured Fields">
                            <table style="width: 100%; border-collapse: collapse;">
                                {detail_rows.into_iter().map(|(label, value)| view! {
                                    <tr style="border-bottom: 1px solid var(--border-subtle);">
                                        <td class="type-caption text-secondary" style="padding: var(--space-2) var(--space-3) var(--space-2) 0; vertical-align: top;">
                                            {label}
                                        </td>
                                        <td class="type-body text-primary" style="padding: var(--space-2) 0; word-break: break-word;">
                                            {value}
                                        </td>
                                    </tr>
                                }).collect_view()}
                            </table>
                        </DetailSection>
                        {(!attributes.is_empty() && detail_level != DiagnosticsDetailLevel::Overview).then(|| view! {
                            <DetailSection title="Attributes">
                                <pre class="type-code-small" style="margin: 0; white-space: pre-wrap; word-break: break-word; color: var(--text-primary);">
                                    {attributes_json}
                                </pre>
                            </DetailSection>
                        })}
                    </>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn DetailSection(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <section
            class="flex flex-col gap-3"
            style="padding: var(--space-4); border-radius: var(--radius-md); \
                   background: var(--surface-sunken); border: 1px solid var(--border-subtle);"
        >
            <h4 class="type-label text-primary">{title}</h4>
            {children()}
        </section>
    }
}

fn source_options(events: &[DiagnosticEvent]) -> Vec<String> {
    let mut values = vec!["all".to_owned()];
    for event in events {
        let source = format!("{:?}", event.source);
        if !values.contains(&source) {
            values.push(source);
        }
    }
    values
}

fn provider_options(events: &[DiagnosticEvent]) -> Vec<String> {
    let mut values = vec!["all".to_owned()];
    for provider in events.iter().filter_map(|event| event.provider_id) {
        let label = provider.display_name().to_owned();
        if !values.contains(&label) {
            values.push(label);
        }
    }
    values
}

fn selected_levels(
    include_critical: bool,
    include_warning: bool,
    include_info: bool,
    include_debug: bool,
) -> Vec<DiagnosticLevel> {
    let mut levels = Vec::new();
    if include_critical {
        levels.push(DiagnosticLevel::Critical);
    }
    if include_warning {
        levels.push(DiagnosticLevel::Warning);
    }
    if include_info {
        levels.push(DiagnosticLevel::Info);
    }
    if include_debug {
        levels.push(DiagnosticLevel::Debug);
    }
    levels
}

fn selected_detail_level(mode: &str) -> DiagnosticsDetailLevel {
    match mode {
        "overview" => DiagnosticsDetailLevel::Overview,
        "verbose" => DiagnosticsDetailLevel::Verbose,
        _ => DiagnosticsDetailLevel::Standard,
    }
}

fn filtered_events(
    events: &[DiagnosticEvent],
    query: &str,
    regex_mode: bool,
    case_sensitive: bool,
    context_before: &str,
    context_after: &str,
    levels: Vec<DiagnosticLevel>,
    source_filter: &str,
    provider_filter: &str,
) -> Vec<DiagnosticEvent> {
    let context_before = context_before.parse::<usize>().unwrap_or(0);
    let context_after = context_after.parse::<usize>().unwrap_or(0);

    let base_events = events
        .iter()
        .filter(|event| levels.contains(&event.level))
        .filter(|event| source_filter == "all" || format!("{:?}", event.source) == source_filter)
        .filter(|event| {
            provider_filter == "all"
                || event
                    .provider_id
                    .map(|provider| provider.display_name() == provider_filter)
                    .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();

    if query.trim().is_empty() {
        return base_events;
    }

    let Some(regex) = compile_search(query, regex_mode, case_sensitive) else {
        return base_events;
    };

    let mut keep = std::collections::BTreeSet::new();
    for (index, event) in base_events.iter().enumerate() {
        if matches_query(event, &regex) {
            let start = index.saturating_sub(context_before);
            let end = (index + context_after + 1).min(base_events.len());
            for idx in start..end {
                keep.insert(idx);
            }
        }
    }

    keep.into_iter()
        .filter_map(|index| base_events.get(index).cloned())
        .collect()
}

fn matches_query(event: &DiagnosticEvent, regex: &Regex) -> bool {
    let haystack = format!(
        "{}\n{}\n{}\n{}\n{}",
        event.title,
        event.summary,
        event.detail,
        event.code,
        serde_json::to_string(&event.attributes).unwrap_or_default()
    );
    regex.is_match(&haystack)
}

fn validate_regex(query: &str, regex_mode: bool, case_sensitive: bool) -> Option<String> {
    if query.trim().is_empty() || !regex_mode {
        return None;
    }
    compile_search(query, regex_mode, case_sensitive)
        .map(|_| None)
        .unwrap_or_else(|| Some("Invalid regular expression. The last valid result set remains visible.".to_owned()))
}

fn compile_search(query: &str, regex_mode: bool, case_sensitive: bool) -> Option<Regex> {
    let pattern = if regex_mode {
        query.to_owned()
    } else {
        regex::escape(query)
    };
    RegexBuilder::new(&pattern)
        .case_insensitive(!case_sensitive)
        .build()
        .ok()
}

fn highlight_text(
    text: String,
    query: String,
    regex_mode: bool,
    case_sensitive: bool,
) -> AnyView {
    if query.trim().is_empty() {
        return view! { <>{text}</> }.into_any();
    }
    let Some(regex) = compile_search(&query, regex_mode, case_sensitive) else {
        return view! { <>{text}</> }.into_any();
    };

    let mut cursor = 0usize;
    let mut segments = Vec::new();
    for matched in regex.find_iter(&text) {
        if matched.start() > cursor {
            segments.push((false, text[cursor..matched.start()].to_owned()));
        }
        segments.push((true, matched.as_str().to_owned()));
        cursor = matched.end();
    }
    if cursor < text.len() {
        segments.push((false, text[cursor..].to_owned()));
    }
    if segments.is_empty() {
        return view! { <>{text}</> }.into_any();
    }

    view! {
        <>
            {segments.into_iter().map(|(highlight, segment)| {
                if highlight {
                    view! {
                        <mark
                            style="background: var(--status-warning-muted); color: var(--text-primary); \
                                   padding: 0 2px; border-radius: var(--radius-sm);"
                        >
                            {segment}
                        </mark>
                    }.into_any()
                } else {
                    view! { <>{segment}</> }.into_any()
                }
            }).collect_view()}
        </>
    }
        .into_any()
}

fn detail_rows(event: &DiagnosticEvent, detail_level: DiagnosticsDetailLevel) -> Vec<(&'static str, String)> {
    let mut rows = vec![
        ("Scope", format!("{:?}", event.scope)),
        ("Source", format!("{:?}", event.source)),
        ("Severity", format!("{:?}", event.level)),
        ("Code", event.code.clone()),
    ];

    if detail_level != DiagnosticsDetailLevel::Overview {
        rows.push(("Workspace", event.workspace_id.0.to_string()));
        rows.push((
            "Provider",
            event.provider_id
                .map(|provider| provider.display_name().to_owned())
                .unwrap_or_else(|| "—".to_owned()),
        ));
        rows.push((
            "Binding",
            event.binding_id
                .map(|binding| binding.0.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ));
        rows.push((
            "Run",
            event.run_id
                .map(|run| run.0.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ));
    }

    if detail_level == DiagnosticsDetailLevel::Verbose {
        rows.push((
            "Round",
            event.round_id
                .map(|round| round.0.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ));
        rows.push((
            "Message",
            event.message_id
                .map(|message| message.0.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ));
        rows.push((
            "Dispatch",
            event.dispatch_id
                .map(|dispatch| dispatch.0.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ));
        rows.push((
            "Snapshot Ref",
            event.snapshot_ref.clone().unwrap_or_else(|| "—".to_owned()),
        ));
    }

    rows
}

fn render_export_payload(
    display_mode: &str,
    detail_mode: &str,
    selected_event: Option<DiagnosticEvent>,
    events: &[DiagnosticEvent],
) -> String {
    if display_mode == "event_data" {
        let payload = if let Some(event) = selected_event {
            vec![event]
        } else {
            events.to_vec()
        };
        payload
            .into_iter()
            .map(|event| serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_owned()))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        let detail_level = selected_detail_level(detail_mode);
        let target_events = if let Some(event) = selected_event {
            vec![event]
        } else {
            events.to_vec()
        };
        let mut out = String::new();
        out.push_str("Chatmux Diagnostics Report\n\n");
        for event in target_events {
            out.push_str(&format!(
                "[{:?}] {} | {:?} | {}\n{}\n{}\n\n",
                event.level,
                event.timestamp.to_rfc3339(),
                event.source,
                event.code,
                event.title,
                event.detail
            ));
            if detail_level != DiagnosticsDetailLevel::Overview {
                out.push_str(&format!(
                    "summary: {}\nprovider: {}\nworkspace: {}\n\n",
                    event.summary,
                    event.provider_id
                        .map(|provider| provider.display_name().to_owned())
                        .unwrap_or_else(|| "—".to_owned()),
                    event.workspace_id.0
                ));
            }
        }
        out
    }
}
