# Chatmux Playwright E2E

This harness exercises the packaged Chrome extension under `extension-dist/chrome` and keeps Firefox coverage honest under `extension-dist/firefox`.

## Suite Layout

The suite is feature-first:

```text
e2e/
  ADDING_TESTS.md
  support/
    chrome-extension.js
    firefox-launcher.js
    providers/
      chatgpt.js
  shell/
    chrome.spec.js
    navigation.spec.js
  chatgpt/
    dom-anchors.spec.js
    roundtrip.spec.js
  firefox/
    launcher.spec.js
```

Use [ADDING_TESTS.md](./ADDING_TESTS.md) when adding coverage. That document is the source of truth for file splitting, helper ownership, and selector policy.

## What It Covers

- `shell/chrome.spec.js`
  Verifies the packaged extension shell renders in a deterministic full-tab Chrome layout.
- `shell/navigation.spec.js`
  Verifies workspace creation/filtering and the mounted Routing, Templates, Diagnostics, Settings, and Active Workspace screens.
- `chatgpt/dom-anchors.spec.js`
  Optionally inspects a ChatGPT tab in the automation browser and checks the DOM anchors the GPT adapter depends on.
- `chatgpt/roundtrip.spec.js`
  Guarded roundtrip that sends from the extension composer, watches ChatGPT receive the prompt and answer, and then checks that the extension ingests the ChatGPT reply. Use a signed-in browser profile for this one.
- `firefox/launcher.spec.js`
  Verifies Firefox packaging metadata, launcher prerequisites, and packaged ChatGPT content-script coverage without pretending full Playwright attachment exists yet.

## Current Constraints

The repo does not yet have an automation-safe signed-in provider profile. Your real Chrome and Firefox profiles are live and locked, so this harness does **not** attach to them.

The ChatGPT smoke path is guarded:

- By default, it skips unless a ChatGPT tab already exists in the automation browser.
- Set `CHATMUX_E2E_OPEN_CHATGPT=1` to let the spec open the configured ChatGPT URL in the automation profile.
- Set `CHATMUX_E2E_CHATGPT_URL=https://chatgpt.com/` to override the default ChatGPT landing URL. This is useful when you want the harness to land on a specific ChatGPT surface instead of the default home page.
- Set `CHATMUX_E2E_CHROME_USER_DATA_DIR=/path/to/chrome-user-data` to reuse a real Chrome user-data directory for signed-in ChatGPT runs.
- Set `CHATMUX_E2E_CHROME_PROFILE_DIRECTORY='Profile 1'` when the reused Chrome user-data directory contains multiple named profiles.
- Set `CHATMUX_E2E_CHROME_CHANNEL=chrome` or `CHATMUX_E2E_CHROME_EXECUTABLE_PATH=/path/to/chrome` if you need the harness to launch a specific Chrome binary instead of the bundled Chromium.
- If the requested profile is already locked, the harness fails fast instead of launching a second browser into the same data dir.
- Set `CHATMUX_E2E_MANUAL_LOGIN=1` to keep the roundtrip test waiting for a ready ChatGPT surface while you log in in the opened browser window.
- Set `CHATMUX_E2E_MANUAL_LOGIN_TIMEOUT_SECS=600` to control how long that manual-login wait lasts.

Firefox now has a working launcher path, but not a full attached browser test path:

- the staged Firefox extension can be launched reliably with `web-ext`
- the bundled Playwright Firefox binary works as the launcher target
- Playwright still does not have a stable attachment path into that launched Firefox extension session
- You can point `CHATMUX_E2E_FIREFOX_BINARY` at Firefox or Zen and `CHATMUX_E2E_FIREFOX_PROFILE` at a real profile for manual launcher-based QA

## Run

```bash
npm install
npx playwright test
```

Run only the Chrome shell smoke:

```bash
npx playwright test e2e/shell/chrome.spec.js
```

Run the guarded ChatGPT DOM check:

```bash
CHATMUX_E2E_OPEN_CHATGPT=1 npx playwright test e2e/chatgpt/dom-anchors.spec.js
```

Run the guarded ChatGPT roundtrip:

```bash
CHATMUX_E2E_OPEN_CHATGPT=1 npx playwright test e2e/chatgpt/roundtrip.spec.js
```

Run the guarded ChatGPT roundtrip against a signed-in local Chrome profile:

```bash
CHATMUX_E2E_OPEN_CHATGPT=1 \
CHATMUX_E2E_CHROME_USER_DATA_DIR="$HOME/.config/google-chrome" \
CHATMUX_E2E_CHROME_PROFILE_DIRECTORY="Default" \
CHATMUX_E2E_CHROME_CHANNEL=chrome \
npx playwright test e2e/chatgpt/roundtrip.spec.js
```

Run the guarded ChatGPT roundtrip and wait while you log in manually:

```bash
CHATMUX_E2E_OPEN_CHATGPT=1 \
CHATMUX_E2E_CHROME_USER_DATA_DIR="$HOME/.config/google-chrome" \
CHATMUX_E2E_CHROME_PROFILE_DIRECTORY="Default" \
CHATMUX_E2E_CHROME_CHANNEL=chrome \
CHATMUX_E2E_MANUAL_LOGIN=1 \
npx playwright test e2e/chatgpt/roundtrip.spec.js
```

Run the guarded Firefox launcher prerequisite spec:

```bash
CHATMUX_E2E_FIREFOX=1 npx playwright test e2e/firefox/launcher.spec.js
```

Launch the staged Firefox extension directly:

```bash
npm run run:e2e:firefox
```

Launch the staged Firefox extension against a real Firefox or Zen profile for manual QA:

```bash
CHATMUX_E2E_FIREFOX_BINARY="/usr/bin/firefox" \
CHATMUX_E2E_FIREFOX_PROFILE="$HOME/.mozilla/firefox/your-profile" \
npm run run:e2e:firefox
```

Keep the Chrome automation browser or profile around for inspection:

```bash
CHATMUX_KEEP_BROWSER=1 CHATMUX_KEEP_PROFILE=1 npx playwright test e2e/shell/chrome.spec.js
```

## Notes

- Chatmux-owned UI should use semantic selectors first.
- Provider-owned DOM fallbacks belong in provider support modules, not in spec files.
- `package.json` pins the repo-root e2e toolchain so Playwright and `web-ext` are reproducible.
- The ChatGPT roundtrip spec requires a ready signed-in ChatGPT page. Logged-out, rate-limited, or challenge-gated surfaces skip rather than pretending the roundtrip passed.
- Firefox coverage is launcher-plus-manifest coverage today. Live Playwright control of the launched Firefox or Zen extension session is still the remaining gap, so real signed-in automated roundtrips should use Chrome or Chromium until that changes.
