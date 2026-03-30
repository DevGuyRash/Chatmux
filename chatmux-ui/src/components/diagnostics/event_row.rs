//! Rich diagnostic event row for the diagnostics explorer.

use leptos::prelude::*;
use regex::{Regex, RegexBuilder};

use crate::models::{DiagnosticEvent, DiagnosticLevel};

#[component]
pub fn EventRow(
    event: DiagnosticEvent,
    selected: Signal<bool>,
    query: String,
    regex_mode: bool,
    case_sensitive: bool,
    on_select: Box<dyn Fn() + Send>,
) -> impl IntoView {
    let severity = severity_tokens(event.level);
    let timestamp = event.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let provider = event
        .provider_id
        .map(|provider| provider.display_name().to_owned())
        .unwrap_or_else(|| "General".to_owned());
    let source = format!("{:?}", event.source);
    let title = event.title.clone();
    let summary = event.summary.clone();
    let query_for_title = query.clone();
    let query_for_summary = query.clone();

    view! {
        <button
            class="diagnostics-event-row"
            style=move || format!(
                "width: 100%; text-align: left; border: 1px solid {}; border-radius: var(--radius-md); \
                 background: {}; padding: var(--space-4); display: flex; flex-direction: column; \
                 gap: var(--space-3); cursor: pointer;",
                if selected.get() { "var(--accent-primary)" } else { "var(--border-subtle)" },
                if selected.get() { "var(--surface-raised)" } else { "var(--surface-sunken)" },
            )
            on:click=move |_| on_select()
        >
            <div class="flex items-start justify-between gap-3">
                <div class="flex flex-col gap-2" style="min-width: 0;">
                    <div class="flex items-center gap-2 flex-wrap">
                        <span
                            class="type-caption-strong"
                            style=format!(
                                "padding: var(--space-1) var(--space-2); border-radius: var(--radius-sm); \
                                 background: {}; color: {};",
                                severity.background, severity.text,
                            )
                        >
                            {severity.label}
                        </span>
                        <span class="type-caption text-secondary">{timestamp}</span>
                        <span class="type-caption text-secondary">{provider}</span>
                        <span class="type-caption text-tertiary">{source}</span>
                    </div>
                    <div class="type-label text-primary" style="word-break: break-word;">
                        {highlighted_text(title, query_for_title, regex_mode, case_sensitive)}
                    </div>
                    <div class="type-body text-secondary" style="word-break: break-word;">
                        {highlighted_text(summary, query_for_summary, regex_mode, case_sensitive)}
                    </div>
                </div>
                <div class="type-code-small text-tertiary" style="font-family: var(--font-mono);">
                    {event.code.clone()}
                </div>
            </div>

            <div class="flex items-center gap-2 flex-wrap">
                {event.tags.iter().take(4).map(|tag| {
                    view! {
                        <span
                            class="type-caption"
                            style="padding: var(--space-1) var(--space-2); border-radius: var(--radius-sm); \
                                   background: var(--surface-elevated, var(--surface-raised)); color: var(--text-secondary);"
                        >
                            {tag.clone()}
                        </span>
                    }
                }).collect_view()}
            </div>
        </button>
    }
}

struct SeverityTokens {
    label: &'static str,
    background: &'static str,
    text: &'static str,
}

fn severity_tokens(level: DiagnosticLevel) -> SeverityTokens {
    match level {
        DiagnosticLevel::Critical => SeverityTokens {
            label: "Critical",
            background: "var(--status-error-muted)",
            text: "var(--status-error-text)",
        },
        DiagnosticLevel::Warning => SeverityTokens {
            label: "Warning",
            background: "var(--status-warning-muted)",
            text: "var(--status-warning-text)",
        },
        DiagnosticLevel::Info => SeverityTokens {
            label: "Info",
            background: "var(--status-info-muted)",
            text: "var(--status-info-text)",
        },
        DiagnosticLevel::Debug => SeverityTokens {
            label: "Debug",
            background: "var(--surface-raised)",
            text: "var(--text-secondary)",
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
                                       padding: 0 2px; border-radius: var(--radius-xs, var(--radius-sm));"
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
