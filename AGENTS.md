# Chatmux Agent Instructions

This repo is a Rust workspace with an extension runtime, a UI crate, and a top-level Playwright e2e harness. Keep instructions here durable and agent-facing. Task-specific detail belongs in the more focused docs under `e2e/` and `context/`.

## State Files

Operational state belongs in `state.md`, not in changelogs or handoff notes.

WHEN a `.local*/context/` directory exists THEN you SHALL use it. ELSE IF a `context/` directory exists at the repo root THEN you SHALL use that. ELSE you SHALL create `context/` at the repo root.

`state.md` should stay short and only describe active work, unresolved risks, or incomplete decisions.

WHEN something is done and working THEN you SHALL remove it from `state.md`.

## Command Surface

Prefer the repo’s existing wrappers instead of inventing new command entrypoints. The main surfaces today are `justfile` for build/package flows and `package.json` for Playwright/web-ext flows.

Use these first when they match the task:

- `just doctor`
- `just setup`
- `just ci`
- `just dist-chrome`
- `just dist-firefox`
- `just package-chrome`
- `just package-firefox`
- `npx playwright test`
- `npx playwright test e2e/shell/chrome.spec.js`
- `CHATMUX_E2E_OPEN_CHATGPT=1 npx playwright test e2e/chatgpt/dom-anchors.spec.js`
- `CHATMUX_E2E_FIREFOX=1 npx playwright test e2e/firefox/launcher.spec.js`
- `npm run run:e2e:firefox`

## Playwright

The e2e suite is feature-first. Browser/bootstrap concerns live under `e2e/support/`, provider-owned selector contracts live under `e2e/support/providers/`, and user-visible flows live under `e2e/<flow>/`.

Keep specs small, focused, and idempotent. They should read like intent plus assertions, not like long browser macros. Prefer declarative support surfaces for page-shape detection, selector fallback ownership, and launcher facts.

WHEN you add or change Playwright coverage THEN you SHALL update the tests in the same change.

WHEN a change stays within the same flow family, provider contract, or launcher concern THEN you SHOULD extend the existing focused file.

WHEN a new test would add unrelated behavior to an existing spec THEN you SHALL create a new spec file instead.

WHEN upstream UI selectors are brittle THEN you SHALL isolate them in one provider-owned support module and treat that module as the single source of truth.

WHEN you add selectors, locators, or URL matching for a provider page THEN you SHALL centralize them in one place and reuse them from specs.

WHEN a helper starts owning multiple unrelated concerns, such as browser bootstrap plus provider selector ownership, THEN you SHALL split it.

The source of truth for e2e authoring rules is `e2e/ADDING_TESTS.md`. The quickstart and current command surface live in `e2e/README.md`.

## Frontend Design System

The UI crate (`chatmux-ui`) uses a token-based design system defined in `chatmux-ui/assets/css/tokens.css`. The authoritative design specification is `context/DESIGN.md`.

WHEN you write or modify CSS or inline styles THEN you SHALL use `var(--*)` tokens from `tokens.css`. You SHALL NOT hardcode hex colors, raw px spacing, or literal font sizes.

WHEN you need a color, spacing value, radius, shadow, duration, or z-index THEN you SHALL check `tokens.css` first. IF no suitable token exists THEN you SHALL add one in the correct section with both dark and light theme values.

WHEN you add or change visual output THEN you SHALL verify it works in both `[data-theme="dark"]` and `[data-theme="light"]`.

WHEN you need a button, badge, input, toggle, modal, tooltip, or other primitive UI element THEN you SHALL use or extend the existing components in `chatmux-ui/src/components/primitives/` rather than creating parallel implementations.

WHEN displaying provider-attributed content THEN you SHALL use `var(--provider-{name}-*)` tokens and the provider components in `chatmux-ui/src/components/provider/`.

The complete frontend discipline is owned by the `chatmux-frontend` skill.

## Maintaining This File

This file should describe durable agent policy, not every current implementation detail.

WHEN repo conventions, command surfaces, test architecture, or durable workflow expectations change THEN you SHALL update this file in the same change.

WHEN a rule can be expressed as an ownership pattern, decision rule, or maintenance invariant THEN you SHALL prefer that over hard-coded inventories.

WHEN the detail is task-specific or contributor-facing rather than agent policy THEN you SHOULD put it in the narrower doc and link to it from here.

## Change Discipline

Re-read files before editing them. Work with existing user changes rather than flattening them.

You SHALL NOT revert user changes you did not make.

WHEN you encounter friction, failing tests, or unexpected behavior THEN you SHALL log it in the repo’s friction diagnostics workflow.
