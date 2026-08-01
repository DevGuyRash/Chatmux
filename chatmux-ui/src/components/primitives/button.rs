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
    #[prop(default = Signal::derive(|| false), into)]
    disabled: Signal<bool>,
    /// Whether the button is in a loading state (shows spinner).
    #[prop(default = Signal::derive(|| false), into)]
    loading: Signal<bool>,
    /// Optional title/tooltip.
    #[prop(optional, into)]
    title: Option<String>,
    /// Optional aria-label for icon-only buttons.
    #[prop(optional, into)]
    aria_label: Option<String>,
    /// Optional pressed state for toggle buttons.
    #[prop(optional, into)]
    aria_pressed: MaybeProp<bool>,
    /// Whether the button should fill its container and left-align content.
    #[prop(default = false)]
    full_width: bool,
    /// Whether the button should render as an active destructive control.
    #[prop(default = Signal::derive(|| false), into)]
    danger_active: Signal<bool>,
    /// Click handler.
    #[prop(optional)]
    on_click: Option<Box<dyn Fn(leptos::ev::MouseEvent) + Send>>,
    /// Button content.
    children: Children,
) -> impl IntoView {
    let rendered_children = children();

    // Presentation lives in components.css as `btn--<variant>` / `btn--<size>`
    // modifiers. Keeping it there is what lets hover, active, focus-visible and
    // disabled states be expressed as real selectors instead of being frozen
    // into an inline style string that no state can override.
    let variant_class = match variant {
        ButtonVariant::Primary => "btn--primary",
        ButtonVariant::Secondary => "btn--secondary",
        ButtonVariant::Danger => "btn--danger",
        ButtonVariant::Ghost => "btn--ghost",
        ButtonVariant::Icon => "btn--icon",
    };

    let size_class = match size {
        ButtonSize::Small => "btn--sm",
        ButtonSize::Medium => "btn--md",
        ButtonSize::Large => "btn--lg",
    };

    let class_list = move || {
        let mut classes = format!("btn select-none {variant_class} {size_class}");
        if full_width {
            classes.push_str(" btn--full");
        }
        if danger_active.get() {
            classes.push_str(" btn--danger-active");
        }
        if loading.get() {
            classes.push_str(" btn--loading");
        }
        classes
    };

    view! {
        <button
            class=class_list
            disabled=move || disabled.get()
            title=title
            aria-label=aria_label
            aria-pressed=move || aria_pressed.get().map(|pressed| if pressed { "true" } else { "false" })
            on:click=move |ev| {
                if !disabled.get_untracked()
                    && !loading.get_untracked()
                    && let Some(ref handler) = on_click
                {
                    handler(ev);
                }
            }
        >
            {move || loading.get().then(|| view! {
                <span class="btn-spinner" aria-hidden="true">"⟳"</span>
            })}
            <span style=move || if loading.get() { "display: none;" } else { "display: contents;" }>
                {rendered_children}
            </span>
        </button>
    }
}
