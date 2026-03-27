//! Target selector row (§3.5).
//!
//! Horizontal row of provider toggle chips. Each chip shows
//! provider icon + name. Clicking toggles selection.

use leptos::prelude::*;

use crate::components::provider::Provider;
use crate::components::provider::provider_icon::ProviderIcon;

/// A target with its selection state.
#[derive(Clone)]
pub struct Target {
    pub provider: Provider,
    pub bound: bool,
}

/// Target selector row.
#[component]
pub fn TargetSelector(
    /// Available targets.
    targets: Signal<Vec<Target>>,
    /// Currently selected provider set.
    selected: ReadSignal<Vec<Provider>>,
    /// Toggle a provider's selection.
    on_toggle: impl Fn(Provider) + 'static + Copy + Send,
) -> impl IntoView {
    view! {
        <div class="target-selector flex items-center gap-2 flex-wrap">
            {move || targets.get().into_iter().map(|target| {
                let provider = target.provider;
                let is_selected = selected.get().contains(&provider);
                let is_disabled = !target.bound;

                view! {
                    <button
                        class="type-caption-strong select-none"
                        style=move || format!(
                            "display: inline-flex; align-items: center; gap: var(--space-2); \
                             padding: var(--space-2) var(--space-4); \
                             border-radius: var(--radius-xl); \
                             border: 1px solid {}; \
                             background: {}; \
                             color: {}; \
                             cursor: {}; \
                             opacity: {}; \
                             transition: all var(--duration-fast) var(--easing-standard);",
                            if is_selected { provider.border_color() } else { "var(--border-default)".into() },
                            if is_selected { provider.muted_color() } else { "var(--surface-raised)".into() },
                            if is_disabled { "var(--text-tertiary)".into() } else { provider.text_color() },
                            if is_disabled { "not-allowed" } else { "pointer" },
                            if is_disabled { "0.5" } else { "1" },
                        )
                        disabled=is_disabled
                        on:click=move |_| {
                            if !is_disabled {
                                on_toggle(provider);
                            }
                        }
                    >
                        <ProviderIcon provider=provider size=12 />
                        {provider.label()}
                        {is_selected.then(|| view! {
                            <span style="font-size: 10px; color: var(--text-inverse);">"✓"</span>
                        })}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}
