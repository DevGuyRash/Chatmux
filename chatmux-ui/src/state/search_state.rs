//! Search and filter state.

use crate::models::{MessageRole, ProviderId};
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
    pub provider_filter: ReadSignal<Option<ProviderId>>,
    pub set_provider_filter: WriteSignal<Option<ProviderId>>,
    pub role_filter: ReadSignal<Option<MessageRole>>,
    pub set_role_filter: WriteSignal<Option<MessageRole>>,
    pub round_min: ReadSignal<Option<u32>>,
    pub set_round_min: WriteSignal<Option<u32>>,
    pub round_max: ReadSignal<Option<u32>>,
    pub set_round_max: WriteSignal<Option<u32>>,
    pub tag_query: ReadSignal<String>,
    pub set_tag_query: WriteSignal<String>,
}

pub fn provide_search_state() {
    let (query, set_query) = signal(String::new());
    let (is_active, set_is_active) = signal(false);
    let (show_filters, set_show_filters) = signal(false);
    let (result_count, set_result_count) = signal(0u32);
    let (current_result, set_current_result) = signal(0u32);
    let (provider_filter, set_provider_filter) = signal(None);
    let (role_filter, set_role_filter) = signal(None);
    let (round_min, set_round_min) = signal(None);
    let (round_max, set_round_max) = signal(None);
    let (tag_query, set_tag_query) = signal(String::new());

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
        provider_filter,
        set_provider_filter,
        role_filter,
        set_role_filter,
        round_min,
        set_round_min,
        round_max,
        set_round_max,
        tag_query,
        set_tag_query,
    });
}
