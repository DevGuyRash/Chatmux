# Chatmux Playwright E2E

The harness has separate deterministic application, provider-selector, and live-provider boundaries. The default command deliberately runs only product-owned deterministic coverage plus the Firefox packaging contract. Authenticated provider work must be requested explicitly.

## Suite Boundaries

| `CHATMUX_E2E_SUITE` | Playwright project | Purpose |
|---|---|---|
| `app` (default) | `app`, `firefox-contract` | Fresh-profile Chrome extension GUI journeys and deterministic Firefox package facts. |
| `provider-canary` | `provider-canary` | Non-sending semantic selector contracts against signed-in ChatGPT, Claude, Gemini, and Grok pages. |
| `provider-live` | `provider-live` | Serialized real-provider outcomes. Live sends have no retries. |
| `qualification` | all projects | Full release-qualification surface, including explicit unfinished-contract annotations. |

The project boundary is part of the safety model. Do not run fresh-state application specs against a user's live CDP profile, and do not add live sends to the deterministic `app` project.

## Run

Install the lockfile-pinned tools once:

```bash
npm ci
```

Run the deterministic default:

```bash
npx playwright test
```

Run the four signed-in selector canaries without sending:

```bash
CHATMUX_E2E_SUITE=provider-canary \
CHATMUX_E2E_CHROME_CDP_URL=http://127.0.0.1:9222 \
CHATMUX_E2E_EXTENSION_ID=your-installed-chatmux-id \
npx playwright test
```

Run the live-provider project:

```bash
CHATMUX_E2E_SUITE=provider-live \
CHATMUX_E2E_CHROME_USER_DATA_DIR=/path/to/dedicated-automation-profile \
CHATMUX_E2E_CHROME_CHANNEL=chrome \
CHATMUX_E2E_OPEN_PROVIDERS=1 \
npx playwright test
```

Run the full qualification surface:

```bash
CHATMUX_E2E_SUITE=qualification npx playwright test
```

Package scripts may wrap these commands, but `CHATMUX_E2E_SUITE` remains the canonical selector understood by `playwright.config.js`.

## Provider Setup

Provider canaries require `ready` authenticated surfaces. Missing tabs, login pages, rate limits, challenges, and unknown pages do not silently pass a required canary.

The harness first claims an already-open matching tab. Set `CHATMUX_E2E_OPEN_PROVIDERS=1` to open missing provider landing pages, or use an individual flag:

- `CHATMUX_E2E_OPEN_CHATGPT=1`
- `CHATMUX_E2E_OPEN_CLAUDE=1`
- `CHATMUX_E2E_OPEN_GEMINI=1`
- `CHATMUX_E2E_OPEN_GROK=1`

Provider landing URLs can be overridden with `CHATMUX_E2E_<PROVIDER>_URL`.

Chrome attachment and profile options:

- `CHATMUX_E2E_CHROME_CDP_URL` attaches to a Chrome debugging endpoint.
- `CHATMUX_E2E_EXTENSION_ID` identifies Chatmux when its MV3 service worker is asleep in an attached session.
- `CHATMUX_E2E_CHROME_USER_DATA_DIR` launches a persistent profile after checking that it is not locked.
- `CHATMUX_E2E_CHROME_PROFILE_DIRECTORY` selects a named profile within that user-data directory.
- `CHATMUX_E2E_CHROME_CHANNEL=chrome` exercises stable Chrome instead of bundled Chromium.
- `CHATMUX_E2E_CHROME_EXECUTABLE_PATH` selects an explicit browser binary.

Use a dedicated automation profile for repeatable live tests. The harness never deletes a configured persistent profile.

## Canary Contract

Every provider module owns:

- URL recognition and landing-page navigation;
- explicit page states: `ready`, `loading`, `login_required`, `not_found`, `error`, `permission_required`, `challenge`, `rate_limited`, `blocked`, and `unknown`;
- ordered semantic targets and validation;
- a selector report containing candidates tried, counts, visibility, validation, and the matched candidate.

Canaries resolve the provider's exact accessible composer, place a unique disposable probe in an initially empty composer, verify the exact send action in the same composer surface, never click it, then clear and verify the composer is empty. Claude and Grok use focused select-all/backspace cleanup because an empty `fill()` is not a reliable clear contract on those contenteditables.

Provider-owned selectors stay in `e2e/support/providers/`; specs must not embed raw provider selectors.

## Failure Evidence

Playwright retains a trace and screenshot on failure. The extension fixture also records sanitized:

- extension and provider console messages;
- page exceptions and crashes;
- context web errors;
- observed service-worker lifecycle events.

Unexpected error-level events from the Chatmux extension origin fail an otherwise passing test. Set `CHATMUX_E2E_ALLOW_EXTENSION_ERRORS=1` only for a deliberate diagnostic run.

Artifacts are written under:

- `.local/playwright-results/`
- `.local/playwright-report/`
- `.local/playwright-junit.xml` in CI

Provider attachments intentionally avoid full DOM and transcript dumps. They contain safe URL/title/state facts and semantic selector reports.

## Firefox

`firefox-contract` verifies staged manifest and launcher prerequisites. It does not claim live Firefox UI control. `run:e2e:firefox` remains a manual launcher path until a stable attached Firefox automation workflow exists.

## Required Package State

Application and launched-profile tests consume `extension-dist/chrome`. They fail or skip with an artifact-specific reason if the UI or required Wasm files are absent. A release qualification command must rebuild the UI and all Wasm packages before staging the extension; artifact existence alone is not proof of freshness.
