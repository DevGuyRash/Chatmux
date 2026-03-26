//! Collapsible and resizable side panel for the full-tab layout.
//!
//! Opens when a message is clicked (inspection), or when the user
//! opens bindings, cursors, or templates from the nav rail.
//! Min width: 300px, max width: 600px. Resizable via drag handle.
//!
//! Transition: content fades out at duration-fast, then width animates
//! to 0 over duration-normal with easing-exit (and reverse for open).

// Side panel rendering is currently inlined in full_tab.rs.
// This module will contain the reusable SidePanel component with
// resize handle and collapse/expand animation once Phase 1 primitives
// are available.

use leptos::prelude::*;

/// Minimum width of the side panel in pixels.
pub const SIDE_PANEL_MIN_WIDTH: f64 = 300.0;
/// Maximum width of the side panel in pixels.
pub const SIDE_PANEL_MAX_WIDTH: f64 = 600.0;
/// Default width of the side panel in pixels.
pub const SIDE_PANEL_DEFAULT_WIDTH: f64 = 360.0;

/// Side panel component with resize handle.
#[component]
pub fn SidePanel(
    /// Whether the panel is visible.
    open: ReadSignal<bool>,
    /// Panel content.
    children: Children,
) -> impl IntoView {
    let (width, _set_width) = signal(SIDE_PANEL_DEFAULT_WIDTH);

    view! {
        <div
            class="side-panel surface-raised"
            class:side-panel--open=move || open.get()
            style=move || format!(
                "width: {}px; min-width: {}px; max-width: {}px; \
                 border-left: 1px solid var(--border-subtle); \
                 transition: width var(--duration-normal) var(--easing-standard); \
                 overflow: hidden;",
                if open.get() { width.get() } else { 0.0 },
                if open.get() { SIDE_PANEL_MIN_WIDTH } else { 0.0 },
                SIDE_PANEL_MAX_WIDTH,
            )
        >
            {children()}
        </div>
    }
}
