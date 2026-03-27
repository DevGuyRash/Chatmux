//! Chatmux UI — Rust/Wasm frontend for the Chatmux browser extension.
//!
//! This crate compiles to WebAssembly and renders the extension's popup,
//! sidebar, and tab page surfaces using Leptos (CSR mode).

pub mod a11y;
pub mod app;
pub mod bridge;
pub mod components;
pub mod layout;
pub mod models;
pub mod state;
pub mod theme;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Wasm entry point. Called from the bootstrap JS to mount the Leptos app.
#[wasm_bindgen(start)]
pub fn main() {
    // Initialize console logging for debug builds
    #[cfg(debug_assertions)]
    {
        console_log::init_with_level(log::Level::Debug)
            .expect("Failed to initialize console_log");
    }
    #[cfg(not(debug_assertions))]
    {
        console_log::init_with_level(log::Level::Warn)
            .expect("Failed to initialize console_log");
    }

    log::info!("Chatmux UI starting");

    let document = web_sys::window()
        .and_then(|window| window.document())
        .expect("window document should be available");
    let app_root = document
        .get_element_by_id("app")
        .expect("Chatmux UI root #app should exist")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("#app should be an HtmlElement");

    leptos::mount::mount_to(app_root, app::App).forget();
}
