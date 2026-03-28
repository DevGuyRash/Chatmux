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
- `firefox/launcher.spec.js`
  Verifies Firefox packaging metadata and launcher prerequisites without pretending full Playwright attachment exists yet.

## Current Constraints

The repo does not yet have an automation-safe signed-in provider profile. Your real Chrome and Firefox profiles are live and locked, so this harness does **not** attach to them.

The ChatGPT smoke path is guarded:

- By default, it skips unless a ChatGPT tab already exists in the automation browser.
- Set `CHATMUX_E2E_OPEN_CHATGPT=1` to let the spec open `https://chatgpt.com/` in the temp automation profile.

Firefox now has a working launcher path, but not a full attached browser test path:

- the staged Firefox extension can be launched reliably with `web-ext`
- the bundled Playwright Firefox binary works as the launcher target
- Playwright still does not have a stable attachment path into that launched Firefox extension session

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

Run the guarded Firefox launcher prerequisite spec:

```bash
CHATMUX_E2E_FIREFOX=1 npx playwright test e2e/firefox/launcher.spec.js
```

Launch the staged Firefox extension directly:

```bash
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
- Firefox coverage is launcher-plus-manifest coverage today. Live Playwright control of the launched Firefox extension session is still the remaining gap.
