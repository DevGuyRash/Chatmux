//! Accessibility helpers (§8).
//!
//! Focus management, live regions, keyboard navigation, and
//! reduced motion detection.

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

/// Detect if the user prefers reduced motion.
pub fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

/// Create a reactive signal tracking reduced motion preference.
pub fn use_reduced_motion() -> ReadSignal<bool> {
    let (reduced, set_reduced) = signal(prefers_reduced_motion());

    Effect::new(move |_| {
        let window = web_sys::window().expect("no window");
        if let Ok(Some(mql)) = window.match_media("(prefers-reduced-motion: reduce)") {
            let closure = Closure::wrap(Box::new(move |_: web_sys::MediaQueryListEvent| {
                set_reduced.set(prefers_reduced_motion());
            }) as Box<dyn Fn(_)>);

            let _ =
                mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    });

    reduced
}

/// Move focus to a specific element by selector.
/// Used when panels open (§8.2: focus moves to first interactive element).
pub fn focus_element(selector: &str) {
    if let Some(document) = web_sys::window().and_then(|w| w.document())
        && let Ok(Some(el)) = document.query_selector(selector)
        && let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>()
    {
        let _ = html_el.focus();
    }
}

/// Keep a Tab key event within the focusable elements of a container.
pub fn trap_focus(container_selector: &str, event: &web_sys::KeyboardEvent) {
    if event.key() != "Tab" {
        return;
    }
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Ok(Some(container)) = document.query_selector(container_selector) else {
        return;
    };
    let Ok(focusable) = container.query_selector_all(
        "button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])",
    ) else {
        return;
    };
    let len = focusable.length();
    if len == 0 {
        event.prevent_default();
        return;
    }
    let first = focusable
        .item(0)
        .and_then(|node| node.dyn_into::<web_sys::Element>().ok());
    let last = focusable
        .item(len - 1)
        .and_then(|node| node.dyn_into::<web_sys::Element>().ok());
    let active = document.active_element();
    let destination = if event.shift_key() && active == first {
        last
    } else if !event.shift_key() && active == last {
        first
    } else {
        None
    };
    if let Some(element) =
        destination.and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    {
        event.prevent_default();
        let _ = element.focus();
    }
}

/// Announce a message to screen readers via an ARIA live region.
/// Creates a temporary element with role="status" and aria-live="polite".
pub fn announce(message: &str) {
    let document = match web_sys::window().and_then(|w| w.document()) {
        Some(d) => d,
        None => return,
    };

    // Find or create the announcer element
    let announcer = match document.get_element_by_id("chatmux-announcer") {
        Some(el) => el,
        None => {
            let el = document.create_element("div").unwrap();
            el.set_id("chatmux-announcer");
            let _ = el.set_attribute("role", "status");
            let _ = el.set_attribute("aria-live", "polite");
            let _ = el.set_attribute("aria-atomic", "true");
            let _ = el.set_attribute("class", "sr-only");
            if let Some(body) = document.body() {
                let _ = body.append_child(&el);
            }
            el
        }
    };

    // Clear and set — the change triggers the screen reader announcement
    announcer.set_text_content(Some(""));

    let msg = message.to_string();
    gloo_timers::callback::Timeout::new(50, move || {
        announcer.set_text_content(Some(&msg));
    })
    .forget();
}

/// Announce a message assertively (for urgent notifications like errors).
pub fn announce_assertive(message: &str) {
    let document = match web_sys::window().and_then(|w| w.document()) {
        Some(d) => d,
        None => return,
    };

    let announcer = match document.get_element_by_id("chatmux-announcer-assertive") {
        Some(el) => el,
        None => {
            let el = document.create_element("div").unwrap();
            el.set_id("chatmux-announcer-assertive");
            let _ = el.set_attribute("role", "alert");
            let _ = el.set_attribute("aria-live", "assertive");
            let _ = el.set_attribute("aria-atomic", "true");
            let _ = el.set_attribute("class", "sr-only");
            if let Some(body) = document.body() {
                let _ = body.append_child(&el);
            }
            el
        }
    };

    announcer.set_text_content(Some(""));
    let msg = message.to_string();
    gloo_timers::callback::Timeout::new(50, move || {
        announcer.set_text_content(Some(&msg));
    })
    .forget();
}
