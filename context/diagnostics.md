# Diagnostics Architecture

Chatmux diagnostics now use a shared structured event envelope across the UI, background, and adapter layers.

## Event Model

`chatmux-common::DiagnosticEvent` is the canonical persisted record.

Core fields:
- identity and time: `id`, `timestamp`
- scope/source: `workspace_id`, `scope`, `source`
- severity: `level`
- human-facing text: `title`, `summary`, `detail`
- machine-facing key: `code`
- correlation fields: `provider_id`, `binding_id`, `run_id`, `round_id`, `message_id`, `dispatch_id`
- structured context: `tags`, `attributes`
- future raw attachments: `artifact_refs`

Compatibility rule:
- New fields are added with serde defaults so existing persisted diagnostics continue to deserialize.

## Query Path

The diagnostics explorer should query through `UiCommand::RequestDiagnosticsSnapshot`.

The coordinator uses `DiagnosticsQuery` to:
- select workspace scope or aggregate all workspaces
- apply backend severity/source/provider filtering
- cap the returned result set
- return `UiEvent::DiagnosticsSnapshot`

The UI keeps text and regex searching local so highlighting, context expansion, copy, and download all use the same filtered visible set.

## Capture Strategy

The coordinator now auto-records:
- `UiCommand` receipt/result/failure
- `AdapterToBackground` ingress/result/failure
- existing explicit warning/error diagnostics such as DOM mismatch and blocking state

When adding new runtime surfaces:
1. Prefer generic recording first.
2. Add prettified `title` / `summary` text second.
3. Put extra stable structured fields in `attributes`.
4. Only add new top-level typed fields when they materially improve cross-feature correlation.

## Explorer Behavior

The diagnostics explorer supports:
- workspace and all-workspace scope
- readable and raw `Event Data` modes
- overview/standard/verbose detail levels
- live mode with continuous reflow
- regex search with optional context-before/context-after event expansion
- copy of selected/current filtered payload
- download of the current filtered payload

## Retention

Diagnostics persistence currently uses a rolling cap in storage (`2500` events total in the active implementation path).

When adjusting retention:
- keep browser-extension storage pressure in mind
- prefer pruning oldest events automatically over requiring manual cleanup
- update this document if the cap or pruning model changes

## Known Follow-up Areas

- Global mode currently aggregates all workspace-scoped diagnostics; true app-scoped records are not emitted yet.
- `artifact_refs` are present in the schema for future raw/blob-backed attachments, but current capture stores raw structured payloads in `attributes`.
