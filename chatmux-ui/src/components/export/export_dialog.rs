//! Complete workspace export and copy dialog (§3.13).

use leptos::prelude::*;
use std::collections::BTreeSet;

use crate::bridge::{clipboard, messaging};
use crate::components::primitives::button::{Button, ButtonVariant};
use crate::components::primitives::modal::Modal;
use crate::models::{
    ExportFilterPreset, ExportFormat, ExportLayout, ExportProfile, ExportProfileId, ExportRequest,
    ExportScopePreset, MetadataIncludeFlags, ProviderId, RunId, UiEvent, WorkspaceId,
};
use crate::state::command_state::CommandOutcomeKind;
use crate::state::{
    app_state::{AppState, ExportState},
    binding_state::BindingState,
    controller::{dispatch_command_result, publish_user_outcome},
    diagnostics_state::DiagnosticsState,
    message_state::MessageState,
    run_state::ActiveRunState,
    workspace_state::WorkspaceListState,
};

#[derive(Clone, Copy)]
struct ExportContexts {
    app: AppState,
    workspaces: WorkspaceListState,
    runs: ActiveRunState,
    bindings: BindingState,
    messages: MessageState,
    diagnostics: DiagnosticsState,
}

/// Export, copy, download, and save-profile workflow.
#[component]
pub fn ExportDialog(
    /// Whether the dialog is open.
    open: ReadSignal<bool>,
    /// Workspace being exported.
    workspace_id: WorkspaceId,
    /// Called to close.
    on_close: impl Fn() + 'static + Copy + Send,
) -> impl IntoView {
    let contexts = ExportContexts {
        app: expect_context::<AppState>(),
        workspaces: expect_context::<WorkspaceListState>(),
        runs: expect_context::<ActiveRunState>(),
        bindings: expect_context::<BindingState>(),
        messages: expect_context::<MessageState>(),
        diagnostics: expect_context::<DiagnosticsState>(),
    };
    let (format, set_format) = signal(ExportFormat::Markdown);
    let (layout, set_layout) = signal(ExportLayout::Chronological);
    let (scope, set_scope) = signal(ExportScopePreset::EntireWorkspace);
    let (provider, set_provider) = signal(None::<ProviderId>);
    let (run_id, set_run_id) = signal(None::<RunId>);
    let (selected_messages, set_selected_messages) = signal(BTreeSet::new());
    let (selected_rounds, set_selected_rounds) = signal(BTreeSet::new());
    let (include_metadata, set_include_metadata) = signal(true);
    let (include_front_matter, set_include_front_matter) = signal(true);
    let (filename_template, set_filename_template) =
        signal("{workspace}-{date}-{format}".to_owned());
    let (profile_id, set_profile_id) = signal(None::<ExportProfileId>);
    let (profile_name, set_profile_name) = signal(String::new());
    let (busy, set_busy) = signal(false);

    let build_request = move || ExportRequest {
        workspace_id,
        scope: scope.get_untracked(),
        format: format.get_untracked(),
        layout: layout.get_untracked(),
        profile_id: profile_id.get_untracked(),
        participants: provider.get_untracked().into_iter().collect(),
        roles: BTreeSet::new(),
        selected_message_ids: selected_messages.get_untracked(),
        selected_rounds: selected_rounds.get_untracked(),
        run_id: run_id.get_untracked(),
        time_range_iso: None,
        delivery_outcomes: Vec::new(),
        tags: Vec::new(),
        query: None,
        invert_selection: false,
        include_flags: metadata_flags(include_metadata.get_untracked()),
        include_front_matter: include_front_matter.get_untracked(),
        filename_template: Some(filename_template.get_untracked()),
    };

    let execute_export = move |copy: bool| {
        if busy.get_untracked() {
            return;
        }
        set_busy.set(true);
        let request = build_request();
        leptos::task::spawn_local_scoped_with_cancellation(async move {
            let result = messaging::export_configured(request).await;
            let export = export_from_result(&result);
            dispatch_command_result(
                contexts.app,
                contexts.workspaces,
                contexts.runs,
                contexts.bindings,
                contexts.messages,
                contexts.diagnostics,
                result,
            );
            let completed = if let Some(export) = export {
                if copy {
                    clipboard::write_clipboard(&export.body).await
                } else {
                    clipboard::download_text(&export.filename, &export.mime_type, &export.body)
                        .await
                }
            } else {
                false
            };
            publish_user_outcome(
                contexts.app,
                if completed {
                    CommandOutcomeKind::Success
                } else {
                    CommandOutcomeKind::Error
                },
                if completed && copy {
                    "Rendered export copied to the clipboard."
                } else if completed {
                    "Rendered export downloaded."
                } else {
                    "The export could not be completed. Review the export selection and try again."
                },
            );
            set_busy.set(false);
            if completed {
                on_close();
            }
        });
    };

    view! {
        <Modal open=open on_close=on_close max_width=720>
            <div class="flex flex-col gap-6" style="max-height: 80vh; overflow-y: auto;">
                <div class="flex items-center justify-between gap-4">
                    <div>
                        <h2 class="type-title text-primary">"Export workspace"</h2>
                        <p class="type-caption text-secondary mt-1">
                            "Choose an exact scope, reusable profile, format, and destination."
                        </p>
                    </div>
                    <span class="type-caption text-tertiary">"Local data only"</span>
                </div>

                <section class="flex flex-col gap-3">
                    <label class="type-label text-secondary" for="export-profile">"Saved profile"</label>
                    <select
                        id="export-profile"
                        class="type-body text-primary surface-sunken border rounded-md"
                        style="padding: var(--space-3) var(--space-4);"
                        on:change=move |event| {
                            let value = event_target_value(&event);
                            let selected = contexts.workspaces.snapshot.get_untracked()
                                .and_then(|snapshot| snapshot.export_profiles.into_iter()
                                    .find(|profile| profile.id.0.to_string() == value));
                            if let Some(profile) = selected {
                                set_profile_id.set(Some(profile.id));
                                set_scope.set(profile.scope_preset);
                                set_format.set(profile.format);
                                set_layout.set(profile.layout);
                                set_provider.set(profile.filter_preset.participants.iter().next().copied());
                                set_run_id.set(profile.filter_preset.run_id);
                                set_filename_template.set(profile.filename_template);
                                set_include_metadata.set(profile.include_flags != MetadataIncludeFlags::default());
                            } else {
                                set_profile_id.set(None);
                            }
                        }
                    >
                        <option value="">"Custom selection"</option>
                        {move || contexts.workspaces.snapshot.get()
                            .map(|snapshot| snapshot.export_profiles)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|profile| view! {
                                <option value=profile.id.0.to_string()>{profile.name}</option>
                            })
                            .collect_view()}
                    </select>
                </section>

                <div class="grid grid-cols-2 gap-5">
                    <section class="flex flex-col gap-3">
                        <label class="type-label text-secondary" for="export-scope">"Scope"</label>
                        <select
                            id="export-scope"
                            class="type-body text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-3) var(--space-4);"
                            prop:value=move || scope_key(scope.get())
                            on:change=move |event| set_scope.set(scope_from_key(&event_target_value(&event)))
                        >
                            <option value="workspace">"Entire workspace"</option>
                            <option value="provider">"Single provider + related user turns"</option>
                            <option value="run">"Single run"</option>
                            <option value="rounds">"Selected rounds"</option>
                            <option value="messages">"Selected messages"</option>
                            <option value="providers">"Provider-only subset"</option>
                            <option value="dispatch">"Dispatch/payload log"</option>
                            <option value="diagnostics">"Diagnostics"</option>
                        </select>
                    </section>
                    <section class="flex flex-col gap-3">
                        <label class="type-label text-secondary" for="export-format">"Format"</label>
                        <select
                            id="export-format"
                            class="type-body text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-3) var(--space-4);"
                            prop:value=move || format_key(format.get())
                            on:change=move |event| set_format.set(format_from_key(&event_target_value(&event)))
                        >
                            <option value="markdown">"Markdown"</option>
                            <option value="json">"JSON"</option>
                            <option value="toml">"TOML"</option>
                        </select>
                    </section>
                </div>

                <Show when=move || matches!(scope.get(), ExportScopePreset::SingleProvider | ExportScopePreset::ProviderOnlySubset)>
                    <section class="flex flex-col gap-3">
                        <label class="type-label text-secondary" for="export-provider">"Provider"</label>
                        <select
                            id="export-provider"
                            class="type-body text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-3) var(--space-4);"
                            on:change=move |event| set_provider.set(provider_from_key(&event_target_value(&event)))
                        >
                            <option value="">"Select a provider"</option>
                            <option value="gpt">"ChatGPT"</option>
                            <option value="gemini">"Gemini"</option>
                            <option value="grok">"Grok"</option>
                            <option value="claude">"Claude"</option>
                        </select>
                    </section>
                </Show>

                <Show when=move || scope.get() == ExportScopePreset::SingleRun>
                    <section class="flex flex-col gap-3">
                        <label class="type-label text-secondary" for="export-run">"Run"</label>
                        <select
                            id="export-run"
                            class="type-body text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-3) var(--space-4);"
                            on:change=move |event| {
                                let value = event_target_value(&event);
                                let selected = contexts.workspaces.snapshot.get_untracked()
                                    .and_then(|snapshot| snapshot.runs.into_iter()
                                        .find(|run| run.id.0.to_string() == value));
                                set_run_id.set(selected.map(|run| run.id));
                            }
                        >
                            <option value="">"Select a run"</option>
                            {move || contexts.workspaces.snapshot.get()
                                .map(|snapshot| snapshot.runs)
                                .unwrap_or_default()
                                .into_iter()
                                .map(|run| view! {
                                    <option value=run.id.0.to_string()>
                                        // Mode and status alone do not separate
                                        // two runs of the same shape, so the
                                        // start time is what makes the list
                                        // selectable.
                                        {format!(
                                            "{} · {:?}{}",
                                            crate::models::view_models::orchestration_mode_label(run.mode),
                                            run.status,
                                            run.started_at
                                                .map(|at| format!(" · {}", crate::time::format_local_datetime(at)))
                                                .unwrap_or_default(),
                                        )}
                                    </option>
                                })
                                .collect_view()}
                        </select>
                    </section>
                </Show>

                <Show when=move || matches!(scope.get(), ExportScopePreset::SelectedRounds | ExportScopePreset::SelectedMessages)>
                    <section class="flex flex-col gap-3">
                        <h3 class="type-label text-secondary">
                            {move || if scope.get() == ExportScopePreset::SelectedRounds { "Rounds" } else { "Messages" }}
                        </h3>
                        <div class="flex flex-col gap-2 surface-sunken border rounded-md p-3" style="max-height: 220px; overflow-y: auto;">
                            {move || if scope.get() == ExportScopePreset::SelectedRounds {
                                let rounds = contexts.messages.messages.get().into_iter()
                                    .filter_map(|message| message.round)
                                    .collect::<BTreeSet<_>>();
                                rounds.into_iter().map(|round| view! {
                                    <label class="flex items-center gap-3 type-body text-primary">
                                        <input type="checkbox"
                                            checked=move || selected_rounds.get().contains(&round)
                                            on:change=move |event| set_selected_rounds.update(|selected| {
                                                if event_target_checked(&event) { selected.insert(round); } else { selected.remove(&round); }
                                            }) />
                                        {format!("Round {round}")}
                                    </label>
                                }).collect_view().into_any()
                            } else {
                                contexts.messages.messages.get().into_iter().map(|message| {
                                    let message_id = message.id;
                                    let label = format!("{} · {:?} · {}", message.participant_id.display_name(), message.role, truncate(&message.body_text));
                                    view! {
                                        <label class="flex items-center gap-3 type-body text-primary">
                                            <input type="checkbox"
                                                checked=move || selected_messages.get().contains(&message_id)
                                                on:change=move |event| set_selected_messages.update(|selected| {
                                                    if event_target_checked(&event) { selected.insert(message_id); } else { selected.remove(&message_id); }
                                                }) />
                                            <span>{label}</span>
                                        </label>
                                    }
                                }).collect_view().into_any()
                            }}
                        </div>
                    </section>
                </Show>

                <div class="grid grid-cols-2 gap-5">
                    <section class="flex flex-col gap-3">
                        <label class="type-label text-secondary" for="export-layout">"Body layout"</label>
                        <select
                            id="export-layout"
                            class="type-body text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-3) var(--space-4);"
                            prop:value=move || layout_key(layout.get())
                            on:change=move |event| set_layout.set(layout_from_key(&event_target_value(&event)))
                        >
                            <option value="chronological">"Chronological"</option>
                            <option value="round">"Grouped by round"</option>
                            <option value="participant">"Grouped by participant"</option>
                        </select>
                    </section>
                    <section class="flex flex-col gap-3">
                        <label class="type-label text-secondary" for="export-filename">"Filename template"</label>
                        <input
                            id="export-filename"
                            class="type-body text-primary surface-sunken border rounded-md"
                            style="padding: var(--space-3) var(--space-4);"
                            prop:value=move || filename_template.get()
                            on:input=move |event| set_filename_template.set(event_target_value(&event))
                        />
                        <span class="type-caption text-tertiary">"Keys: {workspace}, {date}, {format}, {provider}, {profile}, {run}"</span>
                    </section>
                </div>

                <section class="grid grid-cols-2 gap-3">
                    <label class="flex items-center gap-3 type-body text-primary">
                        <input type="checkbox" checked=move || include_metadata.get()
                            on:change=move |event| set_include_metadata.set(event_target_checked(&event)) />
                        "Include configurable metadata"
                    </label>
                    <label class="flex items-center gap-3 type-body text-primary">
                        <input type="checkbox" checked=move || include_front_matter.get()
                            disabled=move || format.get() != ExportFormat::Markdown
                            on:change=move |event| set_include_front_matter.set(event_target_checked(&event)) />
                        "TOML front matter (Markdown)"
                    </label>
                </section>

                <section class="flex flex-col gap-3 border-t pt-4">
                    <label class="type-label text-secondary" for="export-profile-name">"Save this configuration"</label>
                    <div class="flex items-center gap-3">
                        <input
                            id="export-profile-name"
                            class="type-body text-primary surface-sunken border rounded-md flex-1"
                            style="padding: var(--space-3) var(--space-4);"
                            placeholder="Profile name"
                            prop:value=move || profile_name.get()
                            on:input=move |event| set_profile_name.set(event_target_value(&event))
                        />
                        <Button
                            variant=ButtonVariant::Secondary
                            disabled=Signal::derive(move || profile_name.get().trim().is_empty() || busy.get())
                            on_click=Box::new(move |_| {
                                let name = profile_name.get_untracked().trim().to_owned();
                                if name.is_empty() { return; }
                                let request = build_request();
                                let id = profile_id.get_untracked().unwrap_or_else(ExportProfileId::new);
                                let profile = ExportProfile {
                                    id,
                                    workspace_id,
                                    name,
                                    scope_preset: request.scope,
                                    filter_preset: ExportFilterPreset {
                                        participants: request.participants,
                                        roles: request.roles,
                                        round_range: round_bounds(&request.selected_rounds),
                                        time_range_iso: request.time_range_iso,
                                        run_id: request.run_id,
                                        tags: request.tags,
                                        query: request.query,
                                    },
                                    format: request.format,
                                    layout: request.layout,
                                    include_flags: request.include_flags,
                                    filename_template: request.filename_template.unwrap_or_default(),
                                    metadata_template: None,
                                    prefer_copy: false,
                                };
                                set_profile_id.set(Some(id));
                                set_profile_name.set(String::new());
                                leptos::task::spawn_local(async move {
                                    let result = messaging::persist_export_profile(profile).await;
                                    dispatch_command_result(
                                        contexts.app, contexts.workspaces, contexts.runs,
                                        contexts.bindings, contexts.messages, contexts.diagnostics,
                                        result,
                                    );
                                });
                            })
                        >"Save profile"</Button>
                    </div>
                </section>

                <div class="flex justify-end items-center gap-3 border-t pt-4">
                    <Button variant=ButtonVariant::Secondary disabled=busy on_click=Box::new(move |_| execute_export(true))>
                        "Copy rendered export"
                    </Button>
                    <Button variant=ButtonVariant::Primary disabled=busy on_click=Box::new(move |_| execute_export(false))>
                        "Download file"
                    </Button>
                </div>
            </div>
        </Modal>
    }
}

