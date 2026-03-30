# Chatmux Frontend CSS Guide

**Companion to:** `context/DESIGN.md` (design spec), `chatmux-ui/assets/css/` (implementation)
**Audience:** Any agent or contributor modifying Chatmux UI code

This document describes the CSS implementation discipline: what layers exist, when to use each, and what to avoid. The design *intent* lives in `DESIGN.md`; this guide covers the *how*.

---

## CSS Architecture

Four CSS files, loaded in this order (`chatmux-ui/index.html`):

| File | Purpose | When to edit |
|------|---------|--------------|
| `tokens.css` | CSS custom properties (`--surface-*`, `--space-*`, `--type-*`, etc.) | When adding a new semantic token |
| `base.css` | Resets, element defaults, focus indicators, scrollbars | Rarely — global element behavior |
| `utilities.css` | Single-property utility classes (`.flex`, `.mb-3`, `.border-b`) | When adding a new utility |
| `components.css` | Multi-property semantic classes (`.surface-card`, `.code-block`) | When extracting a repeated multi-property pattern |
| `animations.css` | Keyframes and component animation classes | When adding motion |

---

## The Golden Rule: No Hardcoded Values

Every color, spacing value, radius, shadow, font size, duration, and z-index MUST come from a `var(--*)` token. Never write raw hex colors, `px` spacing, or literal font sizes.

```rust
// WRONG
style="color: #8b5cf6; padding: 8px 16px; font-size: 12px;"

// RIGHT
class="text-link py-2 px-4 type-caption"
```

If no token exists for what you need, add one to `tokens.css` in the correct section with both dark and light theme values.

---

## Decision Tree: How to Style Something

When you need to apply a visual property, follow this order:

### 1. Use a utility class (preferred)

Single-property classes from `utilities.css`. Compose them in the `class` attribute.

```rust
// Spacing, borders, text, display, layout
<div class="flex items-center gap-3 py-4 px-6 border-b">

// Typography + color
<span class="type-caption text-tertiary micro-label">

// Margin
<h3 class="type-title text-primary mb-3">
```

**Available utilities:**
- **Layout:** `.flex`, `.flex-col`, `.flex-1`, `.grid`, `.items-center`, `.justify-between`, `.flex-wrap`
- **Spacing (gap):** `.gap-{0..10}`
- **Padding (all):** `.p-{0..10}`
- **Padding (axis):** `.px-{0..10}`, `.py-{0..10}`
- **Padding (side):** `.pt-{0..6}`, `.pb-{0..6}`, `.pl-{0..6}`, `.pr-{0..6}`
- **Margin:** `.mb-{0..8}`, `.mt-{0..6}`, `.ml-{0..4}`, `.mr-{0..4}`, `.mx-auto`
- **Border:** `.border`, `.border-b`, `.border-t`, `.border-l`, `.border-r` (all use `--border-subtle`)
- **Radius:** `.rounded-{none,sm,md,lg,xl,full}`
- **Typography:** `.type-display`, `.type-title`, `.type-subtitle`, `.type-body`, `.type-body-strong`, `.type-caption`, `.type-caption-strong`, `.type-code`, `.type-code-small`, `.type-label`
- **Text color:** `.text-primary`, `.text-secondary`, `.text-tertiary`, `.text-inverse`, `.text-link`, `.text-code`
- **Surface:** `.surface-base`, `.surface-raised`, `.surface-overlay`, `.surface-sunken`
- **Display:** `.hidden`, `.block`, `.inline-block`, `.inline-flex`
- **Text:** `.text-center`, `.text-right`, `.text-left`, `.uppercase`, `.whitespace-pre-wrap`, `.break-words`, `.font-mono`, `.truncate`
- **Overflow:** `.overflow-hidden`, `.overflow-auto`, `.overflow-x-auto`, `.overflow-y-auto`
- **Shadow:** `.shadow-{none,sm,md,lg,xl}`
- **Misc:** `.cursor-pointer`, `.select-none`, `.pointer-events-none`, `.sr-only`

### 2. Use a component class

