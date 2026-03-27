//! Root application component.
//!
//! Sets up the theme provider, layout mode detection, and all global state
//! context providers. Routes to either the sidebar or full-tab layout
//! based on the detected surface.

use leptos::prelude::*;

use crate::bridge::{messaging, storage};
use crate::components::toast_container::ToastContainer;
use crate::layout::LayoutShell;
use crate::state::{
    app_state, binding_state, diagnostics_state, message_state, run_state, search_state,
    selection_state, workspace_state,
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
    let app_state = app_state::provide_app_state();
    let workspace_state = workspace_state::provide_workspace_state();
    let run_state = run_state::provide_run_state();
    let binding_state = binding_state::provide_binding_state();
    let message_state = message_state::provide_message_state();
    selection_state::provide_selection_state();
    search_state::provide_search_state();
    let diagnostics_state = diagnostics_state::provide_diagnostics_state();

    let (initialized, set_initialized) = signal(false);
    Effect::new(move |_| {
        if initialized.get() {
            return;
        }

        set_initialized.set(true);

        let app_for_listener = app_state;
        let workspace_for_listener = workspace_state;
        let run_for_listener = run_state;
        let binding_for_listener = binding_state;
        let message_for_listener = message_state;
        let diagnostics_for_listener = diagnostics_state;
        messaging::listen_for_events(move |event| {
            crate::state::controller::apply_events(
                app_for_listener,
                workspace_for_listener,
                run_for_listener,
                binding_for_listener,
                message_for_listener,
                diagnostics_for_listener,
                vec![event],
            );
        });

        leptos::task::spawn_local(async move {
            if let Some(settings) = storage::load_settings().await {
                app_state.set_kill_switch_active.set(settings.kill_switch_active);
                app_state.set_ui_settings.set(settings);
            }

            crate::state::controller::dispatch_command_result(
                app_state,
                workspace_state,
                run_state,
                binding_state,
                message_state,
                diagnostics_state,
                messaging::request_workspace_list().await,
            );
        });
    });

    view! {
        <LayoutShell layout_mode=layout_mode />
        <ToastContainer />
    }
}