fn export_from_result(result: &Result<Vec<UiEvent>, String>) -> Option<ExportState> {
    result.as_ref().ok()?.iter().find_map(|event| match event {
        UiEvent::ExportRendered {
            format,
            mime_type,
            filename,
            body,
        } => Some(ExportState {
            format: *format,
            mime_type: mime_type.clone(),
            filename: filename.clone(),
            body: body.clone(),
        }),
        _ => None,
    })
}

fn metadata_flags(include: bool) -> MetadataIncludeFlags {
    if !include {
        return MetadataIncludeFlags::default();
    }
    MetadataIncludeFlags {
        workspace_name: true,
        workspace_id: true,
        export_title: true,
        export_timestamp: true,
        scope_type: true,
        selected_participants: true,
        orchestration_mode: true,
        run_id: true,
        round_range: true,
        message_count: true,
        template_used: true,
        context_strategy_snapshot: true,
        edge_policy_snapshot: true,
        conversation_refs: true,
        model_labels: true,
        browser_name: true,
        extension_version: true,
        export_profile_name: true,
        tags_and_notes: true,
        diagnostics_summary: true,
        raw_payload_inclusion: false,
    }
}

fn scope_key(scope: ExportScopePreset) -> &'static str {
    match scope {
        ExportScopePreset::EntireWorkspace => "workspace",
        ExportScopePreset::SingleProvider => "provider",
        ExportScopePreset::SingleRun => "run",
        ExportScopePreset::SelectedRounds => "rounds",
        ExportScopePreset::SelectedMessages => "messages",
        ExportScopePreset::ProviderOnlySubset => "providers",
        ExportScopePreset::DispatchSubset => "dispatch",
        ExportScopePreset::DiagnosticSubset => "diagnostics",
    }
}

