//! Network capture tab for message inspection.

use leptos::prelude::*;

use crate::models::ProviderNetworkCapture;

#[component]
pub fn NetworkCaptureTab(
    capture: Signal<Option<ProviderNetworkCapture>>,
) -> impl IntoView {
    view! {
        {move || match capture.get() {
            Some(capture) => {
                let request_line = match (capture.request_method.clone(), capture.request_url.clone()) {
                    (Some(method), Some(url)) => format!("{method} {url}"),
                    (Some(method), None) => method,
                    (None, Some(url)) => url,
                    (None, None) => "Request metadata unavailable".to_owned(),
                };
                let response_line = match capture.response_status {
                    Some(status) => format!("Response status: {status}"),
                    None => "Response status unavailable".to_owned(),
                };
                view! {
                    <div class="flex flex-col gap-4">
                        <div class="surface-sunken rounded-md" style="padding: var(--space-4);">
                            <p class="type-caption-strong text-primary">{request_line}</p>
                            <p class="type-caption text-secondary">
                                {capture.capture_strategy.unwrap_or_else(|| "dom_fallback".to_owned())}
                            </p>
                            <p class="type-caption text-secondary">{response_line}</p>
                        </div>
                        <section class="flex flex-col gap-2">
                            <span class="type-caption-strong text-primary">"Request Body"</span>
                            <pre
                                class="type-code surface-sunken"
                                style="padding: var(--space-4); border-radius: var(--radius-md); overflow-x: auto; white-space: pre-wrap; word-break: break-word;"
                            >
                                {capture.request_body.unwrap_or_else(|| "Request body unavailable.".to_owned())}
                            </pre>
                        </section>
                        <section class="flex flex-col gap-2">
                            <span class="type-caption-strong text-primary">"Response Body"</span>
                            <pre
                                class="type-code surface-sunken"
                                style="padding: var(--space-4); border-radius: var(--radius-md); overflow-x: auto; white-space: pre-wrap; word-break: break-word;"
                            >
                                {capture.response_body.unwrap_or_else(|| "Response body unavailable.".to_owned())}
                            </pre>
                        </section>
                    </div>
                }.into_any()
            }
            None => view! {
                <p class="type-body text-secondary" style="text-align: center; padding: var(--space-7);">
                    "Network capture not available for this message."
                </p>
            }.into_any(),
        }}
    }
}
