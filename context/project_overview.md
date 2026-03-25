# Polyglot — Multi-AI Chat Orchestrator

## Product Requirements Document

**Version:** 1.1.0
**Date:** 2026-03-25
**Status:** Final (converged between both reviewers)
**Scope:** Product and engineering requirements only. No visual or interaction design direction.

---

## 1. Product Definition

Polyglot is a local-first browser extension for Chrome and Firefox that binds live web conversations in ChatGPT, Gemini, Grok, and Claude into one orchestration workspace.

The extension is not a model runtime and not an API client. It uses the user's existing logged-in sessions in the providers' web applications, mirrors transcript state into a canonical local workspace, and lets the user:

1. Send to one or many providers from one place.
2. Package selected outputs from some providers into structured prompts for another.
3. Catch up a provider once with broad context and then switch that route to delta-only forwarding.
4. Run semi-automatic or fully automatic multi-model conversations.
5. Inspect exactly what every target saw, what was sent, and what came back.
6. Export or copy orchestrated and individual transcripts in structured formats.

The product is extension-first because content scripts and host permissions are required to read from and write to the provider pages.

Polyglot routes, packages, and logs context. Conflict maps, summaries, critiques, and syntheses are authored either by the user or by designated provider participants; the orchestrator itself does not generate them.

---

## 2. Problem Statement

Users who regularly compare or combine multiple AI assistants currently do too much manual work:

1. They split attention across separate tabs with no canonical shared state.
2. They copy and paste responses between providers by hand.
3. They manually decide which history each provider needs.
4. They lack a reliable "full catch-up once, then only forward unseen updates" mechanism.
5. They cannot comfortably run multi-model dialogue loops without constant supervision.
6. They cannot easily export the combined result in a structured, reusable form.

Polyglot exists to turn that manual choreography into explicit, local, inspectable orchestration.

---

## 3. Goals

1. Provide a single durable workspace for ChatGPT, Gemini, Grok, and Claude conversations.
2. Make broadcast, relay, compare, and orchestrate first-class workflows.
3. Make "catch-up once, then delta-only" a core routing primitive using persisted delivery cursors.
4. Support manual, assisted, moderated, and autonomous multi-model discussion.
5. Make every outbound package reconstructible and auditable.
6. Keep the system local-first, backend-free, Rust-first, and Wasm-first.
7. Support rich export and copy flows for both orchestrated and per-provider transcripts.

---

## 4. Non-Goals

1. No visual or interaction design specification in this PRD.
2. No direct provider API integration in v1.
3. No cloud backend, hosted accounts, or mandatory sync service.
4. No mobile browser support.
5. No provider account management or auth management.
6. No native attachment relay for files, images, or audio in v1.
7. No model-picker abstraction over provider-specific model menus in v1.
8. No arbitrary browser automation beyond the supported AI providers.
9. No multi-user collaborative sessions in v1.

---

## 5. Platform and Technical Constraints

### 5.1 Browser Targets

Polyglot targets current stable desktop Chrome and Firefox. The project produces separate Chrome and Firefox extension packages from one shared source tree. Core behavior must be equivalent across both browsers; the extension shell may differ where browser APIs differ.

### 5.2 Extension Surfaces

Two first-party surfaces are supported:

- A browser sidebar surface (Chrome `sidePanel` API; Firefox `sidebar_action`).
- A standalone extension page opened in a normal browser tab via `browser.tabs.create` / `chrome.tabs.create`.

Both are supported. The user chooses their preference in settings. Sidebar is the default.

### 5.3 Background Runtime

The architecture must not assume one identical background runtime across Chrome and Firefox.

- **Chrome MV3:** Background coordinator runs as a service worker (`background.service_worker`).
- **Firefox MV3:** Background coordinator runs as background scripts or a background page (`background.scripts` / `background.page`), because Firefox's MV3 implementation does not support `background.service_worker`.
- **Shared core:** Browser-agnostic Rust orchestration logic compiled to Wasm.
- **Browser-specific glue:** Thin bootstrap files and message bridge code only.

### 5.4 Service Worker Ephemerality (Chrome)

Chrome extension service workers are ephemeral and may shut down at any time. Global in-memory state is not a reliable source of truth. The architecture must therefore:

- Treat persisted storage (IndexedDB / storage.local) as authoritative.
- Make the background coordinator resumable and idempotent.
- Persist run state frequently (not only on completion).
- Ensure background logic is safe across restarts and worker suspension.
- Keep all DOM-dependent logic in content scripts or extension pages, never in the service worker.

If Chrome-only DOM access is ever required in background-triggered flows, offscreen documents may be used as a narrow escape hatch, but they must not host primary extension logic.