fn scope_from_key(value: &str) -> ExportScopePreset {
    match value {
        "provider" => ExportScopePreset::SingleProvider,
        "run" => ExportScopePreset::SingleRun,
        "rounds" => ExportScopePreset::SelectedRounds,
        "messages" => ExportScopePreset::SelectedMessages,
        "providers" => ExportScopePreset::ProviderOnlySubset,
        "dispatch" => ExportScopePreset::DispatchSubset,
        "diagnostics" => ExportScopePreset::DiagnosticSubset,
        _ => ExportScopePreset::EntireWorkspace,
    }
}

fn format_key(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Markdown => "markdown",
        ExportFormat::Json => "json",
        ExportFormat::Toml => "toml",
    }
}

fn format_from_key(value: &str) -> ExportFormat {
    match value {
        "json" => ExportFormat::Json,
        "toml" => ExportFormat::Toml,
        _ => ExportFormat::Markdown,
    }
}

fn layout_key(layout: ExportLayout) -> &'static str {
    match layout {
        ExportLayout::Chronological => "chronological",
        ExportLayout::GroupedByRound => "round",
        ExportLayout::GroupedByParticipant => "participant",
    }
}

fn layout_from_key(value: &str) -> ExportLayout {
    match value {
        "round" => ExportLayout::GroupedByRound,
        "participant" => ExportLayout::GroupedByParticipant,
        _ => ExportLayout::Chronological,
    }
}

fn provider_from_key(value: &str) -> Option<ProviderId> {
    match value {
        "gpt" => Some(ProviderId::Gpt),
        "gemini" => Some(ProviderId::Gemini),
        "grok" => Some(ProviderId::Grok),
        "claude" => Some(ProviderId::Claude),
        _ => None,
    }
}

fn round_bounds(rounds: &BTreeSet<u32>) -> Option<(u32, u32)> {
    Some((*rounds.first()?, *rounds.last()?))
}

fn truncate(value: &str) -> String {
    let mut output = value.chars().take(72).collect::<String>();
    if value.chars().count() > 72 {
        output.push('…');
    }
    output
}
