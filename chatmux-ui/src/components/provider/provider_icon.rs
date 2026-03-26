//! Provider icon component.
//!
//! Renders the provider's icon mark at the specified size.
//! Uses the provider's solid color.

use leptos::prelude::*;

use super::Provider;

/// Provider icon component.
#[component]
pub fn ProviderIcon(
    /// Which provider.
    provider: Provider,
    /// Size in pixels.
    #[prop(default = 14)]
    size: u32,
) -> impl IntoView {
    view! {
        <span
            class="provider-icon"
            style=format!(
                "display: inline-flex; align-items: center; justify-content: center; \
                 width: {size}px; height: {size}px; \
                 font-size: {}px; line-height: 1; \
                 color: {}; flex-shrink: 0;",
                (size as f32 * 0.8) as u32,
                provider.solid_color(),
            )
            aria-hidden="true"
        >
            {provider.icon_char()}
        </span>
    }
}
