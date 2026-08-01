# Adding E2E Tests

Use the lowest test tier that can prove the product contract. Pure Rust logic, bridge response shapes, adapter parsing, manifest facts, and deterministic DOM fixtures belong in unit or integration tests. Playwright is for browser extension behavior, real rendering, browser lifecycle, and cross-system provider workflows.

## Directory Ownership

```text
e2e/
  app/                    deterministic product-owned GUI journeys
  shell/                  extension install/render/navigation smoke
  provider-canary/        third-party semantic selector contracts
  provider-live/          authenticated provider outcomes and TODO contracts
  chatgpt/                existing ChatGPT canary/live compatibility paths
  firefox/                Firefox package/launcher contracts
  support/
    chrome-extension.js   browser bootstrap and extension fixture
    browser-diagnostics.js
    workspace.js          deterministic bridge arrangement
    provider-canary.js
    providers/
      provider-surface.js
      chatgpt.js
      claude.js
      gemini.js
      grok.js
```

Create a new focused spec when a test introduces a different user-visible flow, provider contract, or browser lifecycle concern. Do not grow a single live spec into setup, project management, model controls, send, sync, recovery, and cleanup at once.

## Journey Rules

- Arrange through a stable bridge/API when setup is not the behavior under test.
- Act through the GUI when the product affordance is the behavior under test.
- Assert a user-visible or persisted product outcome, not incidental copy or layout.
- Use `test.step()` for meaningful phases.
- Generate unique run tokens for shared provider/backend state.
- Clean up live workspace/provider state in `finally` when the product supports cleanup.
- Live sends run with one worker and zero retries to avoid duplicate side effects.
- Do not use `waitForTimeout()` or arbitrary sleeps. Use web-first assertions or `expect.poll()` with a named outcome and last-state diagnostics.

## Selector Rules

For Chatmux-owned UI, prefer roles and labels. Placeholder locators are a fallback when the product does not expose a stronger accessible name. Use a test ID only as an intentionally stable product hook.

For provider-owned UI:

- raw selectors belong only in that provider's support module;
- semantic target names describe intent (`composer`, `sendButton`, `transcript`, `assistantMessage`, `generating`);
- each candidate records counts and validation in a selector report;
- a canary must prove it found the intended element, not merely any matching node;
- `unknown` is a page-understanding failure, never a skip;
- known login/rate/challenge states may be valid only for a test explicitly about that state.

Do not combine selector-canary assertions with live journey outcomes. Canaries never click Send.

## Failure Artifacts

Meaningful failures should be debuggable without an immediate rerun. Preserve:

- trace and screenshot;
- sanitized console/pageerror/service-worker diagnostics;
- URL, title, and classified provider state;
- semantic target and candidates tried;
- matched candidate, match counts, and validation facts;
- last observed state from bounded polls;
- a safe workspace snapshot when relevant.

Do not attach full third-party DOM or unrelated transcripts.

## Qualification Backlog

When a real product contract is not implemented, add `test.fixme()` only in the explicit qualification/live surface with a precise outcome-oriented title. Do not add a passing placeholder, and do not put known failing product contracts in the deterministic default project merely to document them.
