//! Theme management.
//!
//! Provides a ThemeProvider component that manages dark/light theme state
//! and sets the `data-theme` attribute on the root HTML element.
//! Supports "dark", "light", and "system" preferences.

pub mod tokens;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

/// The available theme settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

/// The user's theme preference (may defer to system).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemePreference {
    Dark,
    Light,
    System,
}

/// Context type for theme state.
#[derive(Clone, Copy)]
pub struct ThemeContext {
    /// The resolved theme currently in effect.
    pub active: ReadSignal<Theme>,
    /// The user's preference setting.
    pub preference: ReadSignal<ThemePreference>,
    /// Set the user's theme preference.
    pub set_preference: WriteSignal<ThemePreference>,
}

/// Detect the system's color scheme preference.
fn detect_system_theme() -> Theme {
    let window = web_sys::window().expect("no window");
    let query = window
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten();

    match query {
        Some(mql) if mql.matches() => Theme::Dark,
        _ => Theme::Light,
    }
}

/// Resolve a ThemePreference to a concrete Theme.
fn resolve_theme(pref: ThemePreference, system_theme: Theme) -> Theme {
    match pref {
        ThemePreference::Dark => Theme::Dark,
        ThemePreference::Light => Theme::Light,
        ThemePreference::System => system_theme,
    }
}

/// Apply the theme by setting `data-theme` on `<html>`.
fn apply_theme(theme: Theme) {
    let document = web_sys::window()
        .expect("no window")
        .document()
        .expect("no document");

    if let Some(root) = document.document_element() {
        let value = match theme {
            Theme::Dark => "dark",
            Theme::Light => "light",
        };
        let _ = root.set_attribute("data-theme", value);
    }
}

/// Hook to access the current theme context from any component.
pub fn use_theme() -> ThemeContext {
    expect_context::<ThemeContext>()
}

/// Theme provider component. Wrap the app root in this.
#[component]
pub fn ThemeProvider(children: Children) -> impl IntoView {
    let (preference, set_preference) = signal(ThemePreference::Dark);
    let (system_theme, set_system_theme) = signal(detect_system_theme());
    let (settings_loaded, set_settings_loaded) = signal(false);

    let active = Memo::new(move |_| resolve_theme(preference.get(), system_theme.get()));

    Effect::new(move |_| {
        if settings_loaded.get() {
            return;
        }
        set_settings_loaded.set(true);
        leptos::task::spawn_local(async move {
            if let Some(settings) = crate::bridge::storage::load_settings().await {
                set_preference.set(settings.theme);
            }
        });
    });

    // Apply theme to DOM whenever it changes
    Effect::new(move |_| {
        apply_theme(active.get());
    });

    // Keep a live resolved system theme so System reacts when the OS preference changes.
    Effect::new(move |_| {
        let window = web_sys::window().expect("no window");
        if let Ok(Some(mql)) = window.match_media("(prefers-color-scheme: dark)") {
            let closure = Closure::wrap(Box::new(move |event: web_sys::MediaQueryListEvent| {
                set_system_theme.set(if event.matches() {
                    Theme::Dark
                } else {
                    Theme::Light
                });
            }) as Box<dyn Fn(_)>);

            let _ =
                mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    });

    provide_context(ThemeContext {
        active: {
            // Create a derived signal from the memo
            let (sig, set_sig) = signal(active.get_untracked());
            Effect::new(move |_| {
                set_sig.set(active.get());
            });
            sig
        },
        preference,
        set_preference,
    });

    view! {
        {children()}
    }
}
