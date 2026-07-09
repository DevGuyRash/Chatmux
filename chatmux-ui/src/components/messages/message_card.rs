//! Message card component (§3.3).
//!
//! Each message is a card with 3px left border in provider color.
//! Attribution row: provider icon + name + timestamp + round badge.
//! Body: rendered structured blocks.

use leptos::prelude::*;

use super::message_body::MessageBody;
use crate::components::primitives::badge::{Badge, BadgeVariant};
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::icon::{Icon, IconKind};
use crate::components::provider::Provider;
use crate::components::provider::provider_icon::ProviderIcon;
use crate::layout::responsive::LayoutMode;
use crate::models::Message;
use crate::time::format_local_time;

/// Message card component.
#[component]
pub fn MessageCard(
    /// The message to display.
    message: Message,
    /// Whether this card is in selection mode (checkbox visible).
    #[prop(default = false)]
    selection_mode: bool,
    /// Whether this card is selected.
    #[prop(default = false)]
    selected: bool,
    /// Called when the card is clicked (for inspection).
    #[prop(optional)]
    on_click: Option<Box<dyn Fn() + Send>>,
    /// Called when the selection checkbox is toggled.
    #[prop(optional)]
    on_toggle_select: Option<Box<dyn Fn() + Send>>,
    /// Called when the user wants to branch from this message.
    #[prop(optional)]
    on_branch: Option<Box<dyn Fn() + Send>>,
) -> impl IntoView {
    let layout_mode = expect_context::<ReadSignal<LayoutMode>>();
    let provider = Provider::from_provider_id(message.participant_id);
    let timestamp = format_local_time(message.timestamp);
    let aria_label = format!("{} at {}", provider.label(), &timestamp);
    let body = message.body_text.clone();
    let round = message.round;
    let branch_index = message.branch_index;
    let child_count = message.child_message_ids.len();
    let has_parent = message.parent_message_id.is_some();
    let branch_handler = on_branch;

    view! {
        <div
            class="message-card message-card--entering surface-raised cursor-pointer transition-colors"
            role="article"
            aria-label=aria_label
            style=move || format!(
                "border-left: 3px solid {}; \
                 border-radius: var(--radius-md); \
                 padding: {}; \
                 background: {};",
                provider.border_color(),
                match layout_mode.get() {
                    LayoutMode::Sidebar => "var(--space-5)",
                    LayoutMode::FullTab => "var(--space-6)",
                },
                if selected { "var(--surface-selected)" } else { "var(--surface-raised)" },
            )
            on:click=move |_| {
                if let Some(ref handler) = on_click {
                    handler();
                }
            }
        >
            // Attribution row
            <div class="flex items-center gap-2 mb-3">
                // Checkbox (selection mode only)
                {selection_mode.then(|| {
                    let handler = on_toggle_select;
                    view! {
                        <span
                            class="cursor-pointer"
                            style=format!(
                                "width: 16px; height: 16px; border-radius: var(--radius-sm); \
                                 border: 1.5px solid {}; background: {}; \
                                 display: inline-flex; align-items: center; justify-content: center; \
                                 flex-shrink: 0; font-size: 10px; color: var(--text-inverse);",
                                if selected { "var(--accent-primary)" } else { "var(--border-default)" },
                                if selected { "var(--accent-primary)" } else { "transparent" },
                            )
                            on:click=move |ev| {
                                ev.stop_propagation();
                                if let Some(ref h) = handler {
                                    h();
                                }
                            }
                        >
                            {selected.then(|| "✓")}
                        </span>
                    }
                })}

                // Provider icon + name
                <ProviderIcon provider=provider size=14 />
                <span
                    class="type-caption-strong"
                    style=format!("color: {};", provider.text_color())
                >
                    {provider.label()}
                </span>

                // Timestamp
                <span class="type-caption text-secondary">{timestamp}</span>

                // Round badge
                {round.map(|r| view! {
                    <Badge>{format!("R{r}")}</Badge>
                })}

                {branch_index.map(|index| view! {
                    <Badge variant=BadgeVariant::Info>{format!("Branch {index}")}</Badge>
                })}

                {has_parent.then(|| view! {
                    <Badge variant=BadgeVariant::Accent>"Reply"</Badge>
                })}

                {(child_count > 0).then(|| view! {
                    <Badge variant=BadgeVariant::Neutral>
                        {format!(
                            "{} {}",
                            child_count,
                            if child_count == 1 { "reply" } else { "replies" },
                        )}
                    </Badge>
                })}

                <span class="flex-1"></span>

                {branch_handler.map(|handler| view! {
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        title="Branch from this message".to_string()
                        aria_label="Branch from this message".to_string()
                        on_click=Box::new(move |ev| {
                            ev.stop_propagation();
                            handler();
                        })
                    >
                        <Icon kind=IconKind::GitBranch size=14 />
                    </Button>
                })}
            </div>

            // Message body
            <MessageBody text=body />
        </div>
    }
}
