# Chatmux Architecture

Chatmux is a local-first WebExtension. The browser is the integration runtime;
there is no Chatmux server and no provider API transport. Prompts and responses
move between the extension and the provider tabs already authenticated by the
user.

## Runtime ownership

| Layer | Implementation | Owns |
|---|---|---|
| Canonical model and protocol | `chatmux-common` (Rust) | Workspaces, bindings, messages, structured blocks, runs, rounds, dispatches, cursors, policies, templates, profiles, diagnostics, and serialized command/event contracts. |
| Coordinator | `chatmux-core` (Rust/Wasm) | State transitions, routing, context selection, exact package construction, approvals, run lifecycle, stop conditions, idempotent delivery acknowledgement, recovery, and persistence decisions. |
| Provider adapters | `chatmux-adapter-*` (Rust/Wasm) | Provider-specific semantic DOM contracts, structural probes, transcript extraction, control discovery, normalization, blocking states, and capability snapshots. |
| UI | `chatmux-ui` (Rust/Leptos/Wasm) | Sidebar/full-tab presentation, command intent, previews/editors, navigation, accessibility, themes, and user-gesture clipboard/download flows. |
| Export | `chatmux-export` (Rust) | Scope selection, canonical export document, Markdown/JSON/TOML rendering, front matter, and filename specifiers. |
| Browser effect runtime | `extension-src/common/background.js` | WebExtension API compatibility, tab discovery/focus/injection, content-script transport, provider I/O execution, bounded readiness/completion observation, and event delivery to Rust/Wasm. |
| Content bootstraps | `extension-src/common/content-*.js` | Mutation observation, lazy adapter-Wasm loading, and forwarding commands/events between the page DOM and the Rust adapter. |
| Persistence | IndexedDB plus `storage.local` | IndexedDB stores canonical heavy entities; extension storage holds lightweight preferences, defaults, kill-switch state, and recovery markers. |

The JavaScript effect runtime has no independent workspace, route, cursor, or
run store. It executes browser effects from Rust-produced commands and reports
observable outcomes back through explicit delivered, failed, and captured
acknowledgements. Delivery cursors advance only inside the Rust coordinator
after a delivered acknowledgement.

## Browser split

Chrome MV3 runs the shared background module as an ephemeral service worker.
Firefox MV3 runs the same module as a module background script. Browser-specific
manifests and shell APIs are kept at the staging boundary; the core Wasm,
provider adapters, UI, protocol, and persistence schema are shared.

Every core command is reconstructible from persisted state. Startup recovery
loads storage, pauses any run that was persisted as running, preserves pending
dispatches without resending them, and exposes resume markers. Rebinding a
provider restores ready health only after the new binding is persisted.

## Provider send and capture flow

1. The UI sends a typed command to the background runtime.
2. The Rust coordinator validates the command, renders exact target packages,
   and persists pending dispatches.
3. The browser effect runtime resolves the bound provider tab and verifies the
   current conversation reference and blocking state.
4. The provider adapter injects the exact stored payload. Send-control discovery
   uses visible/actionable candidates and a bounded retry for transient UI races.
5. A successful click produces a delivered acknowledgement. Capture uses the
   adapter's completion signal, with a bounded stable-response fallback when a
   provider's stop control is absent or its DOM changes responsively.
6. Captured normalized messages are linked to the dispatch and persisted by the
   Rust coordinator. Failures remain failed and never advance delivery cursors.

## Packaging and qualification

`xtask` stages deterministic Chrome and Firefox trees, validates required local
resources and manifest parity, fingerprints sources and artifacts, and packages
versioned ZIPs. Runtime code is entirely package-local and extension CSP permits
only self-hosted script/Wasm execution.

The test pyramid separates pure Rust behavior, JavaScript bridge heuristics,
deterministic extension GUI journeys, provider selector canaries, serialized live
provider sends, Firefox package/launcher smoke, and final package verification.
Live provider tests are opt-in and have zero retries to prevent duplicate sends.
