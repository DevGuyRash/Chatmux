//! Provider status row.
//!
//! Horizontal row of provider status chips/dots, used in the workspace
//! header (§3.2) and run controls bar (§3.7).

use leptos::prelude::*;

use super::{HealthState, Provider, provider_dot::ProviderDot};

/// A provider with its health state signal.
#[derive(Clone)]
pub struct ProviderHealth {
    pub provider: Provider,
    pub health: ReadSignal<HealthState>,
}

/// Horizontal row of provider health dots.
#[component]
pub fn ProviderStatusRow(
    /// Provider health entries.
    providers: Vec<ProviderHealth>,
    /// Whether to show provider icons (full-tab) or just dots (sidebar).
    #[prop(default = false)]
    show_icons: bool,
) -> impl IntoView {
    view! {
        <div class="provider-status-row flex items-center gap-2">
            {providers.into_iter().map(|ph| {
                view! {
                    <ProviderDot
                        provider=ph.provider
                        health=ph.health
                        size=6
                        show_icon=show_icons
                    />
                }
            }).collect_view()}
        </div>
    }
}
