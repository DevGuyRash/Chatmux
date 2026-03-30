//! Rich diagnostic event row for the diagnostics explorer.

use leptos::prelude::*;
use regex::{Regex, RegexBuilder};

use crate::models::{DiagnosticEvent, DiagnosticLevel};

#[component]
pub fn EventRow(
    event: DiagnosticEvent,
    /// Whether this row is the detail-panel focus.
    selected: Signal<bool>,
    /// Whether this row is part of a multi-selection.
    multi_selected: Signal<bool>,
    query: String,
    regex_mode: bool,
    case_sensitive: bool,
    /// Click handler — receives (ctrl_or_meta, shift).
    on_select: Box<dyn Fn(bool, bool) + Send>,
) -> impl IntoView {
    let severity = severity_tokens(event.level);
    let time = event.timestamp.format("%H:%M:%S").to_string();
    let date = event.timestamp.format("%m-%d").to_string();
    let provider = event
        .provider_id
        .map(|provider| provider.display_name().to_owned())
        .unwrap_or_else(|| "System".to_owned());
    let title = event.title.clone();
    let summary = event.summary.clone();
    let code = event.code.clone();
    let tags = event.tags.clone();
    let query_for_title = query.clone();
    let query_for_summary = query.clone();

    view! {
        <button
            class=move || {
                let mut cls = "diagnostics-event-row".to_string();
                if selected.get() { cls.push_str(" diagnostics-event-row--selected"); }
                if multi_selected.get() { cls.push_str(" diagnostics-event-row--multi-selected"); }
                cls
            }
            style=move || {
                let is_sel = selected.get();
                let is_multi = multi_selected.get();
                format!(
                    "width: 100%; text-align: left; \
                     border: 1px solid {}; \
                     border-left: 3px solid {}; \
                     border-radius: var(--radius-md); \
                     background: {}; \
                     padding: var(--space-4) var(--space-5); \
                     display: flex; flex-direction: column; \
                     gap: var(--space-2); cursor: pointer;",
                    if is_sel { "var(--accent-primary)" }
                    else if is_multi { "var(--accent-secondary)" }
                    else { "var(--border-subtle)" },
                    severity.border,
                    if is_multi { "var(--surface-selected)" }
                    else if is_sel { "var(--surface-raised)" }
                    else { "transparent" },
                )
            }
            on:click=move |ev| {
                let ctrl = ev.ctrl_key() || ev.meta_key();
                let shift = ev.shift_key();
                on_select(ctrl, shift);
            }
        >
            <div class="flex items-center gap-2 flex-wrap">
                <span
                    class="type-caption-strong"
                    style=format!(
                        "padding: var(--space-1) var(--space-3); border-radius: var(--radius-full); \
                         background: {}; color: {};",
                        severity.background, severity.text,
                    )
                >
                    {severity.label}
                </span>
                <span class="type-caption text-tertiary">{date}</span>
                <span class="type-caption text-secondary">{time}</span>
                <span class="type-caption text-tertiary">"·"</span>
                <span class="type-caption text-secondary">{provider}</span>
                <span style="flex: 1;"></span>
                <span class="type-code-small text-tertiary" style="font-family: var(--font-mono);">
                    {code}
                </span>
            </div>
            <div class="type-label text-primary" style="word-break: break-word;">
                {highlighted_text(title, query_for_title, regex_mode, case_sensitive)}
            </div>
            <div class="type-caption text-secondary truncate">
                {highlighted_text(summary, query_for_summary, regex_mode, case_sensitive)}
            </div>
            {(!tags.is_empty()).then(|| view! {
                <div class="flex items-center gap-1 flex-wrap">
                    {tags.iter().take(4).map(|tag| {
                        view! {
                            <span
                                class="type-caption"
                                style="padding: var(--space-1) var(--space-2); border-radius: var(--radius-full); \
                                       background: var(--surface-sunken); color: var(--text-tertiary); \
                                       border: 1px solid var(--border-subtle);"
                            >
                                {tag.clone()}
                            </span>
                        }
                    }).collect_view()}
                </div>
            })}
        </button>
    }
}

struct SeverityTokens {
    label: &'static str,
    background: &'static str,
    text: &'static str,
    border: &'static str,
}

fn severity_tokens(level: DiagnosticLevel) -> SeverityTokens {
    match level {
        DiagnosticLevel::Critical => SeverityTokens {
            label: "Critical",
            background: "var(--status-error-muted)",
            text: "var(--status-error-text)",
            border: "var(--status-error-solid)",
        },
        DiagnosticLevel::Warning => SeverityTokens {
            label: "Warning",
            background: "var(--status-warning-muted)",
            text: "var(--status-warning-text)",
            border: "var(--status-warning-solid)",
        },
        DiagnosticLevel::Info => SeverityTokens {
            label: "Info",
            background: "var(--status-info-muted)",
            text: "var(--status-info-text)",
            border: "var(--status-info-solid)",
        },
        DiagnosticLevel::Debug => SeverityTokens {
            label: "Debug",
            background: "var(--surface-raised)",
            text: "var(--text-secondary)",
            border: "var(--border-default)",
        },
    }
}

fn highlighted_text(
    text: String,
    query: String,
    regex_mode: bool,
    case_sensitive: bool,
) -> AnyView {
    if query.trim().is_empty() {
        return view! { <>{text}</> }.into_any();
    }

    let Some(regex) = build_regex(&query, regex_mode, case_sensitive) else {
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
        view! { <>{text}</> }.into_any()
    } else {
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
}

fn build_regex(query: &str, regex_mode: bool, case_sensitive: bool) -> Option<Regex> {
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
