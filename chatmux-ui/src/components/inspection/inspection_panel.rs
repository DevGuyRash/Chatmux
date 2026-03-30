//! Message inspection panel (§3.4).
//!
//! Opens when a message card is clicked.
//! Sidebar: full-width overlay sliding from right.
//! Full-tab: right-side panel (35–45% width).
//! Four tabs: Sent Payload, Raw Response, Network, Metadata.

use leptos::prelude::*;

use super::metadata_tab::MetadataTab;
use super::network_capture_tab::NetworkCaptureTab;
use super::raw_response_tab::RawResponseTab;
use super::sent_payload_tab::SentPayloadTab;
use crate::components::provider::Provider;
use crate::components::provider::provider_icon::ProviderIcon;
use crate::models::{Message, ProviderNetworkCapture};
use crate::time::format_local_datetime;

/// Active tab in the inspection panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InspectionTab {
    SentPayload,
    RawResponse,
    Network,
    Metadata,
}

/// Message inspection panel.
#[component]
pub fn InspectionPanel(
    /// The message being inspected.
    message: Message,
    /// The rendered payload sent to the provider, if available.
    sent_payload: Option<String>,
    /// The raw captured response text, if available.
    raw_response: Option<String>,
    /// Captured network request/response details, if available.
    network_capture: Option<ProviderNetworkCapture>,
    /// Called to close the panel.
    on_close: impl Fn() + 'static + Copy + Send,
    /// Whether to show the internal header (title + close button).
    /// Set to false when rendered inside a side panel that provides its own header.
    #[prop(default = true)]
    show_header: bool,
) -> impl IntoView {
    let (active_tab, set_active_tab) = signal(InspectionTab::SentPayload);
    let provider = Provider::from_provider_id(message.participant_id);
    let timestamp = format_local_datetime(message.timestamp);
    let msg_for_meta = message.clone();
    let sent_payload_value = sent_payload.clone();
    let raw_response_value = raw_response.clone();
    let network_capture_value = network_capture.clone();

    view! {
        <div class="inspection-panel flex flex-col h-full surface-raised">
            // Header (hidden when side panel provides its own)
            {show_header.then(|| view! {
                <div class="flex flex-col gap-2 p-5 border-b">
                    <div class="flex items-center justify-between">
                        <span class="type-title text-primary">"Message Inspection"</span>
                        <button
                            class="cursor-pointer text-secondary"
                            style="font-size: 16px;"
                            aria-label="Close inspection panel"
                            on:click=move |_| on_close()
                        >
                            "✕"
                        </button>
                    </div>
                    <div class="flex items-center gap-2">
                        <ProviderIcon provider=provider size=14 />
                        <span class="type-caption-strong" style=format!("color: {};", provider.text_color())>
                            {provider.label()}
                        </span>
                        <span class="type-caption text-secondary">{timestamp}</span>
                    </div>
                </div>
            })}
            // Provider metadata (always visible, even without header)
            {(!show_header).then(|| view! {
                <div class="flex items-center gap-2 p-5 border-b">
                    <ProviderIcon provider=provider size=14 />
                    <span class="type-caption-strong" style=format!("color: {};", provider.text_color())>
                        {provider.label()}
                    </span>
                    <span class="type-caption text-secondary">{timestamp}</span>
                </div>
            })}

            // Tab bar
            <div class="flex items-center gap-0 border-b">
                <TabButton
                    label="Sent Payload"
                    active=Signal::derive(move || active_tab.get() == InspectionTab::SentPayload)
                    on_click=move || set_active_tab.set(InspectionTab::SentPayload)
                />
                <TabButton
                    label="Raw Response"
                    active=Signal::derive(move || active_tab.get() == InspectionTab::RawResponse)
                    on_click=move || set_active_tab.set(InspectionTab::RawResponse)
                />
                <TabButton
                    label="Network"
                    active=Signal::derive(move || active_tab.get() == InspectionTab::Network)
                    on_click=move || set_active_tab.set(InspectionTab::Network)
                />
                <TabButton
                    label="Metadata"
                    active=Signal::derive(move || active_tab.get() == InspectionTab::Metadata)
                    on_click=move || set_active_tab.set(InspectionTab::Metadata)
                />
            </div>

            // Tab content
            <div class="flex-1 overflow-y-auto p-5">
                {move || match active_tab.get() {
                    InspectionTab::SentPayload => view! {
                        <SentPayloadTab payload=Signal::derive({
                            let sent_payload_value = sent_payload_value.clone();
                            move || sent_payload_value.clone()
                        }) />
                    }.into_any(),
                    InspectionTab::RawResponse => view! {
                        <RawResponseTab response=Signal::derive({
                            let raw_response_value = raw_response_value.clone();
                            move || raw_response_value.clone()
                        }) />
                    }.into_any(),
                    InspectionTab::Network => view! {
                        <NetworkCaptureTab capture=Signal::derive({
                            let network_capture_value = network_capture_value.clone();
                            move || network_capture_value.clone()
                        }) />
                    }.into_any(),
                    InspectionTab::Metadata => view! {
                        <MetadataTab message=msg_for_meta.clone() />
                    }.into_any(),
                }}
            </div>
        </div>
    }
}

/// Tab button in the tab bar.
#[component]
fn TabButton(
    label: &'static str,
    active: Signal<bool>,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <button
            class="type-caption-strong cursor-pointer select-none"
            role="tab"
            aria-selected=move || if active.get() { "true" } else { "false" }
            style=move || format!(
                "padding: var(--space-4) var(--space-5); \
                 background: none; border: none; \
                 color: {}; \
                 border-bottom: 2px solid {}; \
                 transition: all var(--duration-fast) var(--easing-standard);",
                if active.get() { "var(--text-primary)" } else { "var(--text-secondary)" },
                if active.get() { "var(--accent-primary)" } else { "transparent" },
            )
            on:click=move |_| on_click()
        >
            {label}
        </button>
    }
}
