//! Raw response tab (§3.4).
//!
//! Shows the raw text as extracted from the provider DOM,
//! before normalization. font-mono, type-code, surface-sunken.

use leptos::prelude::*;

/// Raw response tab content.
#[component]
pub fn RawResponseTab(
    /// The raw response text, if available.
    response: Signal<Option<String>>,
) -> impl IntoView {
    view! {
        {move || match response.get() {
            Some(text) => view! {
                <pre class="type-code surface-sunken code-block">
                    {text}
                </pre>
            }.into_any(),
            None => view! {
                <p class="type-body text-secondary text-center p-7">
                    "Raw response not available."
                </p>
            }.into_any(),
        }}
    }
}
