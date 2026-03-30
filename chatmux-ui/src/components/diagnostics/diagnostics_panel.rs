//! Rich diagnostics explorer.

use std::collections::BTreeSet;

use leptos::prelude::*;
use regex::{Regex, RegexBuilder};

use crate::bridge::{
    clipboard::{download_text, write_clipboard},
    messaging,
};
use crate::components::primitives::{
    badge::{Badge, BadgeVariant},
    button::{Button, ButtonSize, ButtonVariant},
    chip::Chip,
    divider::Divider,
    empty_state::EmptyState,
    number_input::NumberInput,
    segmented_control::{Segment, SegmentedControl},
    surface::Surface,
    text_input::TextInput,
    toggle::Toggle,
    tooltip::Tooltip,
};
use crate::models::{
    DiagnosticEvent, DiagnosticLevel, DiagnosticsDetailLevel, DiagnosticsQuery,
    DiagnosticsSearchMode,
};
use crate::state::{
    app_state::AppState, binding_state::BindingState, controller::dispatch_command_result,
    diagnostics_state::DiagnosticsState, message_state::MessageState, run_state::ActiveRunState,
    workspace_state::WorkspaceListState,
};
use crate::time::format_local_datetime;

use super::event_row::EventRow;

// ── Sort types ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortField {
    Timestamp,
    Severity,
    Source,
    Provider,
    Code,
    Title,
}

impl SortField {
    fn label(self) -> &'static str {
        match self {
            Self::Timestamp => "Time",
            Self::Severity => "Severity",
            Self::Source => "Source",
            Self::Provider => "Provider",
            Self::Code => "Code",
            Self::Title => "Title",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, PartialEq, Eq)]
struct SelectionSyncState {
    selected_event_id: Option<crate::models::DiagnosticEventId>,
    selected_ids: BTreeSet<crate::models::DiagnosticEventId>,
}

const ALL_SORT_FIELDS: &[SortField] = &[
    SortField::Timestamp,
    SortField::Severity,
    SortField::Source,
    SortField::Provider,
    SortField::Code,
    SortField::Title,
];

