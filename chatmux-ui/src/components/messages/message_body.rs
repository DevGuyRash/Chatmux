//! Message body rendering (§3.3).
//!
//! Renders message text with basic formatting:
//! paragraphs, code fences (surface-sunken, font-mono),
//! inline code, headings, lists, blockquotes.
//! Long messages collapse after ~8 lines with "Show more".

use crate::models::Block;
use leptos::prelude::*;

/// Render a message body string.
///
/// For now this does simple paragraph splitting and code fence detection.
/// A full Markdown parser can be added later for richer rendering.
#[component]
pub fn MessageBody(
    /// Raw message text.
    text: String,
    /// Canonical structured blocks captured by the provider adapter.
    #[prop(default = Vec::new())]
    structured_blocks: Vec<Block>,
) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);

    // Simple block parsing: split on double newlines for paragraphs,
    // detect ``` fences for code blocks.
    let blocks = if structured_blocks.is_empty() {
        parse_blocks(&text)
    } else {
        structured_blocks
    };
    let is_long = text.lines().count() > 8;

    view! {
        <div
            class="message-body type-body text-primary"
            style=move || format!(
                "overflow: hidden; {}",
                if is_long && !expanded.get() {
                    "max-height: 160px; -webkit-mask-image: linear-gradient(to bottom, black 70%, transparent 100%);"
                } else {
                    ""
                },
            )
        >
            {blocks.into_iter().map(|block| {
                match block {
                    Block::Paragraph { text } => view! {
                        <p class="mb-3">{text}</p>
                    }.into_any(),
                    Block::Heading { level, text } => view! {
                        <p class=if level <= 2 { "type-title text-primary mb-3" } else { "type-subtitle text-primary mb-3" }>
                            {text}
                        </p>
                    }.into_any(),
                    Block::CodeFence { language, code } => view! {
                        <pre
                            class="type-code surface-sunken mb-3"
                            style="padding: var(--space-4); border-radius: var(--radius-md); \
                                   overflow-x: auto;"
                        >
                            {language.map(|l| view! {
                                <span class="type-caption text-secondary mb-2 block">
                                    {l}
                                </span>
                            })}
                            <code>{code}</code>
                        </pre>
                    }.into_any(),
                    Block::BulletedList { items } => view! {
                        <ul class="mb-3" style="padding-left: var(--space-6); list-style: disc;">
                            {items.into_iter().map(|item| view! { <li class="mb-1">{item}</li> }).collect_view()}
                        </ul>
                    }.into_any(),
                    Block::NumberedList { items } => view! {
                        <ol class="mb-3" style="padding-left: var(--space-6); list-style: decimal;">
                            {items.into_iter().map(|item| view! { <li class="mb-1">{item}</li> }).collect_view()}
                        </ol>
                    }.into_any(),
                    Block::Quote { text } => view! {
                        <blockquote
                            class="mb-3"
                            style="\
                            border-left: 2px solid var(--border-subtle); \
                            padding-left: var(--space-4); \
                            color: var(--text-secondary);">
                            {text}
                        </blockquote>
                    }.into_any(),
                    Block::Table { headers, rows } => view! {
                        <div class="mb-3 overflow-x-auto border rounded-md">
                            <table class="type-caption" style="width: 100%; border-collapse: collapse;">
                                <thead class="surface-sunken">
                                    <tr>{headers.into_iter().map(|header| view! {
                                        <th class="text-left text-primary" style="padding: var(--space-3); border-bottom: 1px solid var(--border-subtle);">{header}</th>
                                    }).collect_view()}</tr>
                                </thead>
                                <tbody>{rows.into_iter().map(|row| view! {
                                    <tr>{row.into_iter().map(|cell| view! {
                                        <td class="text-secondary" style="padding: var(--space-3); border-bottom: 1px solid var(--border-subtle);">{cell}</td>
                                    }).collect_view()}</tr>
                                }).collect_view()}</tbody>
                            </table>
                        </div>
                    }.into_any(),
                }
            }).collect_view()}
        </div>

        // "Show more" / "Show less" toggle
        {is_long.then(|| view! {
            <button
                class="type-caption cursor-pointer"
                style="color: var(--text-link); background: none; border: none; \
                       padding: var(--space-2) 0;"
                on:click=move |_| set_expanded.update(|v| *v = !*v)
            >
                {move || if expanded.get() { "Show less" } else { "Show more" }}
            </button>
        })}
    }
}

/// Parse text into blocks.
fn parse_blocks(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut lines = text.lines().peekable();
    let mut current_paragraph = String::new();

    while let Some(line) = lines.next() {
        if line.starts_with("```") {
            // Flush current paragraph
            if !current_paragraph.is_empty() {
                blocks.push(Block::Paragraph {
                    text: current_paragraph.trim().to_string(),
                });
                current_paragraph.clear();
            }

            // Code fence
            let language = line
                .strip_prefix("```")
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty());
            let mut code = String::new();
            for code_line in lines.by_ref() {
                if code_line.starts_with("```") {
                    break;
                }
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(code_line);
            }
            blocks.push(Block::CodeFence { language, code });
        } else if line.starts_with('>') {
            // Flush paragraph
            if !current_paragraph.is_empty() {
                blocks.push(Block::Paragraph {
                    text: current_paragraph.trim().to_string(),
                });
                current_paragraph.clear();
            }
            let quote_text = line.strip_prefix('>').unwrap_or(line).trim().to_string();
            blocks.push(Block::Quote { text: quote_text });
        } else if line.trim().is_empty() {
            // Paragraph break
            if !current_paragraph.is_empty() {
                blocks.push(Block::Paragraph {
                    text: current_paragraph.trim().to_string(),
                });
                current_paragraph.clear();
            }
        } else {
            if !current_paragraph.is_empty() {
                current_paragraph.push(' ');
            }
            current_paragraph.push_str(line);
        }
    }

    if !current_paragraph.is_empty() {
        blocks.push(Block::Paragraph {
            text: current_paragraph.trim().to_string(),
        });
    }

    if blocks.is_empty() {
        blocks.push(Block::Paragraph {
            text: text.to_string(),
        });
    }

    blocks
}
