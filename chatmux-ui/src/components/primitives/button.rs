//! Button component.
//!
//! Variants: Primary, Secondary, Danger, Ghost, Icon-only.
//! All follow the design system tokens for colors, radii, and transitions.

use leptos::prelude::*;

/// Button visual variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Filled accent-primary background, text-inverse text.
    #[default]
    Primary,
    /// surface-sunken fill, text-primary, border-default.
    Secondary,
    /// status-error-solid fill, text-inverse.
    Danger,
    /// Transparent background, text-link color.
    Ghost,
    /// Square icon-only button.
    Icon,
}

/// Button size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// Reusable button component implementing the design system button spec.
#[component]
pub fn Button(
    /// Button variant.
    #[prop(default = ButtonVariant::Primary)]
    variant: ButtonVariant,
    /// Button size.
    #[prop(default = ButtonSize::Medium)]
    size: ButtonSize,
    /// Whether the button is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Whether the button is in a loading state (shows spinner).
    #[prop(default = false)]
    loading: bool,
    /// Optional title/tooltip.
    #[prop(optional, into)]
    title: Option<String>,
    /// Optional aria-label for icon-only buttons.
    #[prop(optional, into)]
    aria_label: Option<String>,
    /// Click handler.
    #[prop(optional)]
    on_click: Option<Box<dyn Fn(leptos::ev::MouseEvent) + Send>>,
    /// Button content.
    children: Children,
) -> impl IntoView {
    let base_style = match variant {
        ButtonVariant::Primary => {
            "\
            background: var(--accent-primary); \
            color: var(--text-inverse); \
            border: none;"
        }
        ButtonVariant::Secondary => {
            "\
            background: var(--surface-sunken); \
            color: var(--text-primary); \
            border: 1px solid var(--border-default);"
        }
        ButtonVariant::Danger => {
            "\
            background: var(--status-error-solid); \
            color: var(--text-inverse); \
            border: none;"
        }
        ButtonVariant::Ghost => {
            "\
            background: transparent; \
            color: var(--text-link); \
            border: none;"
        }
        ButtonVariant::Icon => {
            "\
            background: transparent; \
            color: var(--text-secondary); \
            border: none; \
            padding: 0;"
        }
    };

    let size_style = match size {
        ButtonSize::Small => {
            "\
            padding: var(--space-1) var(--space-3); \
            font-size: var(--type-caption-size); \
            min-height: 24px;"
        }
        ButtonSize::Medium => {
            "\
            padding: var(--space-3) var(--space-5); \
            font-size: var(--type-body-size); \
            min-height: 32px;"
        }
        ButtonSize::Large => {
            "\
            padding: var(--space-4) var(--space-6); \
            font-size: var(--type-body-size); \
            min-height: 40px;"
        }
    };

    let icon_size_style = if variant == ButtonVariant::Icon {
        match size {
            ButtonSize::Small => "width: 24px; height: 24px;",
            ButtonSize::Medium => "width: 32px; height: 32px;",
            ButtonSize::Large => "width: 40px; height: 40px;",
        }
    } else {
        ""
    };

    let combined_style = format!(
        "{base_style} {size_style} {icon_size_style} \
         border-radius: var(--radius-md); \
         cursor: {}; \
         display: inline-flex; align-items: center; justify-content: center; \
         gap: var(--space-2); \
         font-weight: var(--type-label-weight); \
         letter-spacing: var(--type-label-tracking); \
         transition: background var(--duration-fast) var(--easing-standard), \
                     color var(--duration-fast) var(--easing-standard), \
                     box-shadow var(--duration-fast) var(--easing-standard); \
         opacity: {};",
        if disabled { "not-allowed" } else { "pointer" },
        if disabled { "0.5" } else { "1" },
    );

    view! {
        <button
            class="btn select-none"
            style=combined_style
            disabled=disabled
            title=title
            aria-label=aria_label
            on:click=move |ev| {
                if !disabled && !loading {
                    if let Some(ref handler) = on_click {
                        handler(ev);
                    }
                }
            }
        >
            {if loading {
                view! { <span class="btn-spinner" aria-hidden="true">"⟳"</span> }.into_any()
            } else {
                view! { <>{children()}</> }.into_any()
            }}
        </button>
    }
}
