# Chatmux

Chatmux is a local-first Chrome and Firefox extension for orchestrating live
ChatGPT, Gemini, Grok, and Claude conversations from one workspace.

It uses the provider sessions already open in the browser. Chatmux does not
proxy prompts through a backend and does not require provider API keys.

## Product surface

- Broadcast or direct one message to any ready subset of providers.
- Preview and edit the exact per-target package before it is committed.
- Route full catch-up context once, then advance persisted delta cursors only
  after delivery is acknowledged.
- Run roundtable, moderator/jury, relay-chain, and moderated autonomous flows
  with barriers, timing controls, pause/step/resume, review, and stop policies.
- Inspect canonical messages, structured content blocks, dispatch outcomes,
  provider health, and reconstructible sent payloads.
- Search with reusable filters; save templates, export profiles, route presets,
  orchestration recipes, and pinned summaries.
- Export or copy whole workspaces, provider transcripts, runs, rounds, or exact
  message selections as Markdown, JSON, or TOML.
- Archive, duplicate, export, and import complete local workspaces.

The UI supports both the browser sidebar and a full extension tab, including
compact/full-width layouts, light/dark themes, keyboard focus, reduced motion,
and a global orchestration kill switch.

## Architecture

Canonical state, routing, packaging, run transitions, persistence contracts,
exports, diagnostics, and provider DOM contracts live in Rust and ship as
local Wasm. Leptos renders the extension UI. The shared JavaScript runtime is
an effect boundary for WebExtension APIs, provider-tab messaging, and bounded
DOM readiness/completion observation; it does not own canonical product state.
IndexedDB is authoritative for workspaces and ledgers, while `storage.local`
holds lightweight settings and restart markers.

See [context/architecture.md](context/architecture.md) for the component and
runtime ownership map, and [context/project_overview.md](context/project_overview.md)
for the complete PRD.

## Toolchain policy

The repository MSRV is Rust 1.88, including the separately locked
`chatmux-ui` crate. Day-to-day and CI builds use the exact toolchain pinned in
`rust-toolchain.toml`. Trunk and wasm-pack are pinned by `just install-tools`
and checked by `just doctor`.

## Build and qualify

```bash
just setup
just ci
just package-all
just verify-packages
```

Every staged extension contains deterministic `build-metadata.json` with the
source and artifact fingerprints used by launch and package qualification.
The unpacked builds are written to `extension-dist/chrome` and
`extension-dist/firefox`; release ZIPs are written to `extension-packages`.

Useful focused gates:

```bash
cargo test --workspace --locked
cargo test --manifest-path chatmux-ui/Cargo.toml --locked
npm run test:extension
npm run test:launcher
npm run test:e2e:app
CHATMUX_E2E_FIREFOX=1 npm run test:e2e:firefox
just lint
just ui-lint
just lint-firefox-extension
```

The default Playwright application suite uses a clean product-owned profile.
Authenticated provider canaries and live sends are isolated into explicit
projects so deterministic CI cannot accidentally submit third-party prompts.
See [e2e/README.md](e2e/README.md) for the test boundaries and retained failure
evidence.

## Launch Chrome safely

```bash
npm run launch
```

This rebuilds Chatmux and loads it into a dedicated `.local/e2e` Chrome
profile. `npm run relaunch` reuses that profile and `npm run fresh` replaces
only that dedicated profile. A signed-in profile can be selected explicitly
with `CHATMUX_E2E_CHROME_USER_DATA_DIR`; Chatmux never copies or deletes an
explicit user profile and fails fast if Chrome has locked it.

Provider pages can change without notice. The adapters therefore combine
versioned semantic DOM contracts, structural probes, visible/actionable control
selection, mutation-driven capture, and a bounded stable-response fallback.
Transient send-control races are retried only after input injection; dispatch
and cursor state remains authoritative in Rust, so a browser or worker restart
cannot silently duplicate a pending send.
