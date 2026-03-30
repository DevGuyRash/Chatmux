//! Responsive layout mode detection.
//!
//! Determines whether the UI should render in sidebar (~360px) or
//! full-tab (~1200px+) mode using a ResizeObserver on the root element.
//! The threshold is 500px — below that is sidebar, above is full-tab.

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::theme::tokens::breakpoints::SIDEBAR_MAX_WIDTH;

/// The two layout modes the UI supports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutMode {
    /// Sidebar surface (~360px). Stack-based navigation, single column.
    Sidebar,
    /// Full tab surface (~1200px+). Nav rail + multi-panel workspace.
    FullTab,
}

/// Creates a reactive signal tracking the current layout mode.
///
/// Uses a `ResizeObserver` on the document body to detect width changes.
/// Returns `LayoutMode::Sidebar` when width < 500px, `FullTab` otherwise.
pub fn use_layout_mode() -> ReadSignal<LayoutMode> {
    let initial_mode = detect_layout_mode();
    let (mode, set_mode) = signal(initial_mode);

    // Set up ResizeObserver on mount
    Effect::new(move |_| {
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");

        if let Some(body) = document.body() {
            let cb = Closure::wrap(Box::new(move |entries: js_sys::Array, _observer: JsValue| {
                if let Some(entry) = entries.get(0).dyn_ref::<web_sys::ResizeObserverEntry>() {
                    let rect = entry.content_rect();
                    let width = rect.width();
                    let new_mode = if width < SIDEBAR_MAX_WIDTH {
                        LayoutMode::Sidebar
                    } else {
                        LayoutMode::FullTab
                    };
                    set_mode.set(new_mode);
                }
            }) as Box<dyn Fn(js_sys::Array, JsValue)>);

            let observer = web_sys::ResizeObserver::new(cb.as_ref().unchecked_ref())
                .expect("Failed to create ResizeObserver");

            observer.observe(&body);

            // Leak the closure and observer — they live for the app lifetime
            cb.forget();
            std::mem::forget(observer);
        }
    });

    mode
}

/// One-time detection of the initial layout mode based on window width.
fn detect_layout_mode() -> LayoutMode {
    let window = web_sys::window().expect("no window");
    let width = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(360.0);

    if width < SIDEBAR_MAX_WIDTH {
        LayoutMode::Sidebar
    } else {
        LayoutMode::FullTab
    }
}
