//! Exact per-target package preview and editor (§3.6).

use leptos::prelude::*;
use std::collections::BTreeSet;

use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::icon::{Icon, IconKind};
use crate::components::primitives::text_area::TextArea;
use crate::components::provider::Provider;
use crate::models::{MessageId, NextRoundPackage, ProviderId};

/// Editable previews of the exact backend-rendered packages for every selected target.
#[component]
pub fn PackagePreview(
    /// Exact rendered packages returned by the background coordinator.
    packages: ReadSignal<Vec<NextRoundPackage>>,
    /// Whether the preview request is still running.
    loading: ReadSignal<bool>,
    /// Actionable preview error, when rendering failed.
    error: ReadSignal<Option<String>>,
    /// Whether draft, target, note, or context changes made this preview stale.
    stale: ReadSignal<bool>,
    /// Targets whose rendered payload was manually edited.
    edited_targets: ReadSignal<BTreeSet<ProviderId>>,
    /// Called when one target's exact payload is edited.
    on_edit: impl Fn(ProviderId, String) + 'static + Send + Sync,
    /// Remove one source context block and render a new exact package.
    on_remove_source: impl Fn(MessageId) + 'static + Copy + Send,
    /// Called to render a fresh preview from current composer state.
    on_refresh: impl Fn() + 'static + Copy + Send,
    /// Called to close the preview.
    on_close: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    let (active_target, set_active_target) = signal(None::<ProviderId>);
    let (copy_result, set_copy_result) = signal(None::<bool>);
    let on_edit = StoredValue::new(on_edit);

    Effect::new(move |_| {
        let current = active_target.get_untracked();
        let rendered = packages.get();
        if current.is_none_or(|target| {
            !rendered
                .iter()
                .any(|package| package.target_participant_id == target)
        }) {
            set_active_target.set(
                rendered
                    .first()
                    .map(|package| package.target_participant_id),
            );
        }
    });

    let active_package = Signal::derive(move || {
        let target = active_target.get()?;
        packages
            .get()
            .into_iter()
            .find(|package| package.target_participant_id == target)
    });

    view! {
        <div
            class="package-preview surface-card flex flex-col overflow-hidden"
            style="max-height: 50%;"
        >
            <div class="flex items-center justify-between px-4 py-3 border-b">
                <div class="flex flex-col gap-1">
                    <span class="type-subtitle text-primary">"Outbound package"</span>
                    <span class="type-caption text-secondary">
                        "Exact text that will be committed for the selected provider."
                    </span>
                </div>
                <Button
                    variant=ButtonVariant::Icon
                    size=ButtonSize::Small
                    title="Close preview".to_owned()
                    aria_label="Close preview".to_owned()
                    on_click=Box::new(move |_| on_close())
                >
                    <Icon kind=IconKind::Close size=14 />
                </Button>
            </div>

            {move || loading.get().then(|| view! {
                <div class="p-4 type-caption text-secondary" role="status">
                    "Rendering exact packages…"
                </div>
            })}

            {move || error.get().map(|detail| view! {
                <div class="p-4 type-body text-error" role="alert">{detail}</div>
            })}

            {move || stale.get().then(|| view! {
                <div class="flex items-center justify-between gap-3 p-4 border-b"
                     style="background: var(--status-warning-muted);">
                    <span class="type-caption text-primary">
                        "Composer inputs changed. Refresh before sending this preview."
                    </span>
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Small
                        on_click=Box::new(move |_| on_refresh())
                    >
                        "Refresh"
                    </Button>
                </div>
            })}

            {move || (!packages.get().is_empty()).then(|| view! {
                <div class="flex items-center gap-2 flex-wrap px-4 py-3 border-b" role="tablist">
                    {packages.get().into_iter().map(|package| {
                        let target = package.target_participant_id;
                        let provider = Provider::from_provider_id(target);
                        view! {
                            <Button
                                variant=if active_target.get() == Some(target) {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Secondary
                                }
                                size=ButtonSize::Small
                                aria_label=format!("Preview package for {}", provider.label())
                                on_click=Box::new(move |_| set_active_target.set(Some(target)))
                            >
                                {provider.label()}
                            </Button>
                        }
                    }).collect_view()}
                </div>
            })}

            {move || active_package.get().map(|package| {
                let target = package.target_participant_id;
                view! {
                    <div class="flex flex-col min-h-0">
                        {(!package.source_blocks.is_empty()).then(|| view! {
                            <div class="flex flex-col gap-2 px-4 py-3 border-b">
                                <div class="flex items-center justify-between gap-3">
                                    <span class="type-caption-strong text-secondary">"Context blocks"</span>
                                    <span class="type-caption text-tertiary">
                                        {format!("{} included", package.source_blocks.len())}
                                    </span>
                                </div>
                                {package.source_blocks.into_iter().map(|block| {
                                    let message_id = block.message_id;
                                    view! {
                                        <div class="surface-sunken border rounded-md p-3 flex items-start gap-3">
                                            <div class="flex-1 min-w-0">
                                                <span class="type-caption-strong text-primary">
                                                    {format!("{} · {:?}", block.participant_id.display_name(), block.role)}
                                                </span>
                                                <p class="type-caption text-secondary truncate mt-1">{block.preview}</p>
                                            </div>
                                            <Button
                                                variant=ButtonVariant::Icon
                                                size=ButtonSize::Small
                                                title="Remove context block".to_owned()
                                                aria_label=format!("Remove context block {}", message_id.0)
                                                on_click=Box::new(move |_| on_remove_source(message_id))
                                            >
                                                <Icon kind=IconKind::Close size=12 />
                                            </Button>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        })}
                        <TextArea
                            value=Signal::derive(move || {
                                active_package
                                    .get()
                                    .map(|item| item.rendered_payload)
                                    .unwrap_or_default()
                            })
                            monospace=true
                            min_rows=5
                            max_rows=14
                            aria_label=format!(
                                "Exact outbound package for {}",
                                target.display_name(),
                            )
                            on_input=move |value| {
                                on_edit.with_value(|handler| handler(target, value));
                            }
                        />
                        <div class="flex items-center justify-between gap-3 px-4 py-2 border-t">
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Small
                                on_click=Box::new(move |_| {
                                    if let Some(package) = active_package.get_untracked() {
                                        leptos::task::spawn_local(async move {
                                            set_copy_result.set(Some(
                                                crate::bridge::clipboard::write_clipboard(
                                                    &package.rendered_payload,
                                                )
                                                .await,
                                            ));
                                        });
                                    }
                                })
                            >
                                "Copy to Clipboard"
                            </Button>
                            <div class="flex items-center gap-3">
                                {move || edited_targets.get().contains(&target).then(|| view! {
                                    <span class="type-caption text-primary"
                                          style="color: var(--status-warning-text);">
                                        "Edited"
                                    </span>
                                })}
                                {move || copy_result.get().map(|copied| view! {
                                    <span
                                        class=if copied {
                                            "type-caption text-success"
                                        } else {
                                            "type-caption text-error"
                                        }
                                        role=if copied { "status" } else { "alert" }
                                    >
                                        {if copied { "Copied" } else { "Copy failed" }}
                                    </span>
                                })}
                                <span class="type-caption text-secondary">
                                    {move || active_package.get().map(|item| {
                                        format!("{} chars", item.rendered_payload.chars().count())
                                    }).unwrap_or_default()}
                                </span>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