### 5.5 Rust-First, Wasm-First

All substantive business logic lives in Rust compiled to WebAssembly, including:

- Orchestration engine and run lifecycle.
- Routing graph and edge policy resolution.
- Context selection and delivery cursor management.
- Transcript normalization.
- Prompt packaging and template rendering.
- Export rendering.
- Data model and serialization.
- Adapter capability registry.
- Diagnostic state.

Non-Rust code is kept thin and limited to:

| Artifact | Language | Justification |
|----------|----------|---------------|
| `manifest.json` (per browser) | JSON | Required by the WebExtension specification. |
| Static HTML shells | HTML | Entry points for popup, sidebar, and tab page. Contain only a DOM mount point and a script tag that loads the Wasm module. |
| Content script bootstraps (one per provider) | JavaScript | MV3 requires a JS entry point. Each must be a thin loader whose sole job is to load the Wasm module and delegate. |
| Background service worker / script bootstrap | JavaScript | Same constraint as content scripts. Thin loader only. |
| Wasm-JS bridge glue | JavaScript (generated) | Produced automatically by `wasm-bindgen` or `wasm-pack`. Not hand-written. |

**Prohibited at runtime:**

- No Node.js, npm, or bundlers.
- No React, Vue, Svelte, or any JS framework.
- No TypeScript source files (generated `.d.ts` from wasm-bindgen is acceptable).
- No remote script loading or CDN dependencies.
- No `window.localStorage` (service workers cannot access it; content scripts share page storage).

The extension's own UI (popup, sidebar, tab page) is rendered by a Rust-native Wasm UI framework (candidates: Leptos, Dioxus, or Yew — to be decided during implementation).

**JS health metric:** Hand-written JavaScript must remain bootstrap/bridge-only. The project should track total non-generated JS as an ongoing health metric. If it grows beyond a few hundred lines, that signals architectural drift that should be addressed.

### 5.6 Packaged-Code-Only Runtime

All executable code — JavaScript and WebAssembly — must ship inside the extension package. No CDN, remote script, remote Wasm, or backend-fetched runtime logic is permitted. Chrome's extension CSP must include `'wasm-unsafe-eval'` to allow Wasm execution.

### 5.7 Web-Accessible Resources

