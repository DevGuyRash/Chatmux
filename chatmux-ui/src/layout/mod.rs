//! Layout system.
//!
//! Provides the sidebar (~360px) and full-tab (~1200px+) layout shells,
//! responsive layout mode detection, and shared layout components
//! (nav rail, global header, collapsible side panel).

pub mod full_tab;
pub mod global_header;
pub mod nav_rail;
pub mod responsive;
pub mod screens;
pub mod side_panel;
pub mod sidebar;

use leptos::prelude::*;
use responsive::LayoutMode;

/// The top-level layout shell. Renders either sidebar or full-tab layout
/// based on the detected layout mode.
#[component]
pub fn LayoutShell(layout_mode: ReadSignal<LayoutMode>) -> impl IntoView {
    view! {
        <div class="layout-shell w-full h-full">
            {move || match layout_mode.get() {
                LayoutMode::Sidebar => view! { <sidebar::SidebarLayout /> }.into_any(),
                LayoutMode::FullTab => view! { <full_tab::FullTabLayout /> }.into_any(),
            }}
        </div>
    }
}
