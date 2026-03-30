Cross-device Firefox/Chrome validation of the workspace resolution and provider tab-binding fixes is pending.
Local Playwright extension verification remains blocked here because headed Chromium requires X11 and headless Chromium is timing out before the extension service worker appears.
Diagnostics now uses structured envelopes and dedicated snapshot queries, but true app-scoped diagnostics and blob-backed artifact storage are still incomplete; global mode currently aggregates workspace-scoped records only.
Live ChatGPT roundtrip in CDP-attach mode still cannot execute end-to-end in the user's current Chrome session because Chatmux is not installed there; the spec now correctly reaches that skip instead of falsely requiring `CHATMUX_E2E_CHROME_USER_DATA_DIR`.
ChatGPT provider control now exists behind a generic control-plane contract, but the ChatGPT backend is still DOM-strategy-first; validated authenticated network discovery and per-operation network fallback remain incomplete.
ChatGPT project selection and project creation are grounded on live DOM structure, but fresh conversation creation currently means landing on the project composer and letting the next send create the thread; explicit server-confirmed title assignment is not implemented yet.
ChatGPT reasoning support is currently inferred and mapped through known model IDs (`instant`/`thinking`/`pro`); feature-flag control such as auto-switch only works when the relevant ChatGPT configure surface is already open.
ChatGPT richer thought-trace capture beyond visible summary rows remains incomplete.
WHEN manual browser QA needs a signed-in session THEN use the persistent Chrome stable profile under `~/.config/google-chrome` or the Firefox default-release profile under `~/.mozilla/firefox/n8xi91uc.default-release-1735434875083`, not a Playwright temp profile.
WHEN the target profile is already locked or active THEN avoid a second launch against that profile and reuse the existing browser window/session instead.
WHEN using `CHATMUX_E2E_CHROME_CDP_URL` to attach to an already-running Chrome THEN the live roundtrip still requires Chatmux to already be installed in that browser session.
