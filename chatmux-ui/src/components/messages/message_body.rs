//! Message body rendering (§3.3).
//!
//! Renders message text with basic formatting:
//! paragraphs, code fences (surface-sunken, font-mono),
//! inline code, headings, lists, blockquotes.
//! Long messages collapse after ~8 lines with "Show more".

use leptos::prelude::*;

/// Render a message body string.
///
/// For now this does simple paragraph splitting and code fence detection.
/// A full Markdown parser can be added later for richer rendering.
#[component]
pub fn MessageBody(
    /// Raw message text.
    text: String,
) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);

    // Simple block parsing: split on double newlines for paragraphs,
    // detect ``` fences for code blocks.
    let blocks = parse_blocks(&text);
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
                    Block::Paragraph(text) => view! {
                        <p class="mb-3">{text}</p>
                    }.into_any(),
                    Block::CodeFence { lang, code } => view! {
                        <pre
                            class="type-code surface-sunken mb-3"
                            style="padding: var(--space-4); border-radius: var(--radius-md); \
                                   overflow-x: auto;"
                        >
                            {lang.map(|l| view! {
                                <span class="type-caption text-secondary mb-2 block">
                                    {l}
                                </span>
                            })}
                            <code>{code}</code>
                        </pre>
                    }.into_any(),
                    Block::Blockquote(text) => view! {
                        <blockquote
                            class="mb-3"
                            style="\
                            border-left: 2px solid var(--border-subtle); \
                            padding-left: var(--space-4); \
                            color: var(--text-secondary);">
                            {text}
                        </blockquote>
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

/// Simple block types for rendering.
enum Block {
    Paragraph(String),
    CodeFence { lang: Option<String>, code: String },
    Blockquote(String),
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
                blocks.push(Block::Paragraph(current_paragraph.trim().to_string()));
                current_paragraph.clear();
            }

            // Code fence
            let lang = line.strip_prefix("```").map(|l| l.trim().to_string()).filter(|l| !l.is_empty());
            let mut code = String::new();
            while let Some(code_line) = lines.next() {
                if code_line.starts_with("```") {
                    break;
                }
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(code_line);
            }
            blocks.push(Block::CodeFence { lang, code });
        } else if line.starts_with('>') {
            // Flush paragraph
            if !current_paragraph.is_empty() {
                blocks.push(Block::Paragraph(current_paragraph.trim().to_string()));
                current_paragraph.clear();
            }
            let quote_text = line.strip_prefix('>').unwrap_or(line).trim().to_string();
            blocks.push(Block::Blockquote(quote_text));
        } else if line.trim().is_empty() {
            // Paragraph break
            if !current_paragraph.is_empty() {
                blocks.push(Block::Paragraph(current_paragraph.trim().to_string()));
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
        blocks.push(Block::Paragraph(current_paragraph.trim().to_string()));
    }

    if blocks.is_empty() {
        blocks.push(Block::Paragraph(text.to_string()));
    }

    blocks
}
