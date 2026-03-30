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
            style=format!(
                "display: inline-flex; align-items: center; \
                 padding: var(--space-2) var(--space-4); \
                 border-radius: var(--radius-xl); \
                 background: {}; \
                 color: {};",
                if filled { provider.muted_color() } else { "transparent".to_string() },
                provider.text_color(),
            )
        >
            <ProviderIcon provider=provider size=12 />
            {provider.label()}
        </span>
    }
}
