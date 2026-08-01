//! Provider chip component.
//!
//! Provider icon + name, used in edge policy nodes, target selector, etc.

use leptos::prelude::*;

use super::{Provider, provider_icon::ProviderIcon};

/// Provider chip — icon + name in a pill.
#[component]
pub fn ProviderChip(
    /// Which provider.
    provider: Provider,
    /// Whether to use muted background fill.
    #[prop(default = false)]
    filled: bool,
) -> impl IntoView {
    view! {
        <span
            class="provider-chip flex items-center gap-2 select-none type-caption-strong"
            class:provider-chip--plain=!filled
            // Only the channel binding is inline; the pill's shape, edge and
            // hover belong to `.provider-chip` in components.css.
            style=provider.channel_vars()
        >
            <ProviderIcon provider=provider size=12 />
            {provider.label()}
        </span>
    }
}
