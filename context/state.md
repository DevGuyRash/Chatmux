Cross-device Firefox/Chrome validation of the workspace resolution and provider tab-binding fixes is pending.
Diagnostics now uses structured envelopes and dedicated snapshot queries, but true app-scoped diagnostics and blob-backed artifact storage are still incomplete; global mode currently aggregates workspace-scoped records only.
Live signed-in Windows Chrome QA is pending because the `@chrome` plugin channel is not answering even though Chrome, the Codex Chrome Extension, and the native host are present; recovery requires user permission to open the selected-profile Chrome window and retry.
ChatGPT provider control now exists behind a generic control-plane contract, but the ChatGPT backend is still DOM-strategy-first; validated authenticated network discovery and per-operation network fallback remain incomplete.
ChatGPT project selection and project creation are grounded on live DOM structure, but fresh conversation creation currently means landing on the project composer and letting the next send create the thread; explicit server-confirmed title assignment is not implemented yet.
ChatGPT reasoning support is currently inferred and mapped through known model IDs (`instant`/`thinking`/`pro`); feature-flag control such as auto-switch only works when the relevant ChatGPT configure surface is already open.
ChatGPT richer thought-trace capture beyond visible summary rows remains incomplete.
WHEN continuing live ChatGPT or manual browser QA in this thread THEN use Windows Chrome through the `@chrome` plugin, not WSL-launched Chrome or Playwright-managed Chromium.
WHEN Chrome-plugin manual QA needs the signed-in ChatGPT session THEN claim the existing Windows Chrome ChatGPT tab instead of launching a new browser profile.
WHEN using `CHATMUX_E2E_CHROME_CDP_URL` to attach to an already-running Chrome THEN the live roundtrip still requires Chatmux to already be installed in that browser session.