Content scripts that load bundled Wasm binaries via `fetch()` require those binaries to be declared in `web_accessible_resources`. The exposed surface must be minimized: only Wasm modules needed by content scripts should be declared, and each resource entry should be scoped to only the specific provider origins that require access (using the `matches` field in MV3's `web_accessible_resources` manifest entry).

### 5.8 Permissions Model

Provider-origin access is granted through optional host permissions requested per provider at runtime. This allows the user to enable providers incrementally and satisfies the principle of least privilege.

`activeTab` is not sufficient as the primary access model because it is temporary and per-invocation. Persistent multi-tab orchestration requires stable host permission grants. `activeTab` may be included only as a convenience for one-off user-invoked actions.

Required permissions:

- `storage` — for settings and lightweight state.
- `unlimitedStorage` — for IndexedDB transcript persistence at scale.
- `sidePanel` (Chrome only) — required permission for the Chrome Side Panel API.
- Optional host permissions for each provider domain, requested at runtime.

Note: On Firefox, sidebar access is configured via the `sidebar_action` manifest key and the `sidebarAction` API. It does not require a separate permission.

Permissions the product must avoid:

- No broad `tabs` permission (matching host permissions are sufficient for provider tabs on both Chrome and Firefox).
- No `cookies`.
- No `webRequest` / `webRequestBlocking`.
- No `<all_urls>`.

### 5.9 Storage Model

Heavy data belongs in IndexedDB. Lightweight settings belong in extension storage.

| Storage Layer | Contents |
|---------------|----------|
| IndexedDB | Workspaces, messages, runs, rounds, dispatches, delivery cursors, export artifacts, diagnostic snapshots. |
| `storage.local` | Global settings, enabled providers, UI preferences, recent binding state, export profile defaults, lightweight run resume markers. |
| `storage.sync` | Optional. Very small user preferences only, if used at all. |

`unlimitedStorage` covers both `storage.local` and IndexedDB quotas.

`window.localStorage` must not be used as a source of truth. Chrome explicitly advises against Web Storage for extension state (service workers cannot access it; content scripts share page storage with the host page).

### 5.10 Content Script Model

Provider adapters run in content scripts because content scripts are the supported extension mechanism for reading and modifying page content. Content scripts run in an isolated world, access the page DOM, and communicate with the rest of the extension through runtime messaging. They do not expose their JS variables to the host page.

### 5.11 Provider Integration Mechanism

Adapters prefer DOM-observation-driven capture and completion detection, using `MutationObserver` as the primary signal with polling as a fallback. Polling uses exponential backoff from 500ms to 5s.

### 5.12 Keyboard Support

User-configurable keyboard shortcuts use the `commands` API on both Chrome and Firefox.

### 5.13 Clipboard Architecture

`navigator.clipboard` is not available in service workers. All clipboard copy flows must originate from the UI layer (extension page or sidebar), triggered by user gestures. The `clipboardWrite` permission is not required for user-gesture-driven copy and should remain optional unless background-triggered copy is later needed.

### 5.14 Build Toolchain

- `cargo` as the primary build system.
- `wasm-pack` or `wasm-bindgen-cli` for Wasm output.
- A thin build script (`Makefile`, `just`, or `cargo-xtask`) to assemble the final extension directory structures for Chrome and Firefox, copy manifests, and package.
- `web-ext` (Mozilla CLI) may be used as a build-time-only dev dependency for packaging and signing. It is not a runtime dependency.

---

## 6. Supported Providers

V1 supports these providers through their web applications and the user's own logged-in sessions:

| Provider | Web App URL Patterns | Codename |
|----------|---------------------|----------|
| ChatGPT | `https://chat.openai.com/*`, `https://chatgpt.com/*` | `gpt` |
| Gemini | `https://gemini.google.com/*` | `gemini` |
| Grok | `https://grok.com/*`, `https://x.com/i/grok*` | `grok` |
| Claude | `https://claude.ai/*` | `claude` |

Each provider is implemented as an isolated adapter module (see §9).

---

## 7. Core Concepts

### 7.1 Workspace

A workspace is the durable top-level container. It includes bound participants, a canonical message log, routing policies, templates, export profiles, diagnostics history, and run history. A workspace persists across browser restarts.

### 7.2 Participant

A participant is one message source or target in a workspace. V1 participant types: `User`, `Gpt`, `Gemini`, `Grok`, `Claude`, `System`.

### 7.3 Binding

A binding connects a participant to an actual browser tab and provider conversation. It stores:

- Provider ID, current tab ID, window ID, provider URL/origin.
- Conversation reference (if extractable from the provider DOM).
- Capability state, health state.
- Last seen timestamp, last successful capture timestamp.

### 7.4 Message

A message is the canonical normalized turn record. Required fields:

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID | Unique identifier. |
| `workspace_id` | UUID | Parent workspace. |
| `participant_id` | ParticipantId | Source participant. |
| `role` | Enum | `User`, `Assistant`, `System`. |
| `round` | Option\<u32\> | Round number if part of a run. |
| `timestamp` | ISO 8601 | When the message was captured. |
| `body_text` | String | Normalized plain text content. |
| `body_blocks` | Vec\<Block\> | Structured block representation (paragraphs, code fences, lists, headings, tables, quotes). |
| `source_binding_id` | Option\<BindingId\> | Which binding captured this. |
| `dispatch_id` | Option\<DispatchId\> | If this message was produced by a dispatched send. |
| `raw_capture_ref` | Option\<String\> | Reference to the raw DOM capture. |
| `tags` | Vec\<String\> | User-applied labels. |

### 7.5 Run

A run is one orchestrated execution session inside a workspace. It has: run ID, workspace ID, graph type/preset, participant set, edge policies snapshot, timing policy, barrier policy, stop policy, lifecycle state (`Created`, `Running`, `Paused`, `Completed`, `Aborted`), and created/started/completed timestamps.

### 7.6 Round

A round is one orchestration cycle within a run. In broadcast or roundtable modes, a round completes when all active providers have received input and produced a response (or timed out). In relay mode, a round completes when the chain has fully executed.

### 7.7 Routing Graph

The routing graph is the canonical execution model underneath all orchestration modes. Nodes are participants. Directed edges define who can forward context to whom. Every orchestration mode compiles down to a specific graph topology.

### 7.8 Edge Policy

An edge policy controls one source→target relationship. Each edge policy includes:

- Whether the edge is enabled.
- Initial catch-up rule (full history, last N, specific range, pinned summary, none).
- Incremental rule (unseen delta only, last response only, sliding window, full history, manual).
- Self-exclusion rule (exclude target's own prior assistant turns by default).
- Template choice.
- Inclusion/exclusion of user turns, system notes, pinned summaries.
- Truncation behavior.
- Priority.
- Approval requirement (auto-send vs. require user confirmation).

### 7.9 Delivery Cursor

A delivery cursor records the latest source message already delivered to a specific target on a specific edge. This is the critical primitive for:

- Catch-up once, then delta-only forwarding.
- No duplicate resends.
- Re-entry after pause.
- Restart recovery.
- Route inspection.

The user can: inspect cursor position, reset a cursor, force a fresh catch-up, mark a message range as already delivered, temporarily freeze cursor advancement.

### 7.10 Dispatch

A dispatch is one rendered outbound package sent to one target. It stores: dispatch ID, run ID, round number, target participant, source message set (by ID), rendered payload (the exact text injected), send timestamp, capture-complete timestamp, outcome (`Delivered`, `Timeout`, `Error`, `Skipped`), error detail, and retry/backoff state.

### 7.11 Export Profile

A saved preset for transcript export or copy. Stores scope, filters, format, metadata configuration, filename template, body layout, and copy-vs-file preference. See §14.

---

## 8. Architecture

### 8.1 High-Level Components

1. **Background coordinator** — Chrome: service worker. Firefox: background scripts/page. Responsibility: orchestration state machine, tab/binding registry, permissions state, scheduling, retries, recovery, storage coordination.

2. **Provider adapter content scripts** — One adapter boundary per provider. Responsibility: DOM probe, transcript capture, composer injection, send action, completion detection, error detection.

3. **Extension workspace UI** — Extension page usable in side panel/sidebar or full tab. Responsibility: workspace management, composer, route policy editing, run controls, exports, diagnostics.

4. **Shared Rust core** — Orchestration engine, canonical data model, routing graph, rendering engine, export engine, cursor logic.

### 8.2 Rust Workspace Organization

| Crate | Compiles To | Runs In |
|-------|-------------|---------|
| `polyglot-core` | Wasm | Background service worker / script |
| `polyglot-common` | Library (linked, not standalone) | Shared by all other crates |
| `polyglot-ui` | Wasm | Popup / sidebar / tab page |
| `polyglot-adapter-gpt` | Wasm | Content script (ChatGPT tabs) |
| `polyglot-adapter-gemini` | Wasm | Content script (Gemini tabs) |
| `polyglot-adapter-grok` | Wasm | Content script (Grok tabs) |
| `polyglot-adapter-claude` | Wasm | Content script (Claude tabs) |
| `polyglot-export` | Library (linked into core) | Background, invoked by core |
| `polyglot-build` | Native binary (build-time only) | Developer machine |

### 8.3 Communication

All inter-component communication uses the WebExtension messaging API (`runtime.sendMessage`, `runtime.onMessage`, `runtime.connect` for long-lived ports). Content scripts have only a limited set of directly accessible extension APIs and must message the background coordinator for broader coordination. Message types are Rust enums serialized via `serde_json`.

---

## 9. Provider Adapter Contract

Each provider adapter is a Rust crate compiled to a Wasm module and injected as a content script. All adapters implement a common trait:

```rust
trait ProviderAdapter {
    // --- Identity ---
    fn codename(&self) -> &str;
    fn display_name(&self) -> &str;

    // --- Lifecycle ---
    /// Run a structural probe against the current DOM.
    /// Returns Err if the DOM structure is unrecognized.
    fn structural_probe(&self) -> Result<(), AdapterError>;

    /// Return the provider's current health/readiness status.
    fn health(&self) -> ProviderHealth;

    // --- Input ---
    /// Inject text into the provider's input field.
    fn inject_input(&self, text: &str) -> Result<(), AdapterError>;

    /// Trigger the provider's send/submit action.
    fn send(&self) -> Result<(), AdapterError>;

    // --- Output observation ---
    /// Whether the provider is currently generating a response.
    fn is_generating(&self) -> bool;

    /// Extract the latest assistant response from the DOM.
    fn extract_latest_response(&self) -> Result<Message, AdapterError>;

    /// Extract the full conversation history visible in the DOM.
    fn extract_full_history(&self) -> Result<Vec<Message>, AdapterError>;

    // --- Capabilities ---
    /// Whether this provider accepts follow-up input while still generating.
    fn supports_follow_up_while_generating(&self) -> bool;

    // --- State detection ---
    /// Detect login wall, rate limit, error banner, or other blocking states.
    fn detect_blocking_state(&self) -> Option<BlockingState>;

    /// Extract conversation ID or title if visible in the DOM.
    fn conversation_ref(&self) -> Option<ConversationRef>;
}
```

### 9.1 Health States

Minimum health states each adapter must report:

`Disconnected`, `Ready`, `Composing`, `Sending`, `Generating`, `Completed`, `PermissionMissing`, `LoginRequired`, `DomMismatch`, `Blocked`, `RateLimited`, `SendFailed`, `CaptureUncertain`, `DegradedManualOnly`.

### 9.2 Adapter Versioning and Structural Probes

Each adapter declares a version stamp for its DOM selector logic. On load, the adapter runs its structural probe (checks for known landmark elements). If the probe fails, the adapter reports `DomMismatch` and Polyglot disables orchestration for that provider with a precise diagnostic message. Adapter selector updates ship as point releases without touching core logic.

### 9.3 Capture and Normalization

Adapters capture transcript state into the canonical message model. Normalization preserves (best-effort): paragraphs, bullet and numbered lists, headings, code fences, inline code, block quotes, and tables. Forwarding always reinjects normalized plain text, never executable HTML. Non-text content in v1 is stored as placeholders or references, not re-forwarded.

The adapter supports three capture granularities: full history, incremental delta (since a given message ID), and latest message only. When extraction confidence is low, the adapter marks the capture as uncertain.

---

## 10. Orchestration Modes

All orchestration modes compile to the same underlying engine:

1. Choose targets.
2. Resolve active source set.
3. Resolve edge policies.
4. Select messages from canonical log using delivery cursors.
5. Apply catch-up/delta rules.
6. Render package with selected template.
7. Preview/edit if required by the mode.
8. Inject and send.
9. Observe completion.
10. Capture and normalize response.
11. Advance delivery cursors.
12. Evaluate stop conditions.
13. Continue or end.

### 10.1 Manual Modes

**Broadcast.** One user-authored prompt is sent to any selected subset of providers simultaneously. Responses are collected and displayed in the unified view as they arrive.

**Directed.** One target provider receives selected context from other participants plus the user's input. This explicitly supports the pattern:

```
<gpt-response>
...
</gpt-response>

<grok-response>
...
</grok-response>

<gemini-response>
...
</gemini-response>

<user-input>
...
</user-input>
```

The target provider's own latest assistant turn is excluded from the context preamble by default. Override is available for new-conversation or self-comparison workflows.

**Relay-to-One.** Selected outputs from one or more participants are packaged and sent to one target.

**Relay-to-Many.** One rendered package is duplicated and sent to several targets.

**Draft-Only / Copy-Only.** The package is rendered and previewed but not sent. The user can copy it to clipboard. Useful as a fallback when adapter send is broken, or when the user prefers manual paste.

### 10.2 Autonomous Modes

**Roundtable (Full Mesh).** Each active provider receives the newest eligible messages from all other active providers. All sends happen simultaneously. The round completes per the active barrier policy.

**Moderator / Jury.** One designated provider receives the others' outputs and synthesizes, critiques, ranks, or arbitrates. Other providers do not see each other directly — only the moderator's output is forwarded back.

**Relay Chain.** Providers are ordered in a user-defined sequence. Output from provider A becomes input to provider B, whose output becomes input to provider C, and so on. One complete chain traversal is one round.

**Moderated Autonomous.** Any autonomous mode can be run in moderated fashion: the engine pauses after each round for user review. Between rounds, the user may edit the next outbound package, inject a user message, add or remove participants, and change target sets and edge policies for the next step. If the workflow requires a materially different orchestration preset or graph topology (e.g., transitioning from a broadcast phase to a targeted refinement phase to a synthesis phase), the workspace supports launching that next phase as a new run without losing transcript history or delivery cursor state.

### 10.3 Barrier Policies

- **Wait for all:** Round completes only when every active provider has responded or timed out.
- **Quorum:** Round completes when a specified subset of providers have responded.
- **First finisher:** Round completes as soon as any one provider responds.
- **Manual advance:** Round never auto-completes; the user explicitly advances.

### 10.4 Timing Controls

- Per-provider generation timeout (default: 120 seconds).
- Per-provider cooldown between sends (default: 2 seconds).
- Inter-round delay in autonomous modes (default: 5 seconds).
- Jitter: ±20% randomization on all delays.
- Exponential backoff after repeated provider failures.
- Maximum concurrent sends.
- Maximum rounds per run.
- Global run timeout.

### 10.5 Stop Conditions

Runs may stop on any combination of: maximum rounds reached, manual pause/stop, sentinel phrase detected, repeated provider failure, repeated timeout, low-delta/stagnation heuristic, manual approval required and denied.

### 10.6 Follow-Up-Aware Providers

Providers that accept new input while still generating (e.g., Claude) expose this as a capability via the adapter trait. Default behavior is barrier-safe (wait for completion). A per-provider per-run override allows non-blocking follow-ups.

---

## 11. Context and Routing Policies

### 11.1 Workspace Default vs. Edge Override

Each workspace defines a default context strategy. Each edge may override that strategy.

### 11.2 Initial Catch-Up Policies (per edge)

- Full history from selected sources.
- Last N messages from selected sources.
- Specific selected message range.
- Pinned manual summary.
- No automatic catch-up.

### 11.3 Incremental Policies (per edge)

- Unseen delta only (using delivery cursors).
- Last response only.
- Sliding window (last N messages).
- Full history every time.
- Manual-only selection.

### 11.4 Self-Exclusion Rule

When composing a payload for target provider X, the system excludes X's own latest assistant message by default. Overrides exist for: new conversation on the target, explicit self-comparison workflows, and manual inclusion.

### 11.5 Inclusion Controls (per edge)

The user controls whether to include: user-authored turns, system notes, pinned summaries, moderator annotations, and the target provider's own prior turns.

### 11.6 Soft Limits and Truncation

Each target may have a soft character-count limit. When a payload exceeds it, the system warns the user and optionally auto-trims oldest eligible content or swaps older content for a pinned summary.

---

## 12. Prompt Packaging and Templates

### 12.1 Built-In Template Families

V1 ships with: XML-like wrapped blocks (the `<provider-response>...</provider-response>` pattern), Markdown section blocks (headers with provider names), and minimal plain-text labels.

### 12.2 Built-In Preamble Templates

V1 ships with: Neutral (no preamble, just context tags), Collaboration, Debate, Review/Critique, and Synthesis.

### 12.3 Custom Templates

Users can create, save, and manage custom packaging templates. Minimum template variables: provider codename, provider display name, role, timestamp, round number, conversation title (when available), model label (when available), message ID, message body, user note, workspace name.

### 12.4 Preview and Edit

Every rendered outbound package is previewable before send in manual and moderated modes. The user can: edit the full package, edit only the user note, remove individual included messages, swap template without rebuilding the selection, and copy the package to clipboard instead of sending.

---

## 13. Transcript Capture and Normalization

See §9.3 for adapter-level capture requirements.

At the workspace level, the system maintains a canonical, provider-agnostic message log. Messages from all providers are normalized into a common schema. The log supports: full-text search, filtering by participant/role/run/round, and per-message inspection of both normalized and raw captured content.

Forwarded content is always normalized plain text. No provider HTML is ever re-injected or executed.

---

## 14. Export and Copy System

Export is a core feature.

### 14.1 Supported Export Scopes

- Entire workspace canonical transcript.
- A single provider's transcript (including related user messages).
- A single run.
- One or more selected rounds.
- One or more selected messages (arbitrary selection).
- Provider-only subset (exclude user messages).
- Dispatch/payload log subset.
- Diagnostic event subset.

### 14.2 Selection Controls

The user can select messages by: participant, role, checkbox/direct pick, round range, time range, run, delivery status (sent/received/timeout/error), tags, or search results. Selections can be inverted.

### 14.3 Output Formats

V1 supports:

- **Markdown** — with optional TOML front matter, configurable body layout (chronological, grouped-by-round, grouped-by-participant), optional message headings, code fence preservation, optional metadata appendix/footer.
- **JSON** — canonical machine-readable interchange format with schema version, metadata object, participant map, messages array, and optional run/dispatch/diagnostic arrays.
- **TOML** — human-readable structured format with equivalent fields.

All formats are rendered from a single canonical export model defined in Rust.

### 14.4 Copy Actions

The system supports copy-to-clipboard as an alternative to file download for all export formats, as well as: copy rendered prompt package, copy raw normalized message text, copy single provider view. All copy flows originate from the UI layer (extension page or sidebar), triggered by user gestures.

### 14.5 Metadata and Front Matter

Exported files include configurable metadata. Minimum toggleable fields: workspace name/ID, export title, export timestamp (local timezone and/or UTC), scope type, selected participants, orchestration mode, run ID, round range, message count, template used, context strategy snapshot, edge-policy snapshot, provider conversation references (when available), model labels (when available), browser name, extension version, export profile name, tags/notes, diagnostics summary, raw payload inclusion flag.

Metadata is placeable in: filename, front matter, top heading, footer, inline message labels, or metadata appendix.

### 14.6 Filename Templates

The export engine supports a simple templating system for filenames using named specifier keys (workspace name, date, provider, format, etc.) with fallback defaults, slugification, array joining, simple date/time formatting, truncation, and omission of empty values.

### 14.7 Export Profiles

Users can save and reuse export profiles. A profile stores: scope preset, selection filters, output format, body layout, metadata include flags, filename template, and copy-vs-file preference.

---

## 15. Unified View (Extension UI)

This section specifies functional requirements only. All visual design is deferred to a designer.

### 15.1 Workspace Management

Create, rename, duplicate, archive, delete, and export workspaces. Clear all local data.

### 15.2 Provider Binding and Discovery

Bind to existing provider tabs, open new provider tabs, rebind after tab reload, mark missing tabs as disconnected, list multiple candidate tabs for a provider and let the user choose.

### 15.3 Active Workspace View

- Canonical message log with source attribution.
- Provider status indicators (health states from §9.1).
- Current orchestration mode and context strategy display.
- Round counter and run status.

### 15.4 Composer

- Single-target and multi-target send.
- Manual message picking for context.
- Pinned notes and per-target notes.
- Draft-only and copy-only modes.
- Template switching.
- Full rendered-package preview with editing.

### 15.5 Run Controls

Start, pause, resume, step (advance one round), stop, and abort. Disable a participant mid-run. Edit next outbound package. Change target sets and edge policies between rounds in moderated mode. Launch a new run in the same workspace (inheriting transcript history and cursor state) when a phase transition requires a different orchestration topology.

### 15.6 Message Inspection

Click any message to see: the raw payload sent to the provider (including all context tags), the raw response as extracted from the DOM, and metadata (round, timestamp, character count, dispatch linkage, cursor state).

### 15.7 Search and Filtering

Full-text search across the workspace. Saved filters. Provider-specific transcript views. Unseen-since-last-delivery views. Run history filtering.

### 15.8 Keyboard Shortcuts

Bindable via the browser's `commands` API: open/focus workspace, send current draft, pause/resume run, step next round, cycle target provider, toggle provider active/inactive, copy current package.

---

## 16. Diagnostics, Degraded Modes, and Recovery

### 16.1 Adapter Diagnostics

The product exposes actionable diagnostics: provider unreachable, host permission missing, login required, DOM mismatch, transcript root not found, input element not found, send action not found, completion detection uncertain, provider error state, rate limit detected.

Each diagnostic event is persisted with timestamp, severity, code, and detail.

### 16.2 Degraded Modes

If automatic send is not possible but message composition still works, the system falls back to:

- **Draft-only mode:** Package is composed but not sent.
- **Copy-only mode:** Package is rendered and copied to clipboard.
- **Manual-send mode:** The system highlights the target tab and the user pastes manually.

### 16.3 Recovery After Browser Restart

On restart, the system: reloads workspaces and messages from IndexedDB, reloads runs and dispatch logs, attempts rebinding to provider tabs, marks unbound participants as disconnected, avoids duplicate re-sends by checking delivery cursors and dispatch outcomes, and leaves autonomous runs paused. Autonomous runs never auto-resume after browser restart unless the user has explicitly enabled resumable automation.

Because Chrome service workers are ephemeral, recovery and idempotency are required by design, not optional polish.

---

## 17. Data Model

Minimum persisted entities with key fields.

### Workspace
`id`, `name`, `created_at`, `updated_at`, `enabled_providers`, `default_mode`, `default_context_strategy`, `default_template_id`, `active_export_profile_ids`, `tags`, `notes`.

### ParticipantBinding
`id`, `workspace_id`, `provider_id`, `tab_id`, `window_id`, `origin`, `conversation_ref`, `health_state`, `capability_snapshot`, `last_seen_at`.

### Message
See §7.4 for full field table.

### Run
`id`, `workspace_id`, `mode`, `graph_snapshot`, `participant_set`, `barrier_policy`, `timing_policy`, `stop_policy`, `status`, `started_at`, `ended_at`.

### Round
`id`, `run_id`, `round_number`, `started_at`, `completed_at`, `status`.

### Dispatch
`id`, `run_id`, `round_id`, `target_participant_id`, `source_message_ids`, `template_id`, `rendered_payload`, `sent_at`, `captured_at`, `outcome`, `error_detail`, `retry_count`.

### EdgePolicy
`id`, `workspace_id`, `source_participant_id`, `target_participant_id`, `catch_up_policy`, `incremental_policy`, `self_exclusion`, `include_user_turns`, `include_system_notes`, `truncation_policy`, `approval_mode`.

### DeliveryCursor
`id`, `workspace_id`, `source_participant_id`, `target_participant_id`, `last_delivered_message_id`, `last_delivered_at`.

### Template
`id`, `workspace_id`, `kind`, `name`, `version`, `body_template`, `preamble`, `metadata_template`, `filename_template`.

### ExportProfile
`id`, `workspace_id`, `name`, `scope_preset`, `filter_preset`, `format`, `layout`, `include_flags`, `filename_template`, `metadata_template`.

### DiagnosticEvent
`id`, `workspace_id`, `binding_id`, `timestamp`, `level`, `code`, `detail`, `snapshot_ref`.

---

## 18. Security and Privacy

- The product is local-first. No data is transmitted to any server.
- No provider API keys, cookies, or authentication tokens are accessed, stored, or forwarded.
- Content scripts run in an isolated world and interact only with the provider page DOM, never the page's JS context.
- All provider-originated content is treated as untrusted text. No provider HTML is executed.
- All executable runtime code is packaged locally.
- Provider access is requested per-provider at runtime via optional host permissions.
- A global automation kill switch allows the user to immediately halt all orchestration activity.
- Workspace deletion and full local data clearing are supported.

---

## 19. Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Provider DOM drift | Strict adapter isolation, structural probes, degraded manual modes, fast patch releases. |
| Permission friction | Per-provider runtime requests with clear just-in-time explanations. |
| Prompt bloat | Delivery cursors, per-edge catch-up/delta rules, soft limits, truncation, pinned summaries. |
| Infinite or low-value loops | Max rounds, stagnation detection, approval gates, hard stop controls, duplicate prevention. |
| Browser lifecycle differences | Persisted state as source of truth, resumable coordinator, no reliance on global in-memory state. |
| Store policy / anti-abuse friction | Conservative defaults, explicit user control, transparent automation settings, policy review before store submission. |

---

## 20. Engineering Requirements

### 20.1 Build Outputs

The build produces: Chrome extension package, Firefox extension package, source maps for local debugging, and reproducible version metadata.

### 20.2 Testing

The project shall include:

- Unit tests for routing, cursor advancement, and export rendering logic.
- Golden tests for rendered packages and export output.
- Adapter contract tests (mock DOM fixtures).
- Fixture-based DOM extraction tests per provider.
- Restart/recovery tests (simulated browser restart with persisted state).
- Duplicate-send prevention tests.
- Cursor advancement tests.

### 20.3 Performance

The system remains usable with long transcripts, multiple workspaces, and concurrent provider monitoring. Incremental capture is preferred over full-DOM rescans. IndexedDB operations should be asynchronous and non-blocking to the UI thread.

---

## 21. Delivery Phases

### Phase 1 — Foundation and Manual Orchestration

- Workspace creation and persistence (IndexedDB + storage.local).
- Provider adapter framework and structural probes.
- Content script injection and binding lifecycle.
- Transcript mirroring and normalization.
- Manual broadcast.
- Manual directed send with context packaging.
- Manual relay-to-one and relay-to-many.
- Package preview and edit.
- Built-in templates and preambles.
- Export/copy system with Markdown, JSON, TOML.
- Per-provider runtime host permission flow.
- Sidebar and tab-page UI.

### Phase 2 — Routing Engine and Autonomous Runs

- Delivery cursors and edge policies.
- Catch-up-then-delta-only routing.
- Roundtable / full mesh autonomous mode.
- Moderator / jury autonomous mode.
- Relay chain autonomous mode.
- Moderated autonomous mode (pause between rounds).
- Barrier policies (all, quorum, first, manual).
- Timing controls (cooldowns, backoff, timeouts, jitter).
- Stop conditions.
- Run ledger and recovery.
- Degraded modes and diagnostics.

### Phase 3 — Advanced Operator Features

- Saved export profiles with filename templates.
- Saved route-graph presets and scripted multi-phase orchestration recipes (e.g., independent drafts → conflict map → targeted refinement → synthesis → final verification).
- Advanced search and filtering.
- Pinned summaries and compaction helpers.
- Richer diagnostics and adapter self-tests.
- Stagnation/convergence heuristic.
- Workspace import.

---

## 22. Acceptance Criteria

1. The extension installs and runs on current stable desktop Chrome and Firefox.
2. All four provider adapters pass their structural probes on the current provider web UIs.
3. The user can send one prompt to any subset of providers from the workspace and see all responses in the unified view.
4. The user can send a directed message to one provider containing context from the others, with the target's own prior assistant turns excluded by default.
5. The system supports "full catch-up once, then unseen delta only" on a per-edge basis using persisted delivery cursors.
6. The system can run an autonomous roundtable with configurable barrier policy and max-round stop condition.
7. The user can pause a run, edit the next outbound package, and resume.
8. The system persists messages, runs, dispatches, and cursor state across browser restarts without duplicate auto-resends.
9. The user can export a whole workspace transcript as Markdown, JSON, or TOML.
10. The user can export a single provider transcript or an arbitrary message selection including both provider and user messages.
11. The user can save and reuse an export profile with filename specifiers and configurable metadata/front matter fields.
12. The user can copy any rendered export or prompt package to clipboard.
13. All executable runtime code is packaged locally; no remote JS or Wasm.
14. The codebase is Rust-first and Wasm-first, with only thin bootstrap/bridge code outside Rust.
15. Chrome and Firefox builds preserve equivalent core behavior despite different extension shells and background contexts.
16. A global kill switch immediately halts all orchestration activity.

---

*End of PRD.*
