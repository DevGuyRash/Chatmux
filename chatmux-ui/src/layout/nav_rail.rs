//! Navigation rail for the full-tab layout.
//!
//! A 56px-wide vertical icon strip on the left side.
//! Icons: Workspaces, Active Workspace, Routing, Templates, Diagnostics, Settings.
//! Active item has accent-primary icon color + 3px left indicator bar.

use leptos::prelude::*;

/// Navigation destinations accessible from the nav rail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavDestination {
    Workspaces,
    ActiveWorkspace,
    Routing,
    Templates,
    Diagnostics,
    Settings,
}

impl NavDestination {
    /// Display label for tooltips.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Workspaces => "Workspaces",
            Self::ActiveWorkspace => "Active Workspace",
            Self::Routing => "Routing",
            Self::Templates => "Templates",
            Self::Diagnostics => "Diagnostics",
            Self::Settings => "Settings",
        }
    }

    /// SVG icon path data (simple placeholder icons).
    /// These will be replaced with proper icons from the icon set.
    fn icon_char(&self) -> &'static str {
        match self {
            Self::Workspaces => "⊞",
            Self::ActiveWorkspace => "💬",
            Self::Routing => "⑂",
            Self::Templates => "📄",
            Self::Diagnostics => "🛡",
            Self::Settings => "⚙",
        }
    }
}

const ALL_DESTINATIONS: &[NavDestination] = &[
    NavDestination::Workspaces,
    NavDestination::ActiveWorkspace,
    NavDestination::Routing,
    NavDestination::Templates,
    NavDestination::Diagnostics,
    NavDestination::Settings,
];

/// Nav rail component.
#[component]
pub fn NavRail(
    /// The currently active destination.
    active: ReadSignal<NavDestination>,
    /// Callback when a destination is clicked.
    on_navigate: impl Fn(NavDestination) + 'static + Copy,
) -> impl IntoView {
    view! {
        <nav class="nav-rail flex flex-col items-center py-4 gap-2 select-none"
             style="width: 56px; min-width: 56px; \
                    background: var(--surface-raised); \
                    border-right: 1px solid var(--border-subtle);"
             role="navigation"
             aria-label="Main navigation"
             on:keydown=move |ev| {
                 // Arrow key navigation within nav rail (§8.3)
                 let key = ev.key();
                 if key == "ArrowDown" || key == "ArrowUp" {
                     ev.prevent_default();
                     let current_idx = ALL_DESTINATIONS.iter().position(|&d| d == active.get()).unwrap_or(0);
                     let next_idx = if key == "ArrowDown" {
                         (current_idx + 1) % ALL_DESTINATIONS.len()
                     } else {
                         current_idx.checked_sub(1).unwrap_or(ALL_DESTINATIONS.len() - 1)
                     };
                     on_navigate(ALL_DESTINATIONS[next_idx]);
                 }
             }>
            {ALL_DESTINATIONS.iter().map(|&dest| {
                let is_settings = dest == NavDestination::Settings;
                view! {
                    <div style:margin-top=move || {
                        if is_settings { "auto" } else { "" }
                    }>
                        <NavRailItem
                            destination=dest
                            is_active=Signal::derive(move || active.get() == dest)
                            on_click=move || on_navigate(dest)
                        />
                    </div>
                }
            }).collect_view()}
        </nav>
    }
}

/// Individual nav rail icon button.
#[component]
fn NavRailItem(
    destination: NavDestination,
    is_active: Signal<bool>,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <button
            class="nav-rail-item relative flex items-center justify-center cursor-pointer transition-colors"
            class:nav-rail-item--active=move || is_active.get()
            style="width: 44px; height: 44px; border-radius: var(--radius-md);"
            title=destination.label()
            aria-label=destination.label()
            aria-current=move || if is_active.get() { Some("page") } else { None }
            on:click=move |_| on_click()
        >
            // Active indicator bar (3px left)
            {move || is_active.get().then(|| view! {
                <span
                    class="absolute"
                    style="left: -6px; top: 8px; bottom: 8px; width: 3px; \
                           background: var(--accent-primary); \
                           border-radius: var(--radius-full);"
                />
            })}

            // Icon placeholder (will be SVG components later)
            <span
                style=move || format!(
                    "font-size: 18px; color: {};",
                    if is_active.get() { "var(--accent-primary)" } else { "var(--text-secondary)" }
                )
            >
                {destination.icon_char()}
            </span>
        </button>
    }
}
