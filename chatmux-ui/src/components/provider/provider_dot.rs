//! Provider health indicator dot.
//!
//! Small circular dot (6-8px) showing provider health.
//! Color from status color, paired with provider icon.

use leptos::prelude::*;

use super::{HealthState, Provider};

/// Small provider health indicator dot.
#[component]
pub fn ProviderDot(
    /// Which provider.
    provider: Provider,
    /// Health state.
    health: ReadSignal<HealthState>,
    /// Dot size in pixels.
    #[prop(default = 6)]
    size: u32,
    /// Whether to show the provider icon alongside.
    #[prop(default = false)]
    show_icon: bool,
) -> impl IntoView {
    view! {
        <span
            class="provider-dot flex items-center gap-1"
            title=move || format!("{}: {}", provider.label(), health.get().label())
            aria-label=move || format!("{} {}", provider.label(), health.get().label())
        >
            {show_icon.then(|| view! {
                <span
                    style=format!(
                        "font-size: {}px; color: {};",
                        size + 6,
                        provider.solid_color(),
                    )
                >
                    {provider.icon_char()}
                </span>
            })}
            <span
                style=move || format!(
                    "display: inline-block; \
                     width: {size}px; height: {size}px; \
                     border-radius: var(--radius-full); \
                     background: var(--{}-solid); \
                     flex-shrink: 0; \
                     transition: background var(--duration-fast) var(--easing-standard);",
                    health.get().status_color(),
                )
            />
        </span>
    }
}
