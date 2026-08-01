//! Provider icon component.
//!
//! Renders the provider's icon mark at the specified size.
//! Uses the provider's solid color.

use leptos::prelude::*;

use crate::components::primitives::icon::{Icon, IconKind};

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
            // The four provider marks are geometric text glyphs that render
            // monochrome and take the channel colour. "You" and "System" had no
            // such glyph and fell back to colour emoji, so those two resolve to
            // the SVG set instead.
            {match provider {
                Provider::User => view! {
                    <Icon kind=IconKind::PersonSilhouette size=(size as f32 * 0.8) as u32 />
                }.into_any(),
                Provider::System => view! {
                    <Icon kind=IconKind::GearSmall size=(size as f32 * 0.8) as u32 />
                }.into_any(),
                _ => view! { {provider.icon_char()} }.into_any(),
            }}
        </span>
    }
}