Multi-property semantic classes from `components.css` for patterns that recur across components.

```rust
// Card container (sunken bg + border + radius)
<div class="surface-card p-4">

// Code block (pre-wrap + word-break + overflow + padding + radius)
<pre class="code-block surface-card type-code-small">

// Uppercase section label
<span class="type-caption text-tertiary micro-label">"SORT BY"</span>

// Section headers
<div class="section-header flex items-center gap-3">
<div class="section-header-compact flex items-center gap-3">

// Separator lines
<div class="divider-h" />
<div class="divider-v" />
```

### 3. Use a layout primitive component

Leptos components in `chatmux-ui/src/components/primitives/` that wrap the CSS classes with type-safe props and slots.

```rust
use crate::components::primitives::{
    divider::Divider,
    surface::{Surface, SurfaceVariant},
    code_block::CodeBlock,
    section_header::SectionHeader,
};

// Separator
<Divider />                     // horizontal
<Divider vertical=true />       // vertical (between toolbar items)

// Card container
<Surface class="flex items-center gap-3 p-4".to_string()>
    {children()}
</Surface>
<Surface variant=SurfaceVariant::Raised>  // raised variant
    {children()}
</Surface>

// Code display
<CodeBlock>{raw_json}</CodeBlock>

// Screen header with back button
<SectionHeader title="Settings".to_string() on_back=Box::new(move || go_back()) />
```

### 4. Use an existing leaf primitive

For interactive elements: `Button`, `Badge`, `Chip`, `Toggle`, `TextInput`, `NumberInput`, `Dropdown`, `SegmentedControl`, `Modal`, `Tooltip`, `Checkbox`, `Skeleton`, `EmptyState`, `ErrorBanner`, `Toast`.

All live in `chatmux-ui/src/components/primitives/`. Use or extend these — never create parallel implementations.

### 5. Inline style (last resort)

Only use `style=` for:

- **Reactive values** that depend on Leptos signals: `style=move || format!("transform: rotate({}deg)", if open.get() { 90 } else { 0 })`
- **Dynamic values** computed at construction time from props: `style=format!("color: {};", provider.text_color())`
- **Truly one-off values** with no reuse potential: `style="max-width: 280px;"`, `style="grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr);"`

Even in these cases, extract what you can to classes and keep only the dynamic part inline:

```rust
// WRONG — everything inline
style="padding: var(--space-4); border-bottom: 1px solid var(--border-subtle); background: var(--surface-raised);"

// RIGHT — static parts in classes, nothing inline
class="p-4 border-b surface-raised"

// RIGHT — mix when necessary (dynamic + static)
class="flex items-center gap-3 p-4 rounded-md"
style=move || format!("background: {};", if active.get() { "var(--surface-selected)" } else { "transparent" })
```

---

## Theme Awareness

Every visual change must work in both `[data-theme="dark"]` and `[data-theme="light"]`. The token system handles this automatically when you use `var(--*)` tokens. If you hardcode a color, it will break in one theme.

---

## Adding New Patterns

When you find yourself writing the same multi-property inline style 3+ times:

1. Extract it as a class in `components.css`
2. Optionally wrap it as a Leptos component in `primitives/` if it benefits from typed props or slots
3. Register the component in `primitives/mod.rs`

When you need a single-property utility that doesn't exist:

1. Add it to the appropriate section in `utilities.css`
2. Follow the existing naming convention (`.{property}-{value}`)

---

## File Quick Reference

| Need | Where |
|------|-------|
| Design intent and token roles | `context/DESIGN.md` |
| Token values (CSS variables) | `chatmux-ui/assets/css/tokens.css` |
| Utility classes | `chatmux-ui/assets/css/utilities.css` |
| Component classes | `chatmux-ui/assets/css/components.css` |
| Layout primitives | `chatmux-ui/src/components/primitives/{divider,surface,section_header,code_block}.rs` |
| Leaf primitives | `chatmux-ui/src/components/primitives/*.rs` |
| Provider components | `chatmux-ui/src/components/provider/*.rs` |
