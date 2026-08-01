//! Message card component (§3.3).
//!
//! Each message is a card with 3px left border in provider color.
//! Attribution row: provider icon + name + timestamp + round badge.
//! Body: rendered structured blocks.

use leptos::prelude::*;

use super::message_body::MessageBody;
use crate::components::primitives::badge::{Badge, BadgeVariant};
use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::checkbox::Checkbox;
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
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    /// Called when the selection checkbox is toggled.
    #[prop(optional)]
    on_toggle_select: Option<Box<dyn Fn() + Send + Sync>>,
    /// Called when the user wants to branch from this message.
    #[prop(optional)]
    on_branch: Option<Box<dyn Fn() + Send>>,
) -> impl IntoView {
    let layout_mode = expect_context::<LayoutMode>();
    let provider = Provider::from_provider_id(message.participant_id);
    let timestamp = format_local_time(message.timestamp);
    let aria_label = format!("{} at {}", provider.label(), &timestamp);
    let body = message.body_text.clone();
    let body_blocks = message.body_blocks.clone();
    let round = message.round;
    let branch_index = message.branch_index;
    let child_count = message.child_message_ids.len();
    let has_parent = message.parent_message_id.is_some();
    let branch_handler = on_branch;
    let click_handler = StoredValue::new(on_click);

    view! {
        <div
            // Selection is a class, not an inline background. The inline form
            // applied the tint but not the accent border `.message-card--selected`
            // carries, so a selected card lost the one place the design language
            // reserves accent for selection.
            class="message-card message-card--entering surface-raised cursor-pointer transition-colors"
            class:message-card--selected=selected
            role="article"
            tabindex="0"
            aria-selected=if selected { "true" } else { "false" }
            aria-label=aria_label
            style=format!(
                "{} border-radius: var(--radius-md); padding: {};",
                provider.channel_vars(),
                match layout_mode {
                    LayoutMode::Sidebar => "var(--space-5)",
                    LayoutMode::FullTab => "var(--space-6)",
                },
            )
            on:click=move |_| {
                click_handler.with_value(|handler| {
                    if let Some(handler) = handler {
                        handler();
                    }
                });
            }
            on:keydown=move |event| {
                if matches!(event.key().as_str(), "Enter" | " ") {
                    event.prevent_default();
                    click_handler.with_value(|handler| {
                        if let Some(handler) = handler {
                            handler();
                        }
                    });
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
                            on:click=move |ev| {
                                ev.stop_propagation();
                            }
                        >
                            <Checkbox
                                checked=Signal::derive(move || selected)
                                on_change=move |_| {
                                    if let Some(ref handler) = handler {
                                        handler();
                                    }
                                }
                                label="Select message"
                            />
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
            <MessageBody text=body structured_blocks=body_blocks />
        </div>
    }
}
