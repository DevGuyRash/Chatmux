//! Code block component — pre-formatted text with consistent styling.
//!
//! Replaces inline `white-space: pre-wrap; word-break: break-word` patterns.

use leptos::prelude::*;

/// Pre-formatted code block with surface background.
#[component]
pub fn CodeBlock(
    /// Additional CSS classes to compose.
    #[prop(optional, into)]
    class: Option<String>,
    /// Code content.
    children: Children,
) -> impl IntoView {
    let full_class = match &class {
        Some(extra) => format!("code-block surface-card type-code-small {extra}"),
        None => "code-block surface-card type-code-small".to_owned(),
    };
    view! {
        <pre class=full_class>
            {children()}
        </pre>
    }
}
