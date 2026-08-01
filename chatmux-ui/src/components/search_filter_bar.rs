//! Search and filter bar component (§3.22).
//!
//! Appears at the top of the message log. Toggled by search icon.
//! Search input with in-place highlighting, result counter, navigation.
//! Filter row with participant, role, run, round range, status, and tag filters.

use crate::components::primitives::button::{Button, ButtonSize, ButtonVariant};
use crate::components::primitives::icon::{Icon, IconKind};
use crate::models::{MessageRole, ProviderId};
use crate::state::app_state::AppState;
use leptos::prelude::*;

/// Saved filter presets are capped per workspace so the picker stays scannable.
const SAVED_FILTER_LIMIT: usize = 10;

/// Search and filter bar component.
#[component]
pub fn SearchFilterBar(
    /// Current search query.
    query: ReadSignal<String>,
    /// Set the search query.
    set_query: WriteSignal<String>,
    /// Whether the search bar is visible.
    is_active: ReadSignal<bool>,
    /// Whether the filter row is expanded.
    show_filters: ReadSignal<bool>,
    /// Toggle filter row visibility.
    set_show_filters: WriteSignal<bool>,
    /// Total result count.
    result_count: ReadSignal<u32>,
    /// Current result index (1-based).
    current_result: ReadSignal<u32>,
    /// Navigate to next result.
    on_next: impl Fn() + 'static + Copy + Send,
    /// Navigate to previous result.
    on_prev: impl Fn() + 'static + Copy + Send,
    /// Close the search bar.
    on_close: impl Fn() + 'static + Copy + Send,
    /// Provider filter.
    provider_filter: ReadSignal<Option<ProviderId>>,
    set_provider_filter: WriteSignal<Option<ProviderId>>,
    /// Role filter.
    role_filter: ReadSignal<Option<MessageRole>>,
    set_role_filter: WriteSignal<Option<MessageRole>>,
    round_min: ReadSignal<Option<u32>>,
    set_round_min: WriteSignal<Option<u32>>,
    round_max: ReadSignal<Option<u32>>,
    set_round_max: WriteSignal<Option<u32>>,
    tag_query: ReadSignal<String>,
    set_tag_query: WriteSignal<String>,
) -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let (selected_saved_filter, set_selected_saved_filter) = signal(String::new());
    let (save_name, set_save_name) = signal(String::new());
    let saved_filter_count = Signal::derive(move || {
        app_state
            .active_workspace_id
            .get()
            .and_then(|workspace_id| {
                app_state
                    .ui_settings
                    .get()
                    .saved_search_filters
                    .get(&workspace_id)
                    .map(|filters| filters.len())
            })
            .unwrap_or(0)
    });
    // Hoisted out of the view: a bare `>=` inside a `view!` attribute closure
    // is read by the macro as the tag-closing `>`.
    let saved_filter_limit_reached =
        Signal::derive(move || saved_filter_count.get() >= SAVED_FILTER_LIMIT);
    let save_button_title = Signal::derive(move || {
        if saved_filter_limit_reached.get() {
            format!("Preset limit reached ({SAVED_FILTER_LIMIT}). Delete one to save another.")
        } else {
            "Save the current filters as a preset".to_owned()
        }
    });
    view! {
        {move || is_active.get().then(|| view! {
            <div class="search-filter-bar border-b"
                 style="background: var(--surface-raised);">

                // Search input row
                <div class="flex items-center gap-2 px-4 py-3">
                    // Search icon
                    <span class="text-tertiary flex items-center" style="flex-shrink: 0;">
                        <Icon kind=IconKind::Search size=14 />
                    </span>

                    // Input
                    <input
                        class="type-body flex-1"
                        type="text"
                        placeholder="Search messages…"
                        style="\
                            background: var(--surface-sunken); \
                            border: 1px solid var(--border-default); \
                            border-radius: var(--radius-md); \
                            padding: var(--space-3) var(--space-4); \
                            color: var(--text-primary); \
                            min-width: 0;"
                        prop:value=move || query.get()
                        on:input=move |ev| set_query.set(event_target_value(&ev))
                    />

                    // Result counter
                    {move || {
                        let count = result_count.get();
                        (count > 0).then(|| {
                            let current = current_result.get();
                            view! {
                                <span class="type-caption text-secondary" style="white-space: nowrap;">
                                    {format!("{current} of {count}")}
                                </span>
                            }
                        })
                    }}

                    // Navigation arrows
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        aria_label="Previous result".to_string()
                        on_click=Box::new(move |_| on_prev())
                    >
                        <Icon kind=IconKind::ArrowUp size=14 />
                    </Button>
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        aria_label="Next result".to_string()
                        on_click=Box::new(move |_| on_next())
                    >
                        <Icon kind=IconKind::ArrowDown size=14 />
                    </Button>

                    // Filter toggle
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        title="Toggle filters".to_string()
                        aria_label="Toggle filters".to_string()
                        aria_pressed=Signal::derive(move || Some(show_filters.get()))
                        on_click=Box::new(move |_| set_show_filters.update(|v| *v = !*v))
                    >
                        <Icon kind=IconKind::Funnel size=14 />
                    </Button>

                    // Close
                    <Button
                        variant=ButtonVariant::Icon
                        size=ButtonSize::Small
                        aria_label="Close search".to_string()
                        on_click=Box::new(move |_| on_close())
                    >
                        <Icon kind=IconKind::Close size=14 />
                    </Button>
                </div>

                // Filter row (expanded)
                {move || show_filters.get().then(|| view! {
                    <div class="flex items-center gap-3 px-4 py-2 flex-wrap border-t">
                        <span class="type-caption text-secondary">"Filters:"</span>
                        <select
                            aria-label="Filter by provider"
                            class="type-caption text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-2) var(--space-8) var(--space-2) var(--space-3);"
                            prop:value=move || provider_filter.get().map(provider_key).unwrap_or("")
                            on:change=move |event| set_provider_filter.set(provider_from_key(&event_target_value(&event)))
                        >
                            <option value="">"All participants"</option>
                            <option value="user">"You"</option>
                            <option value="gpt">"ChatGPT"</option>
                            <option value="gemini">"Gemini"</option>
                            <option value="grok">"Grok"</option>
                            <option value="claude">"Claude"</option>
                            <option value="system">"System"</option>
                        </select>
                        <input
                            aria-label="Minimum round"
                            class="type-caption text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-2) var(--space-3); width: 7rem;"
                            type="number"
                            min="1"
                            placeholder="From round"
                            prop:value=move || round_min.get().map(|value| value.to_string()).unwrap_or_default()
                            on:change=move |event| set_round_min.set(event_target_value(&event).parse().ok())
                        />
                        <input
                            aria-label="Maximum round"
                            class="type-caption text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-2) var(--space-3); width: 7rem;"
                            type="number"
                            min="1"
                            placeholder="To round"
                            prop:value=move || round_max.get().map(|value| value.to_string()).unwrap_or_default()
                            on:change=move |event| set_round_max.set(event_target_value(&event).parse().ok())
                        />
                        <input
                            aria-label="Filter by tag"
                            class="type-caption text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-2) var(--space-3); min-width: 9rem;"
                            type="text"
                            placeholder="Tag contains…"
                            prop:value=move || tag_query.get()
                            on:input=move |event| set_tag_query.set(event_target_value(&event))
                        />

                        <select
                            aria-label="Saved filters"
                            class="type-caption text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-2) var(--space-8) var(--space-2) var(--space-3);"
                            prop:value=move || selected_saved_filter.get()
                            on:change=move |event| {
                                let id = event_target_value(&event);
                                set_selected_saved_filter.set(id.clone());
                                let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else { return; };
                                if let Some(filter) = app_state.ui_settings.get_untracked()
                                    .saved_search_filters.get(&workspace_id)
                                    .and_then(|filters| filters.iter().find(|filter| filter.id.to_string() == id))
                                    .cloned()
                                {
                                    set_query.set(filter.query);
                                    set_provider_filter.set(filter.provider);
                                    set_role_filter.set(filter.role);
                                    set_round_min.set(filter.round_min);
                                    set_round_max.set(filter.round_max);
                                    set_tag_query.set(filter.tag_query);
                                }
                            }
                        >
                            <option value="">"Saved filters"</option>
                            {move || {
                                let Some(workspace_id) = app_state.active_workspace_id.get() else { return Vec::new().into_iter().collect_view(); };
                                app_state.ui_settings.get().saved_search_filters
                                    .get(&workspace_id)
                                    .cloned()
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|filter| view! { <option value=filter.id.to_string()>{filter.name}</option> })
                                    .collect_view()
                            }}
                        </select>
                        <input
                            aria-label="Saved filter name"
                            class="type-caption text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-2) var(--space-3); min-width: 9rem;"
                            type="text"
                            placeholder="Preset name"
                            prop:value=move || save_name.get()
                            on:input=move |event| set_save_name.set(event_target_value(&event))
                        />
                        <button
                            class="type-caption cursor-pointer text-link"
                            // Disabling at the cap is the real fix: it makes the
                            // limit visible before the user composes a name,
                            // rather than after.
                            disabled=saved_filter_limit_reached
                            title=save_button_title
                            on:click=move |_| {
                                let name = save_name.get_untracked().trim().to_owned();
                                let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else { return; };
                                if name.is_empty() { return; }
                                // Checked out here, not inside the update closure.
                                // A `return` in there exits only the closure, so
                                // the field was cleared regardless and a discarded
                                // preset looked exactly like a saved one.
                                if saved_filter_limit_reached.get_untracked() {
                                    crate::state::controller::publish_user_outcome(
                                        app_state,
                                        crate::state::command_state::CommandOutcomeKind::Error,
                                        format!("Preset limit reached ({SAVED_FILTER_LIMIT}). Delete a preset before saving another."),
                                    );
                                    return;
                                }
                                app_state.set_ui_settings.update(|settings| {
                                    let filters = settings.saved_search_filters.entry(workspace_id).or_default();
                                    let filter = crate::bridge::storage::SavedSearchFilter {
                                        id: uuid::Uuid::new_v4(),
                                        name,
                                        query: query.get_untracked(),
                                        provider: provider_filter.get_untracked(),
                                        role: role_filter.get_untracked(),
                                        round_min: round_min.get_untracked(),
                                        round_max: round_max.get_untracked(),
                                        tag_query: tag_query.get_untracked(),
                                    };
                                    set_selected_saved_filter.set(filter.id.to_string());
                                    filters.push(filter);
                                });
                                set_save_name.set(String::new());
                                crate::state::controller::publish_user_outcome(
                                    app_state,
                                    crate::state::command_state::CommandOutcomeKind::Success,
                                    "Filter preset saved.",
                                );
                            }
                        >"Save filter"</button>
                        <button
                            class="type-caption cursor-pointer text-link"
                            disabled=move || selected_saved_filter.get().is_empty()
                            on:click=move |_| {
                                let selected = selected_saved_filter.get_untracked();
                                let Some(workspace_id) = app_state.active_workspace_id.get_untracked() else { return; };
                                app_state.set_ui_settings.update(|settings| {
                                    if let Some(filters) = settings.saved_search_filters.get_mut(&workspace_id) {
                                        filters.retain(|filter| filter.id.to_string() != selected);
                                    }
                                });
                                set_selected_saved_filter.set(String::new());
                            }
                        >"Delete preset"</button>
                        <select
                            aria-label="Filter by role"
                            class="type-caption text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-2) var(--space-8) var(--space-2) var(--space-3);"
                            prop:value=move || role_filter.get().map(role_key).unwrap_or("")
                            on:change=move |event| set_role_filter.set(role_from_key(&event_target_value(&event)))
                        >
                            <option value="">"All roles"</option>
                            <option value="user">"User"</option>
                            <option value="assistant">"Assistant"</option>
                            <option value="system">"System"</option>
                        </select>

                        <button
                            class="type-caption cursor-pointer text-link"
                            style="margin-left: auto;"
                            on:click=move |_| {
                                set_provider_filter.set(None);
                                set_role_filter.set(None);
                                set_round_min.set(None);
                                set_round_max.set(None);
                                set_tag_query.set(String::new());
                            }
                        >
                            "Clear filters"
                        </button>
                    </div>
                })}
            </div>
        })}
    }
}

fn provider_key(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::User => "user",
        ProviderId::System => "system",
        ProviderId::Gpt => "gpt",
        ProviderId::Gemini => "gemini",
        ProviderId::Grok => "grok",
        ProviderId::Claude => "claude",
    }
}

fn provider_from_key(value: &str) -> Option<ProviderId> {
    match value {
        "user" => Some(ProviderId::User),
        "system" => Some(ProviderId::System),
        "gpt" => Some(ProviderId::Gpt),
        "gemini" => Some(ProviderId::Gemini),
        "grok" => Some(ProviderId::Grok),
        "claude" => Some(ProviderId::Claude),
        _ => None,
    }
}

fn role_key(role: MessageRole) -> &'static str {
    match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    }
}

fn role_from_key(value: &str) -> Option<MessageRole> {
    match value {
        "user" => Some(MessageRole::User),
        "assistant" => Some(MessageRole::Assistant),
        "system" => Some(MessageRole::System),
        _ => None,
    }
}
