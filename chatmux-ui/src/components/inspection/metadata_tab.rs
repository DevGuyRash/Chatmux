//! Metadata tab (§3.4).
//!
//! Key-value table showing message metadata:
//! Message ID, Workspace, Participant, Role, Round, Run, Timestamp,
//! Character Count, Dispatch ID, Source Binding, Delivery Cursor State, Tags.

use leptos::prelude::*;

use crate::models::Message;

/// Metadata tab content.
#[component]
pub fn MetadataTab(
    /// The message to display metadata for.
    message: Message,
) -> impl IntoView {
    let rows = vec![
        ("Message ID", format!("{}", message.id.0)),
        ("Participant", message.participant_id.display_name().to_string()),
        ("Role", format!("{:?}", message.role)),
        ("Round", message.round.map(|r| r.to_string()).unwrap_or_else(|| "—".to_string())),
        ("Timestamp", message.timestamp.to_rfc3339()),
        ("Character Count", message.body_text.len().to_string()),
        ("Dispatch ID", message.dispatch_id.map(|d| d.0.to_string()).unwrap_or_else(|| "—".to_string())),
        ("Tags", if message.tags.is_empty() { "—".to_string() } else { message.tags.join(", ") }),
    ];

    view! {
        <div class="metadata-tab">
            <table style="width: 100%; border-collapse: collapse;">
                {rows.into_iter().map(|(label, value)| {
                    let is_mono = matches!(label, "Message ID" | "Dispatch ID" | "Timestamp");
                    view! {
                        <tr style="border-bottom: 1px solid var(--border-subtle);">
                            <td
                                class="type-caption text-secondary"
                                style="padding: var(--space-3) var(--space-4) var(--space-3) 0; \
                                       white-space: nowrap; vertical-align: top;"
                            >
                                {label}
                            </td>
                            <td
                                style=format!(
                                    "padding: var(--space-3) 0; color: var(--text-primary); \
                                     word-break: break-all; {}",
                                    if is_mono { "font-family: var(--font-mono); font-size: var(--type-code-small-size);" } else { "font-size: var(--type-body-size);" },
                                )
                            >
                                {value}
                            </td>
                        </tr>
                    }
                }).collect_view()}
            </table>
        </div>
    }
}
