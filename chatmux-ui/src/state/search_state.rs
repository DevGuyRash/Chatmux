//! Search and filter state.

use leptos::prelude::*;

/// Search and filter state for the message log.
#[derive(Clone, Copy)]
pub struct SearchState {
    pub query: ReadSignal<String>,
    pub set_query: WriteSignal<String>,
    pub is_active: ReadSignal<bool>,
    pub set_is_active: WriteSignal<bool>,
    pub show_filters: ReadSignal<bool>,
    pub set_show_filters: WriteSignal<bool>,
    pub result_count: ReadSignal<u32>,
    pub set_result_count: WriteSignal<u32>,
    pub current_result: ReadSignal<u32>,
    pub set_current_result: WriteSignal<u32>,
}

pub fn provide_search_state() {
    let (query, set_query) = signal(String::new());
    let (is_active, set_is_active) = signal(false);
    let (show_filters, set_show_filters) = signal(false);
    let (result_count, set_result_count) = signal(0u32);
    let (current_result, set_current_result) = signal(0u32);

    provide_context(SearchState {
        query,
        set_query,
        is_active,
        set_is_active,
        show_filters,
        set_show_filters,
        result_count,
        set_result_count,
        current_result,
        set_current_result,
    });
}
