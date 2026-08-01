//! Provider binding card (§3.8).
//!
//! Card per provider showing: icon, name, health badge, tab info,
//! last activity, and action buttons.

use leptos::prelude::*;

use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::provider::health_badge::HealthBadge;
use crate::components::provider::provider_icon::ProviderIcon;
use crate::components::provider::{HealthState, Provider};

/// Provider binding card.
#[component]
pub fn BindingCard(
    /// Provider.
    provider: Provider,
    /// Health state.
    health: Signal<HealthState>,
    /// Whether a concrete browser tab is currently bound.
    bound: Signal<bool>,
    /// Whether the provider host permission still needs to be granted.
    permission_missing: Signal<bool>,
    /// Tab info string (e.g., "Tab #42 — chat.openai.com").
    tab_info: Signal<Option<String>>,
    /// Last activity string.
    last_activity: Signal<Option<String>>,
    /// Called to detect candidate tabs for binding.
    on_detect: impl Fn() + 'static + Send,
    /// Called to request provider host permission.
    on_grant_permission: impl Fn() + 'static + Send,
    /// Called to open provider tab.
    on_open_tab: impl Fn() + 'static + Send,
) -> impl IntoView {
    let card_bg_tint = Signal::derive(move || match health.get() {
        HealthState::DomMismatch | HealthState::Blocked | HealthState::SendFailed => {
            "background: var(--status-error-muted);"
        }
        HealthState::PermissionMissing
        | HealthState::LoginRequired
        | HealthState::RateLimited
        | HealthState::CaptureUncertain
        | HealthState::DegradedManualOnly => "background: var(--status-warning-muted);",
        _ => "",
    });

    view! {
        <div
            class="binding-card surface-raised rounded-md"
            style=move || format!(
                "border: 1px solid var(--border-default); \
                 {} \
                 padding: var(--space-5); {}",
                provider.channel_vars(),
                card_bg_tint.get(),
            )
        >
            // Row 1: Provider + health badge
            <div class="flex items-center justify-between mb-3">
                <div class="flex items-center gap-3">
                    <ProviderIcon provider=provider size=20 />
                    <span class="type-subtitle text-primary">{provider.label()}</span>
                </div>
                <HealthBadge health=health />
            </div>

            // Row 2: Tab info
            <p class="type-caption" style=move || format!(
                "margin-bottom: var(--space-2); color: {};",
                if tab_info.get().is_some() { "var(--text-secondary)" } else { "var(--text-tertiary)" },
            )>
                {move || tab_info.get().unwrap_or_else(|| "No tab bound".to_string())}
            </p>

            // Row 3: Last activity
            {move || last_activity.get().map(|activity| view! {
                <p class="type-caption text-tertiary mb-3">
                    {activity}
                </p>
            })}

            // Actions
            <div class="flex items-center gap-2">
                <span style=move || if permission_missing.get() { "display: inline-flex;" } else { "display: none;" }>
                    <Button
                        variant=ButtonVariant::Primary
                        size=ButtonSize::Small
                        on_click=Box::new(move |_| on_grant_permission())
                    >
                        "Grant access"
                    </Button>
                </span>
                <span style=move || if permission_missing.get() { "display: none;" } else { "display: inline-flex;" }>
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Small
                        on_click=Box::new(move |_| on_detect())
                    >
                        {move || if bound.get() { "Detect / Rebind" } else { "Detect / Bind" }}
                    </Button>
                </span>
                <Button variant=ButtonVariant::Secondary size=ButtonSize::Small
                        on_click=Box::new(move |_| on_open_tab())>
                    "Open tab"
                </Button>
            </div>
        </div>
    }
}
