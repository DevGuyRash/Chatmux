# Adding E2E Tests

Use this guide when extending the Playwright suite. The goal is to keep tests small, focused, and resilient to upstream UI churn.

## Core Rules

- Organize tests by user-visible flow or surface first.
- Create a new spec file when a test covers a different flow family, provider contract, or launcher path.
- Extend an existing focused file when the new coverage belongs to the same flow family, the same provider contract, or the same launcher concern.
- Keep spec files assertion-focused. Move bootstrap, detection, and selector churn into support modules.
- Do not add unrelated tests to an existing spec just because the browser or provider matches.
- Do not recreate a generic `helpers.js` catch-all file.
- Prefer declarative support APIs over procedural step piles in specs.

## Required Structure

The suite should grow along this shape:

```text
e2e/
  support/
    chrome-extension.js
    firefox-launcher.js
    providers/
      chatgpt.js
  shell/
    chrome.spec.js
  chatgpt/
    dom-anchors.spec.js
    roundtrip.spec.js
  firefox/
    launcher.spec.js
```

Add a new file instead of extending an unrelated one when:

- a test introduces a new user-visible flow
- a test introduces a second provider's DOM contract
- a spec needs browser-specific branching in the test body
- a helper starts owning both browser bootstrap and provider DOM logic

Extend an existing file when:

- the new assertions stay within the same user-visible flow
- the new selectors or URL rules belong to the same provider contract owner
- the new launcher checks are still about the same browser launcher surface

## Where Logic Belongs

Put logic in the narrowest owner:

- `support/chrome-extension.js`
  Chrome extension bootstrap, artifact checks, persistent context fixture, extension page discovery
- `support/firefox-launcher.js`
  Firefox manifest reads, `web-ext` prerequisites, launcher support reporting
- `support/providers/<provider>.js`
  Provider URL matching, selector fallbacks, provider surface detection
- `*.spec.js`
  Test intent, assertions, attachments, and skip rules only

Specs must not embed provider selector arrays inline. If selectors are likely to churn because an upstream web UI changes, centralize them in the provider support module.

Prefer declarative support surfaces such as `detect<Provider>Surface()` or launcher status objects over long procedural spec setup. Specs should read like intent plus assertions, not like a hand-written browser macro.

## Locator Policy

Use locators in this order:

1. Role, label, text, and ARIA for Chatmux-owned UI
2. Manifest- or runtime-derived extension facts
3. Provider-specific fallback selectors isolated in one provider module

For upstream-owned pages like ChatGPT:

- keep selectors in ordered fallbacks
- return a structured surface description instead of asserting raw selectors everywhere
- attach the detected surface to the test run so failures are diagnosable

## File Size and Scope

Keep files small and single-purpose:

- A spec file should cover one flow family.
- A support file should own one setup domain or one provider contract.
- If a file needs multiple unrelated `describe` blocks, split it.
- If a helper export list becomes hard to scan, split the module by ownership.

Preferred examples:

- `shell/chrome.spec.js` for extension shell smoke
- `chatgpt/dom-anchors.spec.js` for ChatGPT adapter contract checks
- `chatgpt/roundtrip.spec.js` for guarded live ChatGPT send/read/ingest checks
- `firefox/launcher.spec.js` for Firefox packaging and launcher prerequisites

Avoid:

- one browser-wide spec for all Chrome behavior
- one provider-wide spec for every provider and flow
- one helper that bootstraps Chrome, checks Firefox, and knows all provider selectors

## Checklist for a New Test

Before adding a test:

- choose the user-visible flow you are covering
- decide whether an existing spec truly owns that flow
- extend the existing focused spec if the flow still matches its ownership
- create a new spec file if the flow does not match an existing one
- reuse an existing support module only if it already owns that setup or provider contract
- add a new support module instead of widening one past its current ownership
- add or update run commands only if discoverability changes
- document any new environment variables in `e2e/README.md`

## Concrete Examples

Good:

- Add `e2e/gemini/dom-anchors.spec.js`
- Add `e2e/support/providers/gemini.js`

Good:

- Extend `chatgpt/dom-anchors.spec.js` when adding another assertion about the same ChatGPT DOM contract
- Extend `support/providers/chatgpt.js` when the ChatGPT selector contract changes and the same owner should remain SSOT

Bad:

- Add Gemini selectors to `support/providers/chatgpt.js`
- Add Gemini assertions to `chatgpt/dom-anchors.spec.js`

Good:

- Add `e2e/shell/workspace-creation.spec.js` when workspace creation becomes testable

Bad:

- Append workspace creation, routing, templates, diagnostics, and provider checks to `shell/chrome.spec.js`