// ── Component ───────────────────────────────────────────────────────────────

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
    let (context_before, set_context_before) = signal(0.0f64);
    let (context_after, set_context_after) = signal(0.0f64);
    let include_critical = diagnostics_state.filter_critical;
    let set_include_critical = diagnostics_state.set_filter_critical;
    let include_warning = diagnostics_state.filter_warning;
    let set_include_warning = diagnostics_state.set_filter_warning;
    let include_info = diagnostics_state.filter_info;
    let set_include_info = diagnostics_state.set_filter_info;
    let include_debug = diagnostics_state.filter_debug;
    let set_include_debug = diagnostics_state.set_filter_debug;
    let (source_filter, set_source_filter) = signal("all".to_owned());
    let (provider_filter, set_provider_filter) = signal("all".to_owned());
    let (view_events, set_view_events) = signal(Vec::<DiagnosticEvent>::new());
    let (selected_event_id, set_selected_event_id) =
        signal(None::<crate::models::DiagnosticEventId>);
    let (refresh_key, set_refresh_key) = signal(0u32);
    let (delete_stored, set_delete_stored) = signal(false);

    // Multi-select state
    let (selected_ids, set_selected_ids) =
        signal(BTreeSet::<crate::models::DiagnosticEventId>::new());
    let (anchor_id, set_anchor_id) = signal(None::<crate::models::DiagnosticEventId>);

    // Sort state — default: newest first
    let (sort_criteria, set_sort_criteria) = signal(vec![(SortField::Timestamp, SortDir::Desc)]);

    // ── Effects ─────────────────────────────────────────────────────────────

    let current_scope_query = move |limit: Option<u32>| DiagnosticsQuery {
        workspace_id: if scope_mode.get_untracked() == "workspace" {
            app_state.active_workspace_id.get_untracked()
        } else {
            None
        },
        include_global: scope_mode.get_untracked() == "global",
        levels: selected_levels(true, true, true, true),
        sources: Vec::new(),
        providers: Vec::new(),
        text_query: None,
        search_mode: DiagnosticsSearchMode::Plain,
        case_sensitive: false,
        context_before: 0,
        context_after: 0,
        detail_level: selected_detail_level(&detail_mode.get_untracked()),
        include_artifacts: true,
        limit,
    };

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
        let mut query = current_scope_query(Some(500));
        query.workspace_id = workspace_id;
        query.levels = selected_levels(
            include_critical.get_untracked(),
            include_warning.get_untracked(),
            include_info.get_untracked(),
            include_debug.get_untracked(),
        );
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

    // Auto-select first visible event; prune stale multi-selection
    Effect::new(move |_| {
        let visible = filtered_events(
            &view_events.get(),
            &search_query.get(),
            regex_mode.get(),
            case_sensitive.get(),
            context_before.get() as usize,
            context_after.get() as usize,
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
        let current_selected_ids = selected_ids.get();
        let next_selection = sync_selection_state(&visible, selected_id, &current_selected_ids);

        if next_selection.selected_event_id != selected_id {
            set_selected_event_id.set(next_selection.selected_event_id);
        }
        if next_selection.selected_ids != current_selected_ids {
            set_selected_ids.set(next_selection.selected_ids);
        }
    });

    // ── Derived signals ─────────────────────────────────────────────────────

    let visible_events = Signal::derive(move || {
        filtered_events(
            &view_events.get(),
            &search_query.get(),
            regex_mode.get(),
            case_sensitive.get(),
            context_before.get() as usize,
            context_after.get() as usize,
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

    let sorted_events = Signal::derive(move || {
        let mut events = visible_events.get();
        let criteria = sort_criteria.get();
        if !criteria.is_empty() {
            sort_events(&mut events, &criteria);
        }
        events
    });

    let selected_event = Signal::derive(move || {
        let selected_id = selected_event_id.get();
        sorted_events
            .get()
            .into_iter()
            .find(|event| Some(event.id) == selected_id)
    });

    let regex_error = Signal::derive(move || {
        validate_regex(&search_query.get(), regex_mode.get(), case_sensitive.get())
    });

    // ── Action closures ─────────────────────────────────────────────────────

    let copy_payload = move || {
        let multi = selected_ids.get_untracked();
        let events: Vec<DiagnosticEvent> = if multi.len() > 1 {
            sorted_events
                .get_untracked()
                .into_iter()
                .filter(|e| multi.contains(&e.id))
                .collect()
        } else if let Some(event) = selected_event.get_untracked() {
            vec![event]
        } else {
            sorted_events.get_untracked()
        };
        let body = render_export_payload(
            &display_mode.get_untracked(),
            &detail_mode.get_untracked(),
            &events,
        );
        leptos::task::spawn_local(async move {
            let _ = write_clipboard(&body).await;
        });
    };

    let copy_all_payload = move || {
        let events = sorted_events.get_untracked();
        let body = events
            .iter()
            .map(|e| serde_json::to_string(e).unwrap_or_else(|_| "{}".to_owned()))
            .collect::<Vec<_>>()
            .join("\n");
        leptos::task::spawn_local(async move {
            let _ = write_clipboard(&body).await;
        });
    };

    let download_payload = move || {
        let multi = selected_ids.get_untracked();
        let events: Vec<DiagnosticEvent> = if multi.len() > 1 {
            sorted_events
                .get_untracked()
                .into_iter()
                .filter(|e| multi.contains(&e.id))
                .collect()
        } else if let Some(event) = selected_event.get_untracked() {
            vec![event]
        } else {
            sorted_events.get_untracked()
        };
        let display = display_mode.get_untracked();
        let detail = detail_mode.get_untracked();
        let body = render_export_payload(&display, &detail, &events);
        let (filename, mime_type) = if display == "event_data" {
            (
                "chatmux-diagnostics.ndjson".to_owned(),
                "application/x-ndjson".to_owned(),
            )
        } else {
            (
                "chatmux-diagnostics.txt".to_owned(),
                "text/plain".to_owned(),
            )
        };
        leptos::task::spawn_local(async move {
            let _ = download_text(&filename, &mime_type, &body).await;
        });
    };

    let clear_diagnostics = move || {
        if delete_stored.get_untracked() {
            let query = current_scope_query(None);
            leptos::task::spawn_local(async move {
                dispatch_command_result(
                    app_state,
                    workspace_state,
                    run_state,
                    binding_state,
                    message_state,
                    diagnostics_state,
                    messaging::clear_diagnostics(query).await,
                );
                set_view_events.set(diagnostics_state.events.get_untracked());
                set_selected_ids.set(BTreeSet::new());
                set_selected_event_id.set(None);
                set_anchor_id.set(None);
            });
        } else {
            set_live.set(false);
            set_view_events.set(Vec::new());
            set_selected_ids.set(BTreeSet::new());
            set_selected_event_id.set(None);
            set_anchor_id.set(None);
        }
    };

    // ── View ────────────────────────────────────────────────────────────────

    view! {
        <div class="diagnostics-panel flex flex-col h-full" style="min-height: 0;">
            // ── Row 1: Header ───────────────────────────────────────────
            <header
                class="diagnostics-header flex items-center justify-between gap-4 border-b"
                style="padding: var(--space-3) var(--space-6); \
                       background: var(--surface-raised);"
            >
                <div class="flex items-center gap-4">
                    <h2 class="type-title text-primary">"Diagnostics"</h2>
                    // Inline summary badges
                    <div class="flex items-center gap-2">
                        <Badge variant=BadgeVariant::Error>
                            {move || format!("{}", diagnostics_state.summary.get().critical)}
                        </Badge>
                        <Badge variant=BadgeVariant::Warning>
                            {move || format!("{}", diagnostics_state.summary.get().warning)}
                        </Badge>
                        <Badge variant=BadgeVariant::Info>
                            {move || format!("{}", diagnostics_state.summary.get().info)}
                        </Badge>
                        <Badge>
                            {move || format!("{}", diagnostics_state.summary.get().debug)}
                        </Badge>
                        <span class="type-caption text-tertiary">
                            {move || {
                                let visible = sorted_events.get().len();
                                let total = diagnostics_state.summary.get().total;
                                format!("{}/{} visible", visible, total)
                            }}
                        </span>
                        {move || {
                            let sel_count = selected_ids.get().len();
                            (sel_count > 1).then(|| view! {
                                <span class="type-caption text-link">
                                    {format!("· {} selected", sel_count)}
                                </span>
                            })
                        }}
                    </div>
                </div>
                <div class="flex items-center gap-3">
                    // ── Live + Refresh ─────────────────────
                    <Surface class="flex items-center gap-3 py-2 px-4".to_string()>
                        <Tooltip text="Automatically show new events as they arrive">
                            <span class="type-label text-secondary">"Live"</span>
                        </Tooltip>
                        <Toggle checked=live on_change=move |value| {
                            set_live.set(value);
                            set_view_events.set(diagnostics_state.events.get_untracked());
                        } />
                        <Divider vertical=true />
                        <Tooltip text="Refetch diagnostics from the coordinator">
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Small
                                on_click=Box::new(move |_| set_refresh_key.update(|v| *v += 1))
                                aria_label="Refresh diagnostics".to_string()
                            >
                                "Refresh"
                            </Button>
                        </Tooltip>
                    </Surface>
                    // ── Copy / Export ──────────────────────
                    <Surface class="flex items-center gap-2 py-2 px-3".to_string()>
                        <Tooltip text="Copy selected events (or focused event) to clipboard">
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Small
                                on_click=Box::new(move |_| copy_payload())
                            >
                                "Copy"
                            </Button>
                        </Tooltip>
                        <Tooltip text="Copy all visible events as raw NDJSON">
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Small
                                on_click=Box::new(move |_| copy_all_payload())
                            >
                                "Copy All"
                            </Button>
                        </Tooltip>
                        <Tooltip text="Download selected events as a file">
                            <Button
                                variant=ButtonVariant::Primary
                                size=ButtonSize::Small
                                on_click=Box::new(move |_| download_payload())
                            >
                                "Export"
                            </Button>
                        </Tooltip>
                    </Surface>
                    // ── Clear / Delete ─────────────────────
                    <Surface class="flex items-center gap-3 py-2 px-4".to_string()>
                        <Tooltip text="Off: hides current view only. On: permanently deletes stored diagnostics in scope.">
                            <div class="flex items-center gap-2">
                                <span class="type-label text-secondary">"Delete Stored"</span>
                                <Toggle
                                    checked=delete_stored
                                    on_change=move |value| set_delete_stored.set(value)
                                    aria_label="Toggle delete stored diagnostics mode".to_string()
                                />
                            </div>
                        </Tooltip>
                        <Tooltip text="Clear or permanently delete diagnostics based on toggle">
                            <button
                                class="type-label select-none"
                                on:click=move |_| clear_diagnostics()
                                style=move || {
                                    let destructive = delete_stored.get();
                                    format!(
                                        "padding: var(--space-2) var(--space-4); \
                                         min-height: 28px; \
                                         border-radius: var(--radius-md); \
                                         border: 1px solid {}; \
                                         background: {}; \
                                         color: {}; \
                                         cursor: pointer; \
                                         font-weight: var(--type-label-weight); \
                                         letter-spacing: var(--type-label-tracking); \
                                         transition: all var(--duration-fast) var(--easing-standard);",
                                        if destructive { "var(--status-error-border)" } else { "var(--border-default)" },
                                        if destructive { "var(--status-error-solid)" } else { "var(--surface-sunken)" },
                                        if destructive { "var(--text-inverse)" } else { "var(--text-primary)" },
                                    )
                                }
                            >
                                {move || if delete_stored.get() { "Delete Stored" } else { "Hide View" }}
                            </button>
                        </Tooltip>
                    </Surface>
                </div>
            </header>

            // ── Row 2: Filters bar ─────────────────────────────────────
            <div
                class="flex items-center gap-4 flex-wrap border-b"
                style="padding: var(--space-3) var(--space-6); background: var(--surface-default);"
            >
                <SegmentedControl
                    aria_label="Diagnostics scope".to_string()
                    segments=vec![
                        Segment { value: "workspace".into(), label: "Workspace".into() },
                        Segment { value: "global".into(), label: "All Workspaces".into() },
                    ]
                    selected=scope_mode
                    on_change=move |value| set_scope_mode.set(value)
                    tooltips=vec![
                        "Events from the active workspace".to_string(),
                        "Events from all workspaces".to_string(),
                    ]
                />
                <SegmentedControl
                    aria_label="Display mode".to_string()
                    segments=vec![
                        Segment { value: "readable".into(), label: "Readable".into() },
                        Segment { value: "event_data".into(), label: "Event Data".into() },
                    ]
                    selected=display_mode
                    on_change=move |value| set_display_mode.set(value)
                    tooltips=vec![
                        "Human-readable formatted view with sections".to_string(),
                        "Raw structured JSON event data".to_string(),
                    ]
                />
                <SegmentedControl
                    aria_label="Detail level".to_string()
                    segments=vec![
                        Segment { value: "overview".into(), label: "Overview".into() },
                        Segment { value: "standard".into(), label: "Standard".into() },
                        Segment { value: "verbose".into(), label: "Verbose".into() },
                    ]
                    selected=detail_mode
                    on_change=move |value| set_detail_mode.set(value)
                    tooltips=vec![
                        "Essential fields: severity, source, code".to_string(),
                        "Standard fields: workspace, provider, binding, run".to_string(),
                        "All fields: round, message, dispatch, snapshot ref".to_string(),
                    ]
                />
                <Divider vertical=true />
                // Level filter chips
                <Chip label="Critical".into() selected=include_critical
                    title="Click to toggle, Ctrl+Click to show only this level".to_string()
                    on_click=move |ev: web_sys::MouseEvent| {
                        if ev.ctrl_key() || ev.meta_key() {
                            set_include_critical.set(true);
                            set_include_warning.set(false);
                            set_include_info.set(false);
                            set_include_debug.set(false);
                        } else {
                            set_include_critical.update(|v| *v = !*v);
                        }
                    }
                    selected_bg="var(--status-error-muted)".to_string()
                    selected_border="var(--status-error-border)".to_string() />
                <Chip label="Warning".into() selected=include_warning
                    title="Click to toggle, Ctrl+Click to show only this level".to_string()
                    on_click=move |ev: web_sys::MouseEvent| {
                        if ev.ctrl_key() || ev.meta_key() {
                            set_include_critical.set(false);
                            set_include_warning.set(true);
                            set_include_info.set(false);
                            set_include_debug.set(false);
                        } else {
                            set_include_warning.update(|v| *v = !*v);
                        }
                    }
                    selected_bg="var(--status-warning-muted)".to_string()
                    selected_border="var(--status-warning-border)".to_string() />
                <Chip label="Info".into() selected=include_info
                    title="Click to toggle, Ctrl+Click to show only this level".to_string()
                    on_click=move |ev: web_sys::MouseEvent| {
                        if ev.ctrl_key() || ev.meta_key() {
                            set_include_critical.set(false);
                            set_include_warning.set(false);
                            set_include_info.set(true);
                            set_include_debug.set(false);
                        } else {
                            set_include_info.update(|v| *v = !*v);
                        }
                    }
                    selected_bg="var(--status-info-muted)".to_string()
                    selected_border="var(--status-info-border)".to_string() />
                <Chip label="Debug".into() selected=include_debug
                    title="Click to toggle, Ctrl+Click to show only this level".to_string()
                    on_click=move |ev: web_sys::MouseEvent| {
                        if ev.ctrl_key() || ev.meta_key() {
                            set_include_critical.set(false);
                            set_include_warning.set(false);
                            set_include_info.set(false);
                            set_include_debug.set(true);
                        } else {
                            set_include_debug.update(|v| *v = !*v);
                        }
                    } />
            </div>

            // ── Row 3: Search + advanced filters (collapsible) ─────────
            <div class="border-b">
                <div class="flex items-center gap-3 flex-wrap"
                     style="padding: var(--space-3) var(--space-6);">
                    <div class="flex-1" style="min-width: 180px; max-width: 360px;">
                        <TextInput
                            value=search_query
                            on_input=move |value| set_search_query.set(value)
                            placeholder="Search diagnostics..."
                            aria_label="Search diagnostics".to_string()
                        />
                    </div>
                    <div class="flex items-center gap-2">
                        <Tooltip text="Use regular expressions in search">
                            <span class="type-caption text-secondary">"Regex"</span>
                        </Tooltip>
                        <Toggle checked=regex_mode on_change=move |v| set_regex_mode.set(v) />
                    </div>
                    <div class="flex items-center gap-2">
                        <Tooltip text="Case-sensitive search">
                            <span class="type-caption text-secondary">"Case"</span>
                        </Tooltip>
                        <Toggle checked=case_sensitive on_change=move |v| set_case_sensitive.set(v) />
                    </div>
                    <Tooltip text="Number of surrounding events to show around each search match">
                        <span class="type-caption text-tertiary micro-label">"Context"</span>
                    </Tooltip>
                    <div class="flex items-center gap-2">
                        <NumberInput
                            value=context_before
                            on_change=move |v| set_context_before.set(v)
                            min=0.0
                            step=1.0
                            aria_label="Context events before match".to_string()
                        />
                        <span class="type-caption text-secondary">"before"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <NumberInput
                            value=context_after
                            on_change=move |v| set_context_after.set(v)
                            min=0.0
                            step=1.0
                            aria_label="Context events after match".to_string()
                        />
                        <span class="type-caption text-secondary">"after"</span>
                    </div>
                    <Divider vertical=true />
                    // Source + Provider filters
                    <div class="flex items-center gap-2">
                        <span class="type-caption text-tertiary micro-label">"Source"</span>
                        {source_options(&view_events.get()).into_iter().map(|source| {
                            let check = source.clone();
                            let click = source.clone();
                            let selected = Signal::derive(move || source_filter.get() == check);
                            view! {
                                <Chip
                                    label=source.clone()
                                    selected=selected
                                    on_click=move |_ev: web_sys::MouseEvent| set_source_filter.set(click.clone())
                                />
                            }
                        }).collect_view()}
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="type-caption text-tertiary micro-label">"Provider"</span>
                        {provider_options(&view_events.get()).into_iter().map(|provider| {
                            let check = provider.clone();
                            let click = provider.clone();
                            let selected = Signal::derive(move || provider_filter.get() == check);
                            view! {
                                <Chip
                                    label=provider.clone()
                                    selected=selected
                                    on_click=move |_ev: web_sys::MouseEvent| set_provider_filter.set(click.clone())
                                />
                            }
                        }).collect_view()}
                    </div>
                </div>

                {move || regex_error.get().map(|msg| view! {
                    <div
                        class="type-caption p-3 rounded-md"
                        style="margin: 0 var(--space-6) var(--space-3); \
                               background: var(--status-error-muted); color: var(--status-error-text);"
                    >
                        {msg}
                    </div>
                })}
            </div>

            // ── Content Area ────────────────────────────────────────────
            <div class="flex-1 grid" style="grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr); min-height: 0;">
                <div class="flex flex-col min-h-0">
                    // Sort controls at top of event list
                    <div
                        class="flex items-center gap-2 flex-wrap border-b"
                        style="padding: var(--space-2) var(--space-5); background: var(--surface-sunken);"
                    >
                        <span class="type-caption text-tertiary">"Sort"</span>
                        {ALL_SORT_FIELDS.iter().map(|&field| {
                            view! {
                                <button
                                    class="type-caption cursor-pointer select-none"
                                    title=format!("Sort by {} — click to cycle: add ↓ → ↑ → remove", field.label())
                                    style=move || {
                                        let crit = sort_criteria.get();
                                        let active = crit.iter().any(|(f, _)| *f == field);
                                        format!(
                                            "padding: var(--space-1) var(--space-3); \
                                             border-radius: var(--radius-md); \
                                             border: 1px solid {}; \
                                             background: {}; \
                                             color: {}; \
                                             font-weight: {}; \
                                             transition: all var(--duration-fast) var(--easing-standard);",
                                            if active { "var(--accent-primary)" } else { "var(--border-default)" },
                                            if active { "var(--surface-selected)" } else { "transparent" },
                                            if active { "var(--accent-primary)" } else { "var(--text-secondary)" },
                                            if active { "600" } else { "var(--type-label-weight)" },
                                        )
                                    }
                                    on:click=move |_| set_sort_criteria.update(|crit| toggle_sort(crit, field))
                                >
                                    {move || {
                                        let crit = sort_criteria.get();
                                        if let Some(pos) = crit.iter().position(|(f, _)| *f == field) {
                                            let arrow = if crit[pos].1 == SortDir::Desc { " ↓" } else { " ↑" };
                                            if crit.len() > 1 {
                                                format!("{}.{}{}", pos + 1, field.label(), arrow)
                                            } else {
                                                format!("{}{}", field.label(), arrow)
                                            }
                                        } else {
                                            field.label().to_string()
                                        }
                                    }}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                    // Scrollable event list
                    <div class="flex flex-col gap-3 overflow-y-auto p-5">
                    {move || {
                        let events = sorted_events.get();
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
                                                multi_selected=Signal::derive(move || selected_ids.get().contains(&event_id))
                                                query=search_query.get()
                                                regex_mode=regex_mode.get()
                                                case_sensitive=case_sensitive.get()
                                                on_select=Box::new(move |ctrl: bool, shift: bool| {
                                                    let ids: Vec<_> = sorted_events.get_untracked().iter().map(|e| e.id).collect();
                                                    let current_anchor = anchor_id.get_untracked();

                                                    let (new_sel, new_anchor) = if shift {
                                                        let start = current_anchor
                                                            .and_then(|a| ids.iter().position(|&id| id == a))
                                                            .unwrap_or(0);
                                                        let end = ids.iter().position(|&id| id == event_id).unwrap_or(0);
                                                        let (lo, hi) = if start <= end { (start, end) } else { (end, start) };
                                                        let range: BTreeSet<_> = ids[lo..=hi].iter().copied().collect();
                                                        if ctrl {
                                                            let mut existing = selected_ids.get_untracked();
                                                            existing.extend(range);
                                                            (existing, current_anchor)
                                                        } else {
                                                            (range, current_anchor)
                                                        }
                                                    } else if ctrl {
                                                        let mut existing = selected_ids.get_untracked();
                                                        if existing.contains(&event_id) {
                                                            existing.remove(&event_id);
                                                        } else {
                                                            existing.insert(event_id);
                                                        }
                                                        (existing, Some(event_id))
                                                    } else {
                                                        let mut s = BTreeSet::new();
                                                        s.insert(event_id);
                                                        (s, Some(event_id))
                                                    };

                                                    set_selected_ids.set(new_sel);
                                                    set_anchor_id.set(new_anchor);
                                                    set_selected_event_id.set(Some(event_id));
                                                })
                                            />
                                        }
                                    }).collect_view()}
                                </>
                            }.into_any()
                        }
                    }}
                    </div>
                </div>

                <div
                    class="flex flex-col overflow-y-auto border-l p-5 surface-raised"
                >
                    {move || {
                        if let Some(event) = selected_event.get() {
                            view! {
                                <div class="diagnostics-detail-content">
                                    <DiagnosticDetail
                                        event=event
                                        display_mode=display_mode.get()
                                        detail_mode=detail_mode.get()
                                        query=search_query.get()
                                        regex_mode=regex_mode.get()
                                        case_sensitive=case_sensitive.get()
                                    />
                                </div>
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

// ── Detail components ───────────────────────────────────────────────────────

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
    let attributes_json =
        serde_json::to_string_pretty(&attributes).unwrap_or_else(|_| "{}".to_owned());
    let query_for_raw = query.clone();
    let query_for_detail = query.clone();

    view! {
        <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
                <div class="flex items-center gap-2 flex-wrap">
                    <span class="type-caption text-secondary">{format_local_datetime(event.timestamp)}</span>
                    <span class="type-caption text-secondary">{format!("{:?}", event.source)}</span>
                    <span class="type-code-small text-tertiary">
                        {event.code.clone()}
                    </span>
                </div>
                <h3 class="type-title text-primary">{event.title.clone()}</h3>
                <p class="type-body text-secondary">{event.summary.clone()}</p>
            </div>

            {if display_mode == "event_data" {
                view! {
                    <DetailSection title="Structured Event">
                        <pre class="type-code-small text-primary whitespace-pre-wrap break-words" style="margin: 0;">
                            {highlight_text(raw_json, query_for_raw, regex_mode, case_sensitive)}
                        </pre>
                    </DetailSection>
                }.into_any()
            } else {
                view! {
                    <>
                        <DetailSection title="Readable Detail">
                            <div class="type-body text-primary whitespace-pre-wrap break-words">
                                {highlight_text(detail_text, query_for_detail, regex_mode, case_sensitive)}
                            </div>
                        </DetailSection>
                        <DetailSection title="Structured Fields">
                            <table class="w-full" style="border-collapse: collapse;">
                                {detail_rows.into_iter().map(|(label, value)| view! {
                                    <tr class="border-b">
                                        <td class="type-caption text-secondary py-2 pr-3" style="vertical-align: top;">
                                            {label}
                                        </td>
                                        <td class="type-body text-primary py-2 break-words">
                                            {value}
                                        </td>
                                    </tr>
                                }).collect_view()}
                            </table>
                        </DetailSection>
                        {(!attributes.is_empty() && detail_level != DiagnosticsDetailLevel::Overview).then(|| view! {
                            <DetailSection title="Attributes">
                                <pre class="type-code-small text-primary whitespace-pre-wrap break-words" style="margin: 0;">
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
        <Surface class="flex flex-col gap-3 p-4".to_string()>
            <h4 class="type-label text-primary">{title}</h4>
            {children()}
        </Surface>
    }
}

// ── Helper functions ────────────────────────────────────────────────────────

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

fn sync_selection_state(
    visible: &[DiagnosticEvent],
    selected_event_id: Option<crate::models::DiagnosticEventId>,
    selected_ids: &BTreeSet<crate::models::DiagnosticEventId>,
) -> SelectionSyncState {
    if visible.is_empty() {
        return SelectionSyncState {
            selected_event_id: None,
            selected_ids: BTreeSet::new(),
        };
    }

    let visible_ids: BTreeSet<_> = visible.iter().map(|event| event.id).collect();
    let mut next_selected_ids = selected_ids
        .iter()
        .copied()
        .filter(|id| visible_ids.contains(id))
        .collect::<BTreeSet<_>>();

    let next_selected_event_id = if selected_event_id.is_some_and(|id| visible_ids.contains(&id)) {
        selected_event_id
    } else {
        let first_visible_id = visible[0].id;
        if next_selected_ids.is_empty() {
            next_selected_ids.insert(first_visible_id);
        }
        Some(first_visible_id)
    };

    SelectionSyncState {
        selected_event_id: next_selected_event_id,
        selected_ids: next_selected_ids,
    }
}

fn filtered_events(
    events: &[DiagnosticEvent],
    query: &str,
    regex_mode: bool,
    case_sensitive: bool,
    context_before: usize,
    context_after: usize,
    levels: Vec<DiagnosticLevel>,
    source_filter: &str,
    provider_filter: &str,
) -> Vec<DiagnosticEvent> {
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

// ── Sort helpers ────────────────────────────────────────────────────────────

fn sort_events(events: &mut [DiagnosticEvent], criteria: &[(SortField, SortDir)]) {
    events.sort_by(|a, b| {
        for &(field, dir) in criteria {
            let ord = match field {
                SortField::Timestamp => a.timestamp.cmp(&b.timestamp),
                SortField::Severity => severity_ord(a.level).cmp(&severity_ord(b.level)),
                SortField::Source => format!("{:?}", a.source).cmp(&format!("{:?}", b.source)),
                SortField::Provider => {
                    let pa = a
                        .provider_id
                        .map(|p| p.display_name().to_owned())
                        .unwrap_or_default();
                    let pb = b
                        .provider_id
                        .map(|p| p.display_name().to_owned())
                        .unwrap_or_default();
                    pa.cmp(&pb)
                }
                SortField::Code => a.code.cmp(&b.code),
                SortField::Title => a.title.cmp(&b.title),
            };
            let ord = match dir {
                SortDir::Asc => ord,
                SortDir::Desc => ord.reverse(),
            };
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    });
}

fn severity_ord(level: DiagnosticLevel) -> u8 {
    match level {
        DiagnosticLevel::Critical => 0,
        DiagnosticLevel::Warning => 1,
        DiagnosticLevel::Info => 2,
        DiagnosticLevel::Debug => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::sync_selection_state;
    use crate::models::DiagnosticEvent;
    use std::collections::BTreeSet;

    fn event_with_id(id: crate::models::DiagnosticEventId) -> DiagnosticEvent {
        DiagnosticEvent {
            id,
            ..DiagnosticEvent::default()
        }
    }

    #[test]
    fn sync_selection_clears_once_for_empty_visible_list() {
        let stale_id = crate::models::DiagnosticEventId::new();
        let mut selected_ids = BTreeSet::new();
        selected_ids.insert(stale_id);

        let next = sync_selection_state(&[], Some(stale_id), &selected_ids);

        assert_eq!(next.selected_event_id, None);
        assert!(next.selected_ids.is_empty());

        let stable = sync_selection_state(&[], next.selected_event_id, &next.selected_ids);
        assert_eq!(stable, next);
    }

    #[test]
    fn sync_selection_keeps_existing_valid_selection() {
        let first_id = crate::models::DiagnosticEventId::new();
        let second_id = crate::models::DiagnosticEventId::new();
        let visible = vec![event_with_id(first_id), event_with_id(second_id)];
        let mut selected_ids = BTreeSet::new();
        selected_ids.insert(second_id);

        let next = sync_selection_state(&visible, Some(second_id), &selected_ids);

        assert_eq!(next.selected_event_id, Some(second_id));
        assert_eq!(next.selected_ids, selected_ids);
    }
}

fn toggle_sort(criteria: &mut Vec<(SortField, SortDir)>, field: SortField) {
    if let Some(pos) = criteria.iter().position(|(f, _)| *f == field) {
        match criteria[pos].1 {
            SortDir::Desc => criteria[pos].1 = SortDir::Asc,
            SortDir::Asc => {
                criteria.remove(pos);
            }
        }
    } else {
        criteria.push((field, SortDir::Desc));
    }
}

// ── Search helpers ──────────────────────────────────────────────────────────

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
        .unwrap_or_else(|| {
            Some(
                "Invalid regular expression. The last valid result set remains visible.".to_owned(),
            )
        })
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

fn highlight_text(text: String, query: String, regex_mode: bool, case_sensitive: bool) -> AnyView {
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

// ── Detail helpers ──────────────────────────────────────────────────────────

fn detail_rows(
    event: &DiagnosticEvent,
    detail_level: DiagnosticsDetailLevel,
) -> Vec<(&'static str, String)> {
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
            event
                .provider_id
                .map(|provider| provider.display_name().to_owned())
                .unwrap_or_else(|| "—".to_owned()),
        ));
        rows.push((
            "Binding",
            event
                .binding_id
                .map(|binding| binding.0.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ));
        rows.push((
            "Run",
            event
                .run_id
                .map(|run| run.0.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ));
    }

    if detail_level == DiagnosticsDetailLevel::Verbose {
        rows.push((
            "Round",
            event
                .round_id
                .map(|round| round.0.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ));
        rows.push((
            "Message",
            event
                .message_id
                .map(|message| message.0.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ));
        rows.push((
            "Dispatch",
            event
                .dispatch_id
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

// ── Export helpers ───────────────────────────────────────────────────────────

fn render_export_payload(
    display_mode: &str,
    detail_mode: &str,
    events: &[DiagnosticEvent],
) -> String {
    if display_mode == "event_data" {
        events
            .iter()
            .map(|event| serde_json::to_string(event).unwrap_or_else(|_| "{}".to_owned()))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        let detail_level = selected_detail_level(detail_mode);
        let mut out = String::new();
        out.push_str("Chatmux Diagnostics Report\n\n");
        for event in events {
            out.push_str(&format!(
                "[{:?}] {} | {:?} | {}\n{}\n{}\n\n",
                event.level,
                format_local_datetime(event.timestamp),
                event.source,
                event.code,
                event.title,
                event.detail
            ));
            if detail_level != DiagnosticsDetailLevel::Overview {
                out.push_str(&format!(
                    "summary: {}\nprovider: {}\nworkspace: {}\n\n",
                    event.summary,
                    event
                        .provider_id
                        .map(|provider| provider.display_name().to_owned())
                        .unwrap_or_else(|| "—".to_owned()),
                    event.workspace_id.0
                ));
            }
        }
        out
    }
}
