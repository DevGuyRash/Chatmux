---
name: chatmux-frontend
description: >-
  REQUIRED when any part of the task touches Chatmux UI code, CSS, or Leptos
  components. Enforces the Chatmux design system: CSS variable discipline,
  spacing token usage, component reuse from primitives, theme awareness (dark
  and light), and provider identity color system. Covers: (1) Writing or
  modifying CSS in chatmux-ui, (2) Creating or styling Leptos components,
  (3) Adding colors, spacing, or visual properties, (4) Working with provider
  theming. If the task touches chatmux-ui visual output, use this skill.
metadata:
  author: rashino
  version: "1.0.0"
---

# Chatmux Frontend Development

You SHALL follow the Chatmux design system for all visual work in `chatmux-ui`.
This skill supersedes the generic `frontend-design` skill within this repository.

---

## 1. Design System Files

The canonical sources of truth for the design system:

| File | Purpose |
|---|---|
| `chatmux-ui/assets/css/tokens.css` | All CSS custom properties: colors, typography, spacing, radii, shadows, motion, z-index. Dark and light theme values. |
| `chatmux-ui/assets/css/base.css` | Global resets, font defaults, scrollbar styling, focus indicators, reduced-motion support, keyframe animations. |
| `chatmux-ui/assets/css/utilities.css` | Utility classes built from tokens (typography, spacing, layout, surfaces, transitions). |
| `chatmux-ui/assets/css/animations.css` | Keyframe animations and transition classes per DESIGN.md section 7. |
| `context/DESIGN.md` | Authoritative design specification. |

WHEN you need to understand the visual system THEN you SHALL read these files before writing CSS or inline styles.

---

## 2. CSS Variable Discipline

WHEN writing any **color** value (background, text color, border color, shadow color, fill, stroke) THEN you SHALL use a `var(--*)` token from `tokens.css`. You SHALL NOT hardcode hex (`#fff`), `rgb()`, `rgba()`, `hsl()`, or named color values.

WHEN writing any **spacing** value (margin, padding, gap, width offsets, height offsets, top/left/right/bottom) THEN you SHALL use `var(--space-*)` tokens. You SHALL NOT write raw `px`, `rem`, or `em` values for spacing.

WHEN writing **font** properties THEN you SHALL use `var(--type-*)` and `var(--font-*)` tokens.

WHEN writing **border-radius** THEN you SHALL use `var(--radius-*)` tokens.

WHEN writing **box-shadow** THEN you SHALL use `var(--shadow-*)` tokens.

WHEN writing **transition/animation** durations or easing THEN you SHALL use `var(--duration-*)` and `var(--easing-*)` tokens.

WHEN writing **z-index** THEN you SHALL use `var(--z-*)` tokens.

**Exception:** Fixed structural dimensions (e.g., `width: 56px` for the nav rail, `min-height: 56px` for the header bar) that define layout geometry rather than visual style MAY use literal values. These are architectural constants, not themeable properties.

---

## 3. Token Lookup Before Creation

BEFORE introducing any new CSS custom property, you SHALL read `tokens.css` and confirm no existing token serves the purpose.

WHEN a new token is genuinely needed THEN you SHALL:
1. Add it to `tokens.css` in the correct section.
2. Define values in **both** the dark theme block (`:root` / `[data-theme="dark"]`) and the light theme block (`[data-theme="light"]`).
3. Follow the existing naming conventions (`--surface-*`, `--text-*`, `--border-*`, `--provider-*-*`, `--status-*-*`, etc.).

You SHALL NOT create component-local CSS variables that duplicate token-level concerns.

---

## 4. Theme Awareness

ALL styles MUST work in both `[data-theme="dark"]` (default) and `[data-theme="light"]` modes.

WHEN adding any color token THEN you SHALL define values in both theme blocks.

You SHALL NOT use colors that only look correct in one theme.

WHEN the change affects colors THEN you SHALL verify visual correctness in both themes by toggling `data-theme` on the `<html>` element.

---

## 5. Primitive Component Reuse

The following primitive components exist in `chatmux-ui/src/components/primitives/`:

| Component | Variants / Notes |
|---|---|
| `button` | Primary, Secondary, Danger, Ghost, Icon; sizes Small, Medium, Large |
| `badge` | Default, Accent, Success, Warning, Error, Info, Neutral |
| `text_input` | Standard text input with label and error states |
| `text_area` | Multi-line text input |
| `number_input` | Numeric input with increment/decrement |
| `toggle` | On/off switch |
| `checkbox` | Checkbox with label |
| `dropdown` | Select dropdown |
| `segmented_control` | Tab-like segment selector |
| `chip` | Selectable pill with optional prefix content |
| `tooltip` | Hover/focus tooltip |
| `toast` | Status notifications (Success, Info, Warning, Error, Provider) |
| `modal` | Dialog with backdrop scrim and focus trap |
| `skeleton` | Loading placeholder with shimmer |
| `empty_state` | Empty content placeholder with icon, heading, description |
| `error_banner` | Error display banner |
| `collapsible` | Accordion expand/collapse |
| `icon` | SVG icon component with ~60 icon variants |

WHEN building a new component THEN you SHALL compose from these primitives rather than reimplementing button, input, badge, or modal behavior with raw HTML elements.

WHEN a primitive does not support a needed variant THEN you SHALL extend the existing primitive rather than creating a parallel component.

You SHALL NOT use raw `<button>` elements with ad-hoc inline styling when the `Button` primitive covers the use case. The same applies to `<input>`, `<select>`, and other form elements.

---

## 6. Provider Identity System

Each AI provider has four color tokens:

```
--provider-{name}-solid    Main brand color
--provider-{name}-muted    Low-opacity background tint
--provider-{name}-text     Legible text color
--provider-{name}-border   Border accent
```

The six providers: `gpt`, `gemini`, `grok`, `claude`, `user`, `system`.

Provider-specific components in `chatmux-ui/src/components/provider/`:
- `provider_icon.rs` — Brand mark at variable size
- `provider_dot.rs` — Status indicator dot (running pulse, paused static)
- `provider_chip.rs` — Colored badge with provider name
- `provider_status_row.rs` — Row with provider name, health, and status
- `health_badge.rs` — Health state indicator with spring animation

WHEN displaying provider-attributed content THEN you SHALL use `var(--provider-{name}-*)` tokens and the provider components above. You SHALL NOT hardcode provider colors.

---

## 7. Architecture Context

Chatmux UI is a **Leptos 0.7** (Rust/Wasm) application compiled as a browser extension. Key architectural facts:

- **Styling approach:** Plain CSS files (`tokens.css`, `base.css`, `utilities.css`, `animations.css`) plus inline `style=` strings in Leptos components. No CSS preprocessor, no CSS-in-Rust framework.
- **Inline styles are normal:** Variant-dependent styling uses `format!()` with `var(--*)` tokens in inline style strings (see `button.rs`, `badge.rs`, `chip.rs`). This is the established pattern.
- **Utility classes:** Components use CSS classes from `utilities.css` for layout and typography (e.g., `class="flex flex-col items-center gap-4 p-6 surface-raised"`).
- **Animations:** State transitions use CSS class toggling (e.g., `class:nav-rail-item--active`), not JavaScript animation libraries.
- **Responsive:** Two layout modes (Sidebar <500px, FullTab >=500px) detected via ResizeObserver, not CSS media queries.
- **Theme switching:** Via `data-theme` attribute on `<html>`, managed by `ThemeProvider` in `chatmux-ui/src/theme/mod.rs`.

WHEN writing inline style strings in Leptos components THEN you SHALL still use `var(--*)` tokens within those strings.
