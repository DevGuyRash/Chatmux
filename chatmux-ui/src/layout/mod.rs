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
use nav_rail::NavDestination;
use responsive::LayoutMode;

/// The top-level layout shell. Renders either sidebar or full-tab layout
/// based on the detected layout mode.
#[component]
pub fn LayoutShell(layout_mode: ReadSignal<LayoutMode>) -> impl IntoView {
    // Keep the user's current destination above the responsive layout switch.
    let (active_nav, set_active_nav) = signal(NavDestination::Workspaces);

    view! {
        <div class="layout-shell w-full h-full">
            {move || match layout_mode.get() {
                LayoutMode::Sidebar => view! {
                    <sidebar::SidebarLayout active_nav=active_nav set_active_nav=set_active_nav />
                }.into_any(),
                LayoutMode::FullTab => view! {
                    <full_tab::FullTabLayout active_nav=active_nav set_active_nav=set_active_nav />
                }.into_any(),
            }}
        </div>
    }
}
