//! Root application component.
//!
//! Sets up the theme provider, layout mode detection, and all global state
//! context providers. Routes to either the sidebar or full-tab layout
//! based on the detected surface.

use leptos::prelude::*;

use crate::components::toast_container::ToastContainer;
use crate::layout::LayoutShell;
use crate::state::{
    app_state, binding_state, diagnostics_state, message_state,
    run_state, search_state, selection_state, workspace_state,
};
use crate::theme::ThemeProvider;

/// Root application component. Wraps everything in theme and layout providers.
#[component]
pub fn App() -> impl IntoView {
    view! {
        <ThemeProvider>
            <AppInner />
        </ThemeProvider>
    }
}

/// Inner app component that has access to theme context.
#[component]
fn AppInner() -> impl IntoView {
    // Detect layout mode from container width
    let layout_mode = crate::layout::responsive::use_layout_mode();

    // Provide layout mode to all descendants
    provide_context(layout_mode);

    // Provide reduced motion signal to all descendants
    let reduced_motion = crate::a11y::use_reduced_motion();
    provide_context(reduced_motion);

    // Initialize all domain state providers
    app_state::provide_app_state();
    workspace_state::provide_workspace_state();
    run_state::provide_run_state();
    binding_state::provide_binding_state();
    message_state::provide_message_state();
    selection_state::provide_selection_state();
    search_state::provide_search_state();
    diagnostics_state::provide_diagnostics_state();

    // TODO(backend): Subscribe to workspace list changes from background coordinator.
    // TODO(backend): Subscribe to provider health changes.
    // TODO(backend): Subscribe to diagnostic events.
    // TODO(backend): Load saved settings from storage.local on startup.

    view! {
        <LayoutShell layout_mode=layout_mode />
        <ToastContainer />
    }
}
