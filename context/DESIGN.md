# Chatmux — Design System & UI Specification

**Version:** 1.0.0
**Date:** 2026-03-25
**Status:** Draft
**Companion to:** `context/project_overview.md` (PRD v1.1.0)
**Scope:** Complete visual design, interaction design, and component specification. No code, no CSS, no technology choices.

---

## 1. Design Tokens

### 1.1 Color System

All colors are defined as semantic tokens. Each token has a light-theme value and a dark-theme value. Dark theme is the default. Concrete hex values are implementation decisions — this section defines the **roles and relationships**.

#### Surface Hierarchy

| Token | Role | Dark Theme Character | Light Theme Character |
|-------|------|---------------------|----------------------|
| `surface-base` | App background, outermost shell | Near-black with faint blue undertone | Off-white warm gray |
| `surface-raised` | Cards, panels, dialogs | Dark gray, ~8% lighter than base | White |
| `surface-overlay` | Dropdowns, tooltips, popovers | ~12% lighter than base | White with subtle shadow |
| `surface-sunken` | Inset areas: code blocks, input fields, inspector panes | ~4% darker than base | Light warm gray |
| `surface-hover` | Hover state overlay | White at 6% opacity, composited | Black at 4% opacity, composited |
| `surface-active` | Active/pressed state overlay | White at 10% opacity, composited | Black at 8% opacity, composited |
| `surface-selected` | Selected row, active tab, chosen item | Accent color at 12% opacity, composited | Accent color at 8% opacity, composited |

#### Text Hierarchy

| Token | Role |
|-------|------|
| `text-primary` | Body text, headings, primary labels |
| `text-secondary` | Metadata, timestamps, supporting labels |
| `text-tertiary` | Placeholders, disabled labels, decorative text |
| `text-inverse` | Text on filled accent backgrounds |
| `text-link` | Clickable text links |
| `text-code` | Inline code, monospace content |

Dark theme: primary is near-white (~90% luminance), secondary is ~60%, tertiary is ~40%.
Light theme: primary is near-black (~10% luminance), secondary is ~45%, tertiary is ~65%.

#### Border Hierarchy

| Token | Role |
|-------|------|
| `border-default` | Standard card/panel borders |
| `border-subtle` | Dividers, section separators |
| `border-strong` | Focused inputs, emphasized containers |
| `border-accent` | Active/selected borders using accent color |

Dark theme borders are light at low opacity (white 10–20%). Light theme borders are dark at low opacity (black 8–15%).

#### Accent and Brand

| Token | Role |
|-------|------|
| `accent-primary` | Primary action buttons, active indicators, selected states. A saturated blue-violet that reads as "orchestration" — not affiliated with any single provider. |
| `accent-primary-hover` | Hovered primary accent — slightly lighter/more saturated |
| `accent-primary-active` | Pressed primary accent — slightly darker |
| `accent-secondary` | Secondary actions, less prominent interactive elements. Desaturated variant of primary accent. |

#### Provider Identity Colors

Each provider gets a **hue family** with three tints: `solid` (for fills, badges, and strong identity), `muted` (for backgrounds and subtle containers), and `text` (for readable text on both surface-base and surface-raised).

| Provider | Hue Direction | Character |
|----------|--------------|-----------|
| `provider-gpt` | Green | OpenAI's established green. Warm green, not mint. |
| `provider-gemini` | Blue | Google's blue. Medium blue, not navy, not sky. |
| `provider-grok` | Orange | Warm orange to burnt orange. Distinct from any warning color. |
| `provider-claude` | Amber-tan | Anthropic's warm tan/terracotta. Not red, not brown — the warm amber-clay that Claude's branding uses. |
| `provider-user` | Neutral with accent tint | Uses `accent-primary` at reduced saturation. The user is not a "provider" — their color should feel like it belongs to the app, not to a brand. |
| `provider-system` | Neutral gray | Pure desaturated gray. System messages are informational, not branded. |

Each provider identity produces these tokens:
- `provider-{name}-solid` — badge fill, icon background, strong attribution
- `provider-{name}-muted` — message card tint, row highlight, subtle grouping
- `provider-{name}-text` — provider name labels, attribution text
- `provider-{name}-border` — card left-border accent, selected indicators

#### Status Colors

| Token | Role |
|-------|------|
| `status-success` | Healthy, completed, delivered | Green family |
| `status-warning` | Degraded, uncertain, approaching limits | Amber family |
| `status-error` | Failed, disconnected, aborted | Red family |
| `status-info` | Informational, neutral status | Blue family |
| `status-neutral` | Inactive, idle, not-yet-started | Gray family |

Each status color has `-solid`, `-muted`, `-text`, and `-border` variants following the same pattern as provider colors.

#### Diagnostic Severity Colors

Map directly to status colors:
- `Critical` → `status-error`
- `Warning` → `status-warning`
- `Info` → `status-info`
- `Debug` → `status-neutral`

### 1.2 Typography Scale

The extension uses two font stacks. Specific font family names are implementation decisions.

| Stack | Role |
|-------|------|
| `font-sans` | All UI text: labels, headings, body, metadata |
| `font-mono` | Code fences, raw payloads, template variables, dispatch IDs, cursor positions, JSON/TOML export preview |

#### Type Scale

| Token | Size | Weight | Line Height | Tracking | Usage |
|-------|------|--------|-------------|----------|-------|
| `type-display` | 20px | 600 | 1.3 | -0.01em | Workspace name in header (full-tab only) |
| `type-title` | 16px | 600 | 1.35 | -0.005em | Panel headings, dialog titles, section headers |
| `type-subtitle` | 14px | 600 | 1.4 | 0 | Sub-section headers, card titles |
| `type-body` | 13px | 400 | 1.5 | 0 | Message body text, descriptions, form labels |
| `type-body-strong` | 13px | 600 | 1.5 | 0 | Emphasized body text, active states |
| `type-caption` | 11px | 400 | 1.4 | 0.01em | Timestamps, metadata, badge labels, cursor positions |
| `type-caption-strong` | 11px | 600 | 1.4 | 0.01em | Status labels, participant names in caption context |
| `type-code` | 12px | 400 | 1.5 | 0 | Code blocks, raw payloads, template source, IDs |
| `type-code-small` | 11px | 400 | 1.4 | 0 | Inline code in captions, monospace metadata |
| `type-label` | 11px | 500 | 1.2 | 0.03em | Button labels, tab labels, form field labels (uppercase tracking for category labels only) |

In sidebar mode (360px), `type-display` drops to 17px and `type-title` to 15px. All other sizes remain unchanged — the scale is already optimized for compact contexts.

### 1.3 Spacing Scale

A base-4 spacing system. All spacing in the design uses only these values.

| Token | Value | Common Usage |
|-------|-------|-------------|
| `space-0` | 0px | — |
| `space-1` | 2px | Tight inline gaps, icon-to-label micro-gap |
| `space-2` | 4px | Badge internal padding, compact list item gaps |
| `space-3` | 6px | Inline element spacing, tight row padding |
| `space-4` | 8px | Standard internal padding, list item gaps, input padding-y |
| `space-5` | 12px | Card internal padding (compact), section gaps within a panel |
| `space-6` | 16px | Card internal padding (standard), panel padding |
| `space-7` | 20px | Section spacing, panel header padding |
| `space-8` | 24px | Panel-to-panel gap, major section breaks |
| `space-9` | 32px | Page-level vertical rhythm |
| `space-10` | 48px | Empty state vertical centering offsets |

### 1.4 Border Radii

| Token | Value | Usage |
|-------|-------|-------|
| `radius-none` | 0px | — |
| `radius-sm` | 3px | Badges, inline tags, small buttons |
| `radius-md` | 6px | Cards, inputs, dropdowns, message cards |
| `radius-lg` | 10px | Dialogs, panels, large containers |
| `radius-xl` | 16px | Floating action elements, pill shapes |
| `radius-full` | 9999px | Circular indicators, avatar-like elements, round buttons |

### 1.5 Elevation / Shadow

| Token | Character | Usage |
|-------|-----------|-------|
| `shadow-none` | No shadow | Default for all in-flow elements |
| `shadow-sm` | Tight, subtle, 1–2px offset | Hover state lifts on cards |
| `shadow-md` | Medium, 2–4px offset | Dropdowns, popovers, floating panels |
| `shadow-lg` | Diffuse, 4–12px offset | Dialogs, modal overlays |
| `shadow-xl` | Strong diffuse, 8–24px offset | Command palette, floating workspace switcher |

In dark theme, shadows are less visible — supplement with `border-subtle` for visual separation. In light theme, shadows carry more visual weight and borders can be lighter.

### 1.6 Transition and Animation

| Token | Duration | Easing | Usage |
|-------|----------|--------|-------|
| `duration-instant` | 50ms | — | Checkbox toggles, icon swaps |
| `duration-fast` | 100ms | ease-out | Hover color changes, focus rings |
| `duration-normal` | 200ms | ease-in-out | Panel collapse/expand, tab switches, state changes |
| `duration-slow` | 350ms | ease-in-out | Large panel transitions, layout shifts |
| `duration-gentle` | 500ms | ease-out | Message arrival entrance, toast entrance |
| `easing-enter` | — | cubic-bezier(0, 0, 0.2, 1) | Elements entering/appearing |
| `easing-exit` | — | cubic-bezier(0.4, 0, 1, 1) | Elements leaving/dismissing |
| `easing-standard` | — | cubic-bezier(0.4, 0, 0.2, 1) | General movement |
| `easing-spring` | — | cubic-bezier(0.34, 1.56, 0.64, 1) | Bouncy micro-interactions (toggle snaps, badge count changes) |

### 1.7 Z-Index Layers

| Token | Value | Usage |
|-------|-------|-------|
| `z-base` | 0 | Normal flow content |
| `z-raised` | 10 | Sticky headers, floating action bar |
| `z-dropdown` | 100 | Dropdowns, menus, selects |
| `z-overlay` | 200 | Panel overlays in sidebar mode |
| `z-modal` | 300 | Dialogs, confirmation modals |
| `z-toast` | 400 | Toast notifications |
| `z-tooltip` | 500 | Tooltips |

---

## 2. Provider Identity System

### 2.1 Provider Cards

Each provider is identified by a consistent triad: **color**, **icon**, and **label**.

| Provider | Label | Icon Description | Color Token Prefix |
|----------|-------|-----------------|-------------------|
| ChatGPT | "ChatGPT" | Rounded hexagonal aperture shape (the OpenAI mark) | `provider-gpt` |
| Gemini | "Gemini" | Four-pointed sparkle/star shape (the Gemini mark) | `provider-gemini` |
| Grok | "Grok" | Angular stylized "G" or lightning-like shape (the Grok/xAI mark) | `provider-grok` |
| Claude | "Claude" | Rounded warm organic shape (the Anthropic/Claude mark) | `provider-claude` |
| User | "You" | Simple person silhouette | `provider-user` |
| System | "System" | Gear/cog | `provider-system` |

**Provider names are never abbreviated in UI text.** Codenames (`gpt`, `gemini`, `grok`, `claude`) appear only in developer-facing contexts (diagnostics detail, raw payloads, template variables).

### 2.2 Where Provider Identity Appears

#### Message Attribution
- Provider icon (14×14px at body scale) + provider name in `type-caption-strong` using `provider-{name}-text`.
- Message card has a 3px left border in `provider-{name}-border`.

#### Status Indicators
- Circular indicator dot (8px diameter) filled with provider color, paired with health state label.
- The health state label text uses status colors (not provider colors) — the dot provides provider identity, the label text provides health status.

#### Binding Cards
- Card header: provider icon + provider name in `type-subtitle`.
- Card left border: 3px `provider-{name}-border`.

#### Composer Target Selector
- Each selectable target shows: provider icon + name.
- Selected targets are indicated by `provider-{name}-solid` background at low opacity with a checkmark overlay.

#### Edge Policy Editor
- Source node: provider icon + name, pill-shaped, filled with `provider-{name}-muted`.
- Target node: same treatment.
- Connecting arrow/line uses `border-default` — neutral, not provider-colored.

#### Export Headers
- In exported metadata, provider identity uses display names only (no icons in text export).
- In the export dialog UI, message selection rows show provider icon + name.

---

## 3. Component Inventory

For every component: structure, spacing, typography, color, states, and responsive behavior.

### 3.1 Workspace List

**Context:** First view when the extension opens with no active workspace, or accessed via a navigation breadcrumb/back action from an active workspace.

**Layout:**
- Vertical scrolling list.
- Each workspace row: `space-5` vertical padding, `space-6` horizontal padding.
- Row content, left to right: workspace name (`type-body-strong`, `text-primary`), status summary badge cluster (right-aligned).

**Status summary badges on each row:**
- Provider count badge: e.g., "3 providers" in `type-caption`.
- Active run indicator: small pulsing dot if a run is in progress.
- Message count: e.g., "142 msgs" in `type-caption`, `text-secondary`.

**Actions:**
- **Create:** Top of list or empty-state CTA. Icon-button (plus icon) + label "New Workspace".
- **Row right-click / long-press / overflow menu:** Rename, Duplicate, Archive, Delete, Export.
- **Row click:** Opens the workspace.
- **Rename:** Inline text edit on the workspace name. Press Enter to confirm, Escape to cancel.
- **Delete:** Confirmation dialog (see §3.22).

**States:**
- Default: `surface-raised`, `border-subtle` bottom separator.
- Hover: `surface-hover` overlay.
- Active (current workspace): `surface-selected`, `border-accent` left indicator (3px).
- Archived: `text-tertiary` for all text, italicized name, "Archived" badge.

**Archive Filter:**
- Below the "New Workspace" button, a toggle filter row: "Active" / "Archived" segmented control.
- Default: "Active" (archived workspaces are hidden).
- When "Archived" is selected, only archived workspaces are shown. Each archived workspace row includes a "Restore" action in its overflow menu.
- Restoring an archived workspace moves it back to the active list and removes the "Archived" badge.

**Responsive:**
- Sidebar: Full-width list, no changes needed — this is already a vertical list.
- Full-tab: List occupies a 320px left column. Selected workspace content fills the remaining space.

### 3.2 Workspace Header

**Context:** Top of the active workspace view. Always visible, never scrolls with content.

**Layout — sidebar (360px):**
- Two rows stacked.
- **Row 1:** Back arrow (to workspace list) | Workspace name (`type-title`, truncated with ellipsis at ~220px) | Overflow menu icon.
- **Row 2:** Mode badge | Strategy badge | Run status indicator | Provider status row.
- Total height: ~80px with `space-5` padding.

**Layout — full-tab (1200px+):**
- Single row.
- Left: Back arrow | Workspace name (`type-display`).
- Center: Mode badge | Strategy badge.
- Right: Run status indicator | Provider status row | Overflow menu.
- Total height: ~56px with `space-6` padding.

**Mode badge:** Pill shape (`radius-xl`), `surface-sunken` background, `type-caption-strong` text. Shows current orchestration mode: "Broadcast", "Directed", "Relay", "Roundtable", "Moderator", "Chain", etc.

**Strategy badge:** Same treatment. Shows context strategy: "Delta-Only", "Full History", "Window(5)", etc.

**Run status indicator:**
- No run: Hidden.
- Running: Pulsing `status-success` dot + "Running R3" (round number) in `type-caption-strong`.
- Paused: Static `status-warning` dot + "Paused R3".
- Completed: `status-info` dot + "Completed (5 rounds)".
- Aborted: `status-error` dot + "Aborted R2".

**Provider status row:**
- Horizontal row of provider status chips, each: provider icon (12×12px) + colored dot (6px, indicating health — green for Ready, amber for Degraded, red for Error/Disconnected, gray for unbound).
- Sidebar: All 4 provider dots are always visible in a compact row (4 dots at 6px with `space-2` gaps fits in ~56px). If a hypothetical 5th+ provider is added in future versions, overflow into "+N" counter chip.
- Full-tab: All providers visible with icon + dot.

### 3.3 Message Log

**Context:** The primary content area of the active workspace. Scrollable. Occupies all vertical space between the workspace header and the composer.

**Message Card Layout:**
- Each message is a card.
- Left border: 3px `provider-{name}-border`.
- Card background: `surface-raised`.
- Internal padding: `space-5` (sidebar), `space-6` (full-tab).

**Card Content — Top Row (Attribution):**
- Provider icon (14×14px) + Provider name (`type-caption-strong`, `provider-{name}-text`) + Timestamp (`type-caption`, `text-secondary`) + Round badge if part of a run ("R3", `type-caption`, pill shape).
- Right-aligned: Status badge if applicable (e.g., "Timeout", "Error", "Uncertain Capture").
- If export selection mode is active: Checkbox appears at the far left of the attribution row, before the provider icon.

**Card Content — Body:**
- Message text in `type-body`, `text-primary`.
- Structured blocks rendered as: paragraphs, headings (mapped to `type-subtitle`), code fences (`surface-sunken` background, `type-code`, `font-mono`, `radius-md`), bullet/numbered lists (indented with standard list markers), blockquotes (left border 2px `border-subtle`, `text-secondary` text, `space-4` left padding), tables (compact table with `border-subtle` cell borders), inline code (`surface-sunken` inline, `type-code-small`, `radius-sm`).
- Long messages: Collapsed after ~8 lines with a "Show more" text button. Fully expanded on click.

**Card Content — Inspection Trigger:**
- Clicking anywhere on the card (except interactive elements) opens the message inspection panel (§3.4).
- A subtle expand/inspect icon appears on hover at the top-right corner of the card.

**Round Grouping:**
- Messages belonging to the same round are visually grouped.
- A round header divider appears before the first message of each round: thin horizontal line with "Round 3" label centered, in `type-caption`, `text-tertiary`.
- Within a round group, message card vertical spacing is reduced (`space-2` vs the normal `space-4` between unrelated messages).

**Search Highlight:**
- When search is active, matching text spans within message bodies are highlighted with `status-warning-muted` background and `text-primary` foreground. Current match (navigated to) uses `status-warning-solid` background with `text-inverse`.

**Selection for Export:**
- When export selection mode is active, each message card gains a checkbox at the far left.
- Selected cards show `surface-selected` background.
- A floating selection toolbar appears at the bottom of the message log with: count of selected messages, "Select All", "Select All by Participant" dropdown, "Invert Selection", "Clear", and "Export Selected" button.

**Responsive:**
- Sidebar: Cards stretch full width. Code blocks scroll horizontally.
- Full-tab: Message log column is 55–65% width (flexible). Cards have `space-8` max left/right margin for readability.

### 3.4 Message Inspection Panel

**Context:** Opens when a message card is clicked. In sidebar mode, this slides over the message log as a full-width overlay panel. In full-tab mode, this occupies a right-side panel (35–45% width).

**Layout:**
- **Header:** "Message Inspection" title (`type-title`) + close (X) button. Provider icon + provider name + timestamp below.
- **Tabs:** Three tabs — "Sent Payload", "Raw Response", "Metadata".

**Sent Payload Tab:**
- Shows the exact rendered text that was injected into this provider's input, if this message was produced by a dispatch.
- If the message is a user-authored input or a captured response with no dispatch linkage, this tab shows "No dispatch payload — message was captured directly."
- Content rendered in `font-mono`, `type-code`, `surface-sunken` background, scrollable.
- Context tags (e.g., `<gpt-response>`) highlighted with `text-secondary`.

**Raw Response Tab:**
- The raw text as extracted from the provider DOM, before normalization.
- Same rendering: `font-mono`, `type-code`, `surface-sunken`.

**Metadata Tab:**
- Key-value table.

| Field | Value |
|-------|-------|
| Message ID | UUID in `font-mono` |
| Workspace | Name |
| Participant | Provider display name |
| Role | User / Assistant / System |
| Round | Number or "—" |
| Run | Run ID link or "—" |
| Timestamp | Full ISO 8601 |
| Character Count | Number |
| Dispatch ID | UUID link or "—" |
| Source Binding | Binding ID or "—" |
| Delivery Cursor State | "Delivered to: Claude (R3), Gemini (R3). Pending: Grok." |
| Tags | Tag list, editable |

**States:**
- Loading: Skeleton shimmer on content areas.
- No data: Informational placeholder per tab.

**Responsive:**
- Sidebar (360px): Full-width overlay, slides in from the right. Back button returns to message log.
- Full-tab (1200px+): Side panel, resizable between 300px and 600px. Message log narrows to accommodate.

### 3.5 Composer

**Context:** Fixed at the bottom of the active workspace view, always visible below the message log.

**Layout:**
- Vertically, bottom to top:
  1. **Action row:** Send button (right) + mode selector (left) + package preview toggle (center-left) + context pick button (center).
  2. **Input area:** Multi-line text input.
  3. **Notes bar** (visible when notes are pinned or per-target notes exist): Collapsible strip showing note indicators.
  4. **Target selector row:** Horizontal row of provider toggle chips.
  5. **Template selector:** Small dropdown, visible only when >1 template is available.

**Target Selector Row:**
- Horizontal row of chips, one per bound provider.
- Each chip: Provider icon (12×12px) + provider name (`type-caption-strong`).
- Unselected: `surface-raised`, `border-default`.
- Selected: `provider-{name}-muted` background, `provider-{name}-border` border, checkmark overlay on icon.
- Disabled (unbound/disconnected): `text-tertiary`, no interaction.
- Clicking toggles selection. Multiple selection is the default.

**Input Area:**
- `surface-sunken` background, `border-default` border, `radius-md`.
- `type-body`, `font-sans`.
- Min height: 2 lines (~52px). Max height: 8 lines (~208px) before scrolling.
- Placeholder: "Type a message…" in `text-tertiary`.
- Focus state: `border-accent`, subtle `accent-primary` glow.

**Mode Selector:**
- Segmented control or dropdown with three options: "Send" (default), "Draft Only", "Copy Only".
- "Send" — `accent-primary` indicator.
- "Draft Only" — `status-info` indicator, icon: document outline.
- "Copy Only" — `status-info` indicator, icon: clipboard.

**Context Pick Button:**
- Icon-button: crosshair/target icon. Activates context picking mode in the message log.
- Tooltip: "Pick messages for context".
- When active: `accent-primary` fill. The message log enters a selection mode similar to export selection (checkboxes appear on each message card). A floating bar at the bottom of the message log shows: "{N} messages picked for context" + "Done" primary button + "Clear" text button.
- Picked messages are included in the outbound package as context blocks (visible in the package preview). The selection persists until cleared, changed, or the message is sent.
- When no messages are manually picked, the system uses the workspace's default context strategy (delivery cursors, edge policies). When messages are manually picked, they override the automatic context selection for this send only.

**Pinned Notes and Per-Target Notes:**
- The notes bar sits between the target selector row and the input area.
- **Pinned note:** A workspace-level note included in every outbound package. Displayed as a small banner: pin icon + note text preview (truncated to 1 line) + edit (pencil) icon + remove (X) icon. `surface-sunken` background, `type-caption`.
- **Per-target note:** When a target chip in the target selector is long-pressed or right-clicked, a "Add note for {Provider}" option appears. Per-target notes are shown as small pills below the target chip they belong to: `provider-{name}-muted` background, note text preview, edit/remove icons.
- **Editing notes:** Clicking edit on any note opens a small inline text input that replaces the note pill. Press Enter to save, Escape to cancel. Max length: 500 characters.
- Notes are included in the rendered package: pinned notes appear at the top of the package (after the preamble, before context blocks), per-target notes appear only in the package for that specific target.

**Package Preview Toggle:**
- Icon-button: eye icon. Toggles the package preview panel (§3.6) between the input area and the target selector.
- Active state: `accent-primary` fill.
- Tooltip: "Preview outbound package".

**Send Button:**
- Primary action button: filled `accent-primary`, `text-inverse`, `radius-md`.
- Label: "Send" (or "Draft" / "Copy" depending on mode).
- Disabled when: no text entered, no targets selected, or all selected targets are disconnected.
- Loading state: spinner replaces label text.

**Template Selector:**
- Small dropdown button to the right of the target row.
- Shows current template name in `type-caption`.
- Dropdown lists available templates with radio-select behavior.

**Responsive:**
- Sidebar: Full-width. Target chips may wrap to two rows if 4+ providers.
- Full-tab: Composer stretches across the message log column width (not the full viewport — does not span the inspection panel).

### 3.6 Package Preview

**Context:** Toggled from the composer. Appears between the target selector row and the text input area, pushing the input area down (or in full-tab, may appear as a side panel).

**Layout:**
- Header: "Outbound Package" (`type-subtitle`) + provider target labels + close (X) button.
- Content: The fully rendered payload as it will be injected, in `font-mono`, `type-code`, `surface-sunken` background.
- Editable: The entire preview is a text area. Edits modify the payload that will be sent.
- Per-message removal: Each included message block has a small (X) remove button at its top-right corner. Removing a message block reflows the template.
- Footer: "Copy to Clipboard" button (secondary style) + character count in `type-caption`.

**States:**
- Empty: "No content to preview. Compose a message or configure context sources."
- Editing: Yellow-tinted border (`status-warning-border`) to indicate the payload has been manually modified.
- Copied: "Copied" confirmation text replaces "Copy to Clipboard" for 2 seconds.

**Responsive:**
- Sidebar: Full-width, max height 50% of available space, scrollable.
- Full-tab: Same column as composer, inline above the input area. Not a floating panel — it pushes the input area down within the composer column.

### 3.7 Run Controls Bar

**Context:** Appears below the workspace header when a run is active or can be started. Sticky, does not scroll with messages.

**Layout:**
- Single horizontal row, `space-4` vertical padding, `surface-raised` background, `border-subtle` top and bottom borders.
- Left section: Run state buttons + "Edit Package" button (visible only when paused).
- Center: Round counter + inter-round delay indicator + barrier policy label.
- Right: Active participants row + kill switch button.

**Buttons and State-Dependent Visibility:**

| Run State | Visible Buttons |
|-----------|----------------|
| No run (idle) | **Start** (primary) |
| Running | **Pause** (secondary), **Step** (secondary), **Stop** (secondary), **Abort** (danger) |
| Paused | **Resume** (primary), **Edit Package** (secondary), **Step** (secondary), **Stop** (secondary), **Abort** (danger) |
| Completed | **New Run** (primary) |
| Aborted | **New Run** (primary) |

**Button Styles:**
- Primary: `accent-primary` fill, `text-inverse`.
- Secondary: `surface-sunken` fill, `text-primary`, `border-default`.
- Danger (Abort): `status-error-solid` fill, `text-inverse`. Requires a confirmation press — first click changes label to "Confirm Abort?" for 3 seconds, second click executes.

**Round Counter:** "Round 3 / 10" (current / max) in `type-body-strong`. If no max, just "Round 3".

**Inter-Round Delay Indicator:** When between rounds, a small countdown or label: "Next in 4s" in `type-caption`, `text-secondary`.

**Barrier Policy Label:** Small badge showing the active barrier: "Wait All", "Quorum(3)", "First", "Manual". `type-caption`, `surface-sunken`, `radius-sm`.

**Active Participants Row:**
- Small horizontal row of provider icon dots (same style as workspace header provider status row).
- Each dot is clickable. Clicking a provider dot during a run opens a small popover with: provider name, health status, and a "Disable for this run" / "Re-enable" toggle.
- Disabled participants: dot changes to `text-tertiary` with a strikethrough line over the icon. The provider is excluded from future rounds but their existing messages remain in the log.
- Re-enabling a disabled participant makes them active for the next round.

**"Edit Package" Button (Paused State):**
- Visible only when the run is paused.
- Clicking opens the Between-Rounds Review surface (§3.23) where the user can inspect and edit the next outbound package before resuming.

**Responsive:**
- Sidebar: Buttons use icon-only variants (no labels). Tooltips provide labels. Active participants row collapses into a single "{N} active" badge that opens the participant popover on click.
- Full-tab: Buttons show icon + label. Active participants row shows all provider dots inline.

### 3.8 Provider Binding Cards

**Context:** Accessed from the workspace header provider status row (click to expand) or from a dedicated "Providers" section/tab within the workspace.

**Layout per card:**
- Card: `surface-raised`, `radius-md`, `border-default`. 3px left border in `provider-{name}-border`.
- Internal padding: `space-5`.
- **Row 1:** Provider icon (20×20px) + Provider name (`type-subtitle`) + Health state badge (right-aligned).
- **Row 2:** Tab status line: "Tab #42 — chat.openai.com" in `type-caption`, `text-secondary`. Or "No tab bound" in `text-tertiary`.
- **Row 3:** Last activity: "Last seen 2m ago" in `type-caption`, `text-tertiary`.
- **Actions row:** "Rebind" button (secondary) + "Open Tab" button (secondary) + overflow menu.

**Health State Badge:**
- Pill shape, icon + label in `type-caption-strong`.

| Health State | Icon | Color |
|-------------|------|-------|
| Ready | Filled circle | `status-success` |
| Composing | Pencil | `status-success` |
| Sending | Outgoing arrow | `status-success` |
| Generating | Animated dots / spinner | `status-info` |
| Completed | Checkmark | `status-success` |
| Disconnected | Broken link | `status-neutral` |
| PermissionMissing | Lock | `status-warning` |
| LoginRequired | Person with X | `status-warning` |
| DomMismatch | Warning triangle | `status-error` |
| Blocked | Stop sign | `status-error` |
| RateLimited | Clock with slash | `status-warning` |
| SendFailed | X circle | `status-error` |
| CaptureUncertain | Question mark circle | `status-warning` |
| DegradedManualOnly | Hand / manual icon | `status-warning` |

**States:**
- Healthy (Ready/Completed): Standard card.
- Error states: Card gets a faint `status-error-muted` background tint.
- Warning states: Card gets a faint `status-warning-muted` background tint.
- Unbound: Card is more muted — `text-tertiary` for most text, "Bind" CTA button appears.

**Responsive:**
- Sidebar: Cards stack vertically, full width.
- Full-tab: 2-column grid (2×2 for 4 providers).

### 3.9 Edge Policy Editor

**Context:** Accessed from a workspace settings/configuration area, or via a dedicated "Routing" tab.

**Layout:**

The editor is divided into two zones:

**Zone 1 — Graph Visualization (top or left):**
- Participants are arranged as nodes. Each node is a pill: provider icon + name, filled with `provider-{name}-muted`.
- The User node is positioned distinctly (top-center or left-center).
- Directed edges between nodes are drawn as lines with arrowheads. Each edge line is clickable.
- Enabled edges: solid line, `border-default` color.
- Disabled edges: dashed line, `text-tertiary` color.
- Edge with approval required: small pause/gate icon on the edge midpoint.
- Selected edge (being edited): thicker line, `accent-primary` color.

**Zone 2 — Edge Detail Panel (bottom or right):**
- Appears when an edge is selected.
- Header: Source pill → arrow → Target pill.
- Form fields:

| Field | Control |
|-------|---------|
| Enabled | Toggle switch |
| Catch-Up Rule | Dropdown: Full History, Last N, Range, Pinned Summary, None |
| Catch-Up "N" | Number input (visible when "Last N" selected) |
| Incremental Rule | Dropdown: Unseen Delta, Last Response, Sliding Window, Full History, Manual |
| Window Size | Number input (visible when "Sliding Window" selected) |
| Self-Exclusion | Toggle: "Exclude target's own prior turns" (default on) |
| Include User Turns | Checkbox (default on) |
| Include System Notes | Checkbox (default off) |
| Include Pinned Summaries | Checkbox (default on) |
| Truncation | Toggle + char-count input |
| Approval Required | Toggle: "Require user approval before sending" |
| Template Override | Dropdown: "Use workspace default" + template list |
| Priority | Number input (lower = higher priority) |

- Each form field has a label in `type-label` and description text in `type-caption`, `text-secondary`.
- **Priority visualization:** In the graph view (full-tab), edges are rendered in priority order — higher-priority edges (lower number) are drawn on top of lower-priority edges. In the sidebar edge list, edges are sorted by priority (ascending). Each edge row shows a small priority number badge (`type-caption`, `surface-sunken`, `radius-full`, 20×20px circle) at the far right of the row. When the user changes a priority value and saves, the edge list re-sorts with a `duration-normal` reorder animation.

**States:**
- No edge selected: Zone 2 shows instructional text — "Select an edge to configure its routing policy."
- Modified but unsaved: "Unsaved changes" indicator + Save/Discard buttons.

**Responsive:**
- Sidebar (360px): Graph visualization is a compact vertical list instead of a spatial graph. Each edge is a row: "ChatGPT → Claude" with toggle and expand-arrow. Expanding a row shows the edge detail form inline.
- Full-tab (1200px+): Left column (~40%) for graph visualization, right column (~60%) for edge detail panel.

### 3.10 Delivery Cursor Inspector

**Context:** Accessed from message inspection panel's metadata, or from a dedicated "Cursors" section within routing configuration.

**Layout:**
- Table view. One row per active edge (source→target pair).

| Column | Content |
|--------|---------|
| Edge | Source provider icon + "→" + Target provider icon + names |
| Cursor Position | Message ID (truncated, monospace) + timestamp |
| Status | "Current" / "Frozen" / "Behind by N" |
| Actions | Reset, Freeze/Unfreeze, Force Catch-Up, Mark Delivered |

**Action Buttons:**
- "Reset" — secondary button, triggers confirmation.
- "Freeze" / "Unfreeze" — toggle button. Frozen state shows a snowflake icon and `status-info` tint.
- "Force Catch-Up" — secondary button.
- "Mark Delivered" — secondary button. Opens a small inline panel: "Mark messages as already delivered up to:" + a message picker (dropdown of recent messages on this edge, showing message ID prefix + timestamp + first line preview). Selecting a message and clicking "Confirm" advances the cursor to that message without actually sending anything. This is used for manual cursor management when the user has pasted content manually or wants to skip messages.

**States:**
- Normal: Standard table row.
- Frozen: Row has `status-info-muted` background, "Frozen" badge.
- Behind: Row has `status-warning-muted` background if >10 messages behind.

**Responsive:**
- Sidebar: Table collapses into card-per-edge layout, actions in overflow menu.
- Full-tab: Standard table.

### 3.11 Context Strategy Selector

**Context:** In workspace settings or the workspace header as a dropdown.

**Layout:**
- Dropdown or segmented selector showing the workspace-level default strategy.
- Options: "Delta Only (Unseen)", "Full History", "Sliding Window", "Last Response Only", "Manual".
- **Conditional inputs:** When "Sliding Window" is selected, a number input appears inline to the right: "Window size:" + number input (default: 5). Same treatment as the edge policy editor's conditional fields.
- Below the selector: a note in `type-caption`, `text-secondary`: "Per-edge overrides can be set in the routing editor."
- If any edge has a per-edge override, an indicator shows: "2 edges with overrides" as a clickable link. Clicking navigates to the edge policy editor. Below this link, a compact list shows which edges have overrides: each line shows "Source → Target: {override strategy}" in `type-caption`, `text-secondary`, with the override strategy name using the same label as the dropdown options. This lets the user see at a glance which edges deviate from the default without navigating away.
- In the edge policy editor (§3.9), any edge whose incremental rule differs from the workspace default shows a small "Override" badge (`type-caption`, `status-info-muted` background, `radius-sm`) next to the Incremental Rule dropdown.

### 3.12 Template Manager

**Context:** Accessible from workspace settings or a "Templates" tab.

**Layout:**
- Left column (or top in sidebar): Template list.
  - Each row: Template name (`type-body`), kind badge, format family badge (for body templates), version in `type-caption`.
  - Built-in templates marked with a "Built-in" badge (not editable, but can be duplicated).
  - Actions: Create (top button), Edit, Duplicate, Delete (per-row overflow menu).
  - **Grouping:** Templates are grouped by kind with section headers: "Preamble Templates", "Body Templates", "Filename Templates".
- Right column (or expanded section in sidebar): Template editor.
  - Template name input.
  - Template kind dropdown: "Preamble", "Body", "Filename".
  - **Format family selector** (visible only for Body kind): Radio group — "XML Wrapped" (the `<provider-response>` pattern), "Markdown Sections" (headers with provider names), "Plain Text Labels" (minimal labels). Built-in templates each correspond to one format family. Custom body templates also declare a format family so the system can show meaningful previews.
  - **Preamble sub-kind** (visible only for Preamble kind): Radio group — "Neutral", "Collaboration", "Debate", "Review/Critique", "Synthesis", "Custom". Built-in preamble templates correspond to the first five; "Custom" is for user-authored preambles.
  - Template body: large text area, `font-mono`, `type-code`, `surface-sunken`.
  - Variable reference: Collapsible section listing all available template variables with descriptions. Example: `{{provider_name}}`, `{{timestamp}}`, `{{body}}`, `{{round}}`, etc.
  - Preview: Below the editor, a live preview section showing the template rendered with sample data. `surface-sunken` background, `font-mono`.

**States:**
- New template: Editor open with empty fields, "Create" button.
- Editing: "Save" and "Cancel" buttons.
- Preview loading: Skeleton shimmer.

**Responsive:**
- Sidebar: List and editor are sequential (navigate into editor, back to list).
- Full-tab: Side-by-side.

### 3.13 Export Dialog

**Context:** Opened from workspace overflow menu, message selection toolbar, or keyboard shortcut.

**Layout:**
A full-overlay dialog in sidebar, or a large centered modal (640px wide) in full-tab.

**Sections, top to bottom:**

**1. Scope Selector:**
- Radio group: "Entire Workspace", "Single Provider", "Single Run", "Selected Rounds", "Selected Messages", "Dispatch Log", "Diagnostics".
- Sub-filters appear contextually: e.g., provider dropdown for "Single Provider", run picker for "Single Run", round range inputs for "Selected Rounds".

**2. Selection Filters:**
- Collapsible section. Checkboxes for: Include user messages, Include system messages, Include assistant messages, Filter by tag (tag multi-select).
- When scope is "Selected Messages": shows count of selected messages, "Change Selection" link returns to message log in selection mode.

**3. Format Picker:**
- Segmented control: Markdown | JSON | TOML.
- Each segment shows the format name and a small icon.

**4. Layout Options (Markdown only, hidden for JSON/TOML):**
- Radio: "Chronological", "Grouped by Round", "Grouped by Participant".
- Toggles: "Include message headings", "Preserve code fences", "Include metadata appendix".

**5. Metadata Toggle Panel:**
- Grid of toggles (2 columns), each a labeled checkbox.
- Toggleable fields: workspace name, workspace ID, export title, export timestamp (local), export timestamp (UTC), scope type, participants, orchestration mode, run ID, round range, message count, template used, context strategy, edge policy snapshot, conversation references, model labels, browser name, extension version, export profile name, tags/notes, diagnostics summary, raw payload inclusion.

**5b. Metadata Placement Selector** (Markdown only, hidden for JSON/TOML which have fixed structure):
- For Markdown exports, a row of placement options, each a checkbox (multiple can be active):
  - "Front matter (TOML)" — metadata block at the top of the file in TOML front matter delimiters (`+++`).
  - "Top heading" — metadata rendered as a structured section after the title heading.
  - "Inline message labels" — per-message metadata (provider, timestamp, round) rendered as part of each message heading.
  - "Footer" — metadata block at the end of the file.
  - "Metadata appendix" — a dedicated "## Metadata" section at the end with full structured metadata.
- Default: "Front matter" + "Inline message labels" checked.
- Note in `type-caption`: "Filename metadata is always controlled by the filename template above."

**6. Filename Template Input:**
- Text input showing the filename template, e.g., `{workspace}-{date}-{format}`.
- Below: expandable specifier key reference table.

**7. Profile Management:**
- "Save as Profile" button + "Load Profile" dropdown.
- Profile name input (when saving).

**8. Action Row:**
- Left: "Copy to Clipboard" secondary button.
- Right: "Download" primary button.
- Between: A preview link: "Preview output" opens a read-only preview of the rendered export in a scrollable area.

**States:**
- Loading (generating export): Spinner overlay on action row.
- Large export warning: if >1MB estimated, yellow info bar: "This export is large (~2.3 MB). Consider narrowing the scope."
- Error: Red info bar with error message.

**Responsive:**
- Sidebar: Full-screen overlay, sections stack vertically with generous scrolling.
- Full-tab: Centered modal, 640px wide, max 80vh tall, scrollable.

### 3.14 Diagnostics Panel

**Context:** Accessible from a "Diagnostics" tab or section in the workspace, or from a global diagnostics button.

**Layout:**
- Filter bar (top): Severity filter (toggle chips: Critical, Warning, Info, Debug) + Provider filter (provider chips) + Time range (preset: Last hour, Last run, All).
- Event list: Scrolling list of diagnostic events.

**Event Row:**
- Left: Severity icon (colored per §1.1 diagnostic severity colors) + Timestamp (`type-caption`, `text-secondary`).
- Center: Event code (`type-code-small`, `font-mono`) + Detail text (`type-body`, truncated to 1 line).
- Right: Provider icon (if provider-specific).
- Click expands to show the full event detail. Expanded layout:
  - Full detail text in `type-body`, wrapping freely.
  - Key-value metadata rows in `type-caption`: Event Code, Binding ID, Snapshot Reference.
  - If snapshot reference is present: a "View Snapshot" text link that navigates to the relevant binding card or message inspection panel.
  - Expanded area has `surface-sunken` background, `space-4` padding, `radius-sm`.

**States:**
- Empty: "No diagnostic events." with checkmark icon.
- Filtered to empty: "No events match the current filters."
- Streaming: New events appear at the top with a brief `duration-gentle` fade-in.

**Responsive:**
- Sidebar: Full-width list, filter bar wraps.
- Full-tab: Full-width within workspace content area.

### 3.15 Settings Page

**Context:** Accessed from the extension's global menu, not workspace-specific.

**Layout:**
- Vertical scrolling page organized into sections with section headers (`type-title`).

**Sections:**

**1. Appearance:**
- Theme: "Dark" / "Light" / "System" segmented control.
- Surface preference: "Sidebar" / "Full Tab" segmented control.

**2. Timing Defaults:**
- Per-provider generation timeout: number input + "s" suffix, default 120.
- Per-provider cooldown: number input + "s" suffix, default 2.
- Inter-round delay: number input + "s" suffix, default 5.
- Jitter: toggle + percentage input, default ±20%.
- Max concurrent sends: number input, default 4.
- Max rounds per run: number input, default 20.
- Global run timeout: number input + "m" suffix, default 60.

**3. Per-Provider Overrides:**
- Collapsible section per provider. Within each: override toggles for timeout, cooldown, and other timing settings.

**4. Orchestration Defaults:**
- Default mode: dropdown (Broadcast, Directed, etc.).
- Default context strategy: dropdown.
- Default barrier policy: dropdown.

**5. Storage:**
- Storage usage indicator: horizontal bar showing used/total (estimated from IndexedDB + storage.local).
- "Clear All Data" danger button with confirmation dialog.

**6. Automation Safety:**
- Kill switch: prominent toggle. "Immediately halt all orchestration activity."
- Styled with `status-error-muted` background when enabled, `status-error-solid` toggle color.

**7. Keyboard Shortcuts:**
- Link to the keyboard shortcut reference (§3.21), or inline list if space permits.

**Responsive:**
- Sidebar: Full-width, single column.
- Full-tab: Centered column, max-width 680px, generous `space-9` section spacing.

### 3.16 Permission Request Flow

**Context:** Appears when a provider is first activated (bound) and the required host permission has not yet been granted. Also appears if a permission is revoked and the user tries to interact with that provider.

**Layout:**
A focused overlay/modal. Not a full page — the user should feel they're in a quick step, not redirected.

- **Provider icon** (32×32px) centered at top, with `provider-{name}-muted` circular background.
- **Heading:** "Connect to {Provider Name}" in `type-title`.
- **Explanation text:** 2–3 sentences in `type-body`, `text-secondary`. Example: "Chatmux needs permission to access {provider URL} so it can read your conversations and send messages on your behalf. This permission can be revoked at any time in your browser's extension settings."
- **Permission details:** A small collapsed "What does this allow?" section that expands to list specific capabilities: "Read conversation text from the page", "Type into the message input", "Click the send button".
- **Actions:**
  - "Grant Permission" — primary button (`accent-primary`).
  - "Skip" — text link below. Tooltip: "You can grant this later from the provider binding card."
- **After grant:** Brief success checkmark animation, then auto-dismiss. Provider binding card updates to reflect new status.
- **If denied by the browser:** Error message in `status-error-text`: "Permission was not granted. You can try again from the provider binding card."

### 3.17 Keyboard Shortcut Reference

**Context:** Accessible from settings or via a help action.

**Layout:**
- Table with two columns: Action (`type-body`) and Binding (`type-code`, `font-mono`, pill-shaped `surface-sunken` backgrounds for each key).

| Action | Default Binding |
|--------|----------------|
| Open/Focus Workspace | (browser-defined) |
| Send Current Draft | (browser-defined) |
| Pause/Resume Run | (browser-defined) |
| Step Next Round | (browser-defined) |
| Cycle Target Provider | (browser-defined) |
| Toggle Provider Active/Inactive | (browser-defined) |
| Copy Current Package | (browser-defined) |

Note: Actual key combinations are configured through the browser's `commands` API. The reference shows whatever the user has currently bound.

**Responsive:**
- No differences — simple table works at all widths.

### 3.18 Empty States

Every major view has a designed empty state. Each empty state has: a centered illustration placeholder area (64×64px icon or illustrative shape), a heading in `type-subtitle`, a description in `type-body` / `text-secondary`, and a CTA button when applicable.

| View | Icon Description | Heading | Description | CTA |
|------|-----------------|---------|-------------|-----|
| Workspace List (no workspaces) | Stacked rectangles outline | "No workspaces yet" | "Create a workspace to start orchestrating conversations across AI providers." | "Create Workspace" |
| Message Log (no messages) | Chat bubble outline | "No messages" | "Send a message or start a run to begin the conversation." | Focus composer |
| Provider Bindings (none bound) | Puzzle piece outline | "No providers connected" | "Open a provider tab and bind it to this workspace, or let Chatmux detect open provider tabs." | "Detect Provider Tabs" |
| Run History (no runs) | Play button outline | "No runs yet" | "Start a run to orchestrate automated conversations between providers." | "Configure & Start Run" |
| Export (no results) | Document outline | "Nothing to export" | "This workspace has no messages matching your export criteria." | "Adjust Filters" |
| Diagnostics (no events) | Checkmark in shield | "All clear" | "No diagnostic events have been recorded." | — |
| Templates (no custom) | Pencil on document | "Using built-in templates" | "Create a custom template to control how messages are packaged for each provider." | "Create Template" |
| Search (no results) | Magnifying glass with X | "No matches" | "No messages match your search. Try different keywords or adjust your filters." | "Clear Search" |
| Pinned Summaries (none) | Pin outline | "No pinned summaries" | "Create a pinned summary to use as compact context for catch-up rules." | "Create Summary" |

### 3.19 Error States

Each error state appears contextually — not as a full-page error, but as an inline banner or card-level indicator within the relevant component.

**Inline Error Banner:**
- `status-error-muted` background, `status-error-border` left border (3px), `space-4` padding.
- Icon: warning triangle, `status-error-text`.
- Title in `type-body-strong`, `status-error-text`.
- Description in `type-body`, `text-primary`.
- Dismiss (X) if the error is informational. Action button if the user can take corrective action.

| Error | Where It Appears | Title | Description | Action |
|-------|-----------------|-------|-------------|--------|
| Adapter DOM Mismatch | Binding card + diagnostics | "Provider page changed" | "{Provider} has updated their page structure. Chatmux cannot interact with this provider until the adapter is updated." | "View Details" |
| Login Required | Binding card | "Login required" | "You need to log in to {Provider} before Chatmux can access your conversations." | "Open {Provider} Tab" |
| Rate Limited | Binding card + message log (on affected message) | "Rate limited" | "{Provider} is temporarily limiting requests. Chatmux will retry automatically." | — |
| Send Failed | Message card (dispatch status badge) | "Send failed" | "Could not deliver the message to {Provider}. The message was saved as a draft." | "Retry" |
| Storage Full | Settings + toast | "Storage full" | "Chatmux's local storage is full. Export or delete old workspaces to free space." | "Manage Storage" |
| Permission Missing | Binding card | "Permission needed" | "Chatmux does not have permission to access {Provider}." | "Grant Permission" |

### 3.20 Confirmation Dialogs

**Standard dialog structure:**
- Centered modal, `shadow-lg`, `surface-raised`, `radius-lg`.
- Max width: 440px.
- Internal padding: `space-7`.
- Icon (optional, top-center): relevant icon at 32×32px.
- Heading: `type-title`.
- Description: `type-body`, `text-secondary`, 1–3 sentences.
- Actions: right-aligned — secondary cancel button + primary/danger confirm button.

| Dialog | Icon | Heading | Description | Cancel | Confirm |
|--------|------|---------|-------------|--------|---------|
| Delete Workspace | Trash can | "Delete workspace?" | "This will permanently delete '{name}' and all its messages, runs, and settings. This cannot be undone." | "Cancel" | "Delete" (danger) |
| Clear All Data | Exclamation in triangle | "Clear all data?" | "This will delete all workspaces, messages, settings, and profiles. This cannot be undone." | "Cancel" | "Clear Everything" (danger) |
| Abort Run | Stop octagon | "Abort this run?" | "This will immediately halt the current run. In-flight dispatches may leave providers in an incomplete state." | "Cancel" | "Abort" (danger) |
| Archive Workspace | Archive box | "Archive workspace?" | "'{name}' will be moved to the archive. You can restore it later." | "Cancel" | "Archive" (secondary) |
| Reset Cursor | Rewind arrows | "Reset delivery cursor?" | "This will reset the cursor for {source}→{target}. The next send will include full catch-up context." | "Cancel" | "Reset" (secondary) |
| Mark Delivered | Fast-forward arrows | "Mark messages as delivered?" | "This will advance the cursor for {source}→{target} to the selected message without actually sending anything." | "Cancel" | "Mark Delivered" (secondary) |
| Steal Binding | Link/chain icon | "Rebind this tab?" | "This tab is currently bound to workspace '{name}'. Binding it here will disconnect it from that workspace." | "Cancel" | "Bind Here" (primary) |

### 3.21 Toast / Notification System

**Toast layout:**
- Appears at the top-right of the extension viewport (sidebar or tab).
- Max width: 320px.
- `surface-raised`, `shadow-md`, `radius-md`.
- Internal: icon (left, 16×16px) + message text (`type-body`, 1–2 lines) + dismiss (X) on the right.
- Auto-dismiss after 5 seconds for info/success, 8 seconds for warnings, manual dismiss for errors.

**Toast types:**

| Type | Icon | Left Border Color |
|------|------|--------------------|
| Provider status change | Provider icon | `provider-{name}-border` |
| Round completion | Checkmark circle | `status-success-border` |
| Run state change | Play/pause/stop icon | `status-info-border` |
| Export ready | Document with checkmark | `status-success-border` |
| Copy confirmed | Clipboard checkmark | `status-success-border` |
| Warning | Warning triangle | `status-warning-border` |
| Error | X circle | `status-error-border` |

**Stacking:** Multiple toasts stack vertically with `space-2` gaps. Maximum 3 visible — older toasts are dismissed when a 4th arrives.

### 3.22 Search and Filter Bar

**Context:** Appears at the top of the message log, below the run controls bar (or below the workspace header if no run is active). Toggled by a search icon-button in the workspace header or by keyboard shortcut.

**Layout:**
- Single row when collapsed (search only): magnifying glass icon + text input (`surface-sunken`, `border-default`, `radius-md`, `type-body`, placeholder: "Search messages…") + close (X) icon-button.
- When expanded (search + filters): the search input row + a filter row below it.

**Search Input:**
- Full-width text input.
- Searches message body text across the entire workspace.
- Results are highlighted in-place in the message log (see §3.3 search highlight) — the message log is not replaced by a results list.
- A results counter appears at the right of the input: "3 of 17" in `type-caption`, `text-secondary`.
- Up/down arrow buttons (or keyboard shortcuts) navigate between matches. The message log auto-scrolls to center the current match.

**Filter Row:**
- Toggled by a funnel icon-button at the right of the search input.
- Horizontal row of filter controls, wrapping on sidebar:

| Filter | Control | Default |
|--------|---------|---------|
| Participant | Multi-select chips (one per provider + User + System) | All selected |
| Role | Toggle chips: User, Assistant, System | All selected |
| Run | Dropdown: "All Runs", "Current Run", or specific run by ID/timestamp | "All Runs" |
| Round Range | Two number inputs: "From round" / "To round" (visible when a specific run is selected) | Empty (all rounds) |
| Delivery Status | Toggle chips: Sent, Received, Timeout, Error | All selected |
| Tags | Tag multi-select dropdown | None (show all) |

- Active filters are indicated by a badge count on the funnel icon: e.g., a small "3" badge means 3 filters are actively narrowing the results.
- "Clear Filters" text button at the right end of the filter row.

**Saved Filters:**
- Below the filter row: "Save Filter" text button + "Saved Filters" dropdown.
- "Save Filter" prompts for a filter name (inline input, `type-body`) then saves the current filter combination.
- "Saved Filters" dropdown lists saved filter presets. Selecting one applies all its filter settings. Each saved filter has an (X) delete action on hover.
- Maximum 10 saved filters per workspace.

**Special Views:**
- **Provider-specific transcript:** Selecting only one participant in the participant filter creates a single-provider view. A header label appears above the filtered message log: "Showing: ChatGPT only" in `type-caption-strong`, `provider-{name}-text`.
- **Unseen-since-last-delivery:** A predefined filter accessible as a quick-action link in the filter row: "Show unseen by {Provider}" dropdown (one option per bound provider). Selecting this sets the message log to show only messages that have not yet been delivered to that provider (based on delivery cursor state). The header label shows: "Undelivered to Claude" in `type-caption-strong`, `status-warning-text`.

**States:**
- No results: Message log shows an inline empty state — "No messages match your search." + "Clear Search" button.
- Filter active with no search: Results counter hidden, filter badge visible.
- Search active with no filter: Filter row hidden, results counter visible.

**Responsive:**
- Sidebar: Search input takes full width. Filter row wraps — chips stack into 2–3 rows as needed. Saved filter controls move to an overflow menu.
- Full-tab: Search input and filter row stretch across the message log column width. Filters fit in a single row.

### 3.23 Between-Rounds Review Surface

**Context:** Appears when an autonomous run is paused (either by the user or by the moderated-autonomous pause policy) and the user clicks "Edit Package" in the run controls bar. This is the surface for reviewing and editing what will be sent in the next round before resuming.

**Layout:**
A full-width overlay in sidebar, or a dedicated panel replacing the side panel in full-tab.

**Header:**
- "Review Next Round" (`type-title`) + "Round {N+1} Preview" subtitle in `type-caption`, `text-secondary`.
- Close (X) button returns to the paused run view without changes.

**Sections:**

**1. Target Summary:**
- A list of target providers that will receive a package in the next round. Each row: provider icon + name + health status badge + "Skip this round" toggle.
- Toggling "Skip this round" on a target means that provider will not receive a package in the upcoming round but remains active for future rounds.

**2. Per-Target Package Preview:**
- A tabbed or accordion view, one tab/section per target provider.
- Each tab shows the rendered outbound package for that target, using the same rendering as the package preview component (§3.6): `font-mono`, `type-code`, `surface-sunken`, editable text area.
- Per-message remove (X) buttons on each context block.
- Character count in `type-caption`.
- If truncation would apply (soft limit exceeded): a truncation warning appears inline (see §3.26).

**3. Configuration Overrides:**
- Collapsible section: "Adjust for next round".
- Controls (all apply only to the next round, not permanently):
  - Target set checkboxes (same as §1 above, duplicated for discoverability).
  - Edge policy quick-overrides: for each active edge, a compact row showing source→target + incremental rule dropdown. Changes here are temporary (one round only) and do not modify the workspace edge policies.
  - "Edit full routing" link → navigates to the edge policy editor (§3.9), with a banner: "Changes made here will persist for future rounds."

**4. Inject User Message:**
- A small text input: "Add a user message before the next round" with a "Add" button.
- The injected message appears in the canonical message log as a User message and is included in the next round's context for all targets.

**5. Action Row:**
- "Resume with these changes" — primary button. Sends the (possibly edited) packages and resumes the run.
- "Resume without changes" — secondary button. Ignores edits and resumes with the originally generated packages.
- "Stay Paused" — text link. Closes the review surface without resuming.

**States:**
- Loading: Skeleton shimmer on package previews while packages are being rendered.
- Edited: Packages with manual edits show `status-warning-border` indicator (same as §3.6).
- Error: If a target provider has entered an error health state since pausing, its tab shows an error banner: "This provider is in an error state. The package may fail to deliver."

**Responsive:**
- Sidebar: Full-width overlay, sections stack vertically, per-target previews use accordion (not tabs).
- Full-tab: Replaces the side panel (35–45% width). Per-target previews use tabs along the top of the panel.

### 3.24 Run Configuration Sheet

**Context:** Appears when the user clicks "Start" (from idle) or "New Run" (after a completed/aborted run) in the run controls bar. This is the primary surface for configuring all parameters of an orchestrated run before it begins.

**Layout:**
In sidebar: full-width overlay, scrollable. In full-tab: centered modal, 720px wide, max 85vh tall, scrollable.

**Header:**
- "Configure Run" (`type-title`) or "Configure New Run" if a previous run exists.
- If a previous run exists: a notice bar in `status-info-muted` background: "Transcript history and delivery cursors from Run {N} are preserved." + checkbox: "Reset delivery cursors for this run" (default: unchecked).

**Sections:**

**1. Orchestration Mode:**
- Same mode selector as §5.1, but rendered as a full-sized radio card list instead of a dropdown.
- Each option is a card: mode icon + mode name (`type-subtitle`) + one-line description (`type-body`, `text-secondary`).
- Selected card: `surface-selected`, `border-accent`.
- Cards are grouped under "Manual" and "Autonomous" section headers.
- **Moderated toggle:** When an autonomous mode is selected, a toggle appears below the card: "Moderated — pause between rounds for review" (default: on for first-time users, remembers last setting thereafter).

**2. Mode-Specific Configuration:**
This section changes based on the selected mode:

| Mode | Additional Controls |
|------|-------------------|
| Broadcast | (none — targets are selected in §3 below) |
| Directed | "Primary target" dropdown (single provider) + "Context sources" multi-select checkboxes |
| Relay-to-One | "Target" dropdown + "Source" multi-select |
| Relay-to-Many | "Sources" multi-select + "Targets" multi-select |
| Roundtable | (none — all active providers participate) |
| Moderator/Jury | **"Moderator provider"** dropdown (single provider select). The remaining providers are automatically the "panel" members. A label below the dropdown: "{Provider} will receive all others' outputs and synthesize." in `type-caption`, `text-secondary`. |
| Relay Chain | **"Chain order"** — a vertical drag-to-reorder list of active providers. Each row: drag handle (six dots) + provider icon + name. Drag a row to reorder. The order defines the relay sequence: A→B→C→... A label above: "Drag to set the relay order. Output flows top to bottom." |
| Draft Only | (none) |
| Copy Only | (none) |

**3. Participant Set:**
- Grid of provider cards (same as binding cards but compact). Each card: provider icon + name + health badge + active/inactive toggle.
- Inactive providers are excluded from the run.
- "Detect & Bind" button if any providers are unbound.

**4. Barrier Policy:**
- Dropdown: "Wait for All", "Quorum", "First Finisher", "Manual Advance".
- **Quorum threshold** (visible when "Quorum" is selected): Number input + label "out of {N} active providers". Range: 1 to active provider count. Default: ceil(active/2).
- Description text below each option in `type-caption`, `text-secondary`.

**5. Timing Controls:**
- Uses the same field layout as settings (§3.15, section 2) but scoped to this run. Each field has a "Use default" checkbox (checked by default) + an override input (disabled when using default).
- Fields: Generation timeout, Cooldown, Inter-round delay, Jitter toggle + %, Max concurrent sends.
- **Per-provider follow-up override:** Below the timing fields, a collapsible "Advanced: Follow-up behavior" section. For each provider that reports `supports_follow_up_while_generating: true` (i.e., the adapter trait capability), a toggle: "Allow non-blocking follow-ups for {Provider}" (default: off). Description: "When enabled, new messages can be sent to this provider while it is still generating a response."

**6. Stop Conditions:**
- A checklist of stop conditions, each with a toggle and a configurable threshold:

| Condition | Control | Default |
|-----------|---------|---------|
| Maximum rounds | Toggle + number input | On, 20 |
| Global run timeout | Toggle + number input (minutes) | On, 60 |
| Manual stop only | Toggle (mutually exclusive with all others) | Off |
| Sentinel phrase | Toggle + text input ("Stop when any response contains:") | Off |
| Repeated provider failure | Toggle + number input ("Stop after N consecutive failures") | Off, 3 |
| Repeated timeout | Toggle + number input ("Stop after N consecutive timeouts") | Off, 3 |
| Stagnation detection | Toggle + description "Stop when responses show minimal change across rounds" | Off |
| Approval gate denied | Toggle + description "Stop if user denies approval at a gate" | On |

- Multiple conditions can be active simultaneously (OR logic — any triggered condition stops the run).

**7. Routing:**
- Compact summary of current edge policies: a mini edge list showing source→target + enabled/disabled + catch-up rule + incremental rule, one row per edge.
- "Edit Routing" link → navigates to the full edge policy editor (§3.9).
- "Use workspace defaults" checkbox (checked by default for new runs).

**8. Action Row:**
- "Start Run" — primary button.
- "Cancel" — secondary button, closes the sheet.
- If configuration is invalid (e.g., no active providers, no stop conditions): "Start Run" is disabled with a validation error message in `status-error-text` below the button.

**States:**
- Pre-populated: When opening for a new run after a previous run, all fields are pre-populated from the workspace settings and the previous run's configuration.
- Validation errors: Inline error messages below specific fields (e.g., "At least one provider must be active" below the participant set).
- Loading: If provider health is being checked, binding cards show skeleton shimmer.

**Responsive:**
- Sidebar: Full-width overlay, sections stack vertically, all content scrollable.
- Full-tab: Centered modal, 720px wide. Sections use `space-7` vertical spacing. Mode cards arrange in a 2-column grid.

### 3.25 Pinned Summary Manager

**Context:** Accessed from the edge policy editor when "Pinned Summary" is selected as a catch-up rule, or from a "Summaries" section in workspace settings.

**Layout:**
A small panel or overlay.

**Header:** "Pinned Summaries" (`type-title`) + "Create" button (plus icon).

**Summary List:**
- Each summary is a card:
  - Name/label (`type-body-strong`) — user-assigned, e.g., "Context through Round 5".
  - Preview of first 2 lines of summary text (`type-caption`, `text-secondary`).
  - Timestamp of creation (`type-caption`, `text-tertiary`).
  - Actions: Edit, Duplicate, Delete (overflow menu).
  - "In use" badge if this summary is referenced by any edge policy catch-up rule.

**Summary Editor (inline expand or sub-panel):**
- Name input (`type-body`).
- Summary body: multi-line text area, `font-sans`, `type-body`, `surface-sunken`. This is where the user writes (or pastes) a manual summary of the conversation up to a certain point.
- Character count in `type-caption`.
- "Save" primary button + "Cancel" secondary button.
- Helper text: "Write a concise summary of the conversation context. This will be sent as the initial context when catch-up rule is set to 'Pinned Summary'." in `type-caption`, `text-secondary`.

**Integration with Edge Policy Editor:**
- When "Pinned Summary" is selected as the catch-up rule in the edge policy editor (§3.9), a dropdown appears: "Select summary:" listing all pinned summaries by name. If no summaries exist, the dropdown shows "No summaries — Create one" as a clickable link that opens the pinned summary manager.

**States:**
- Empty: "No pinned summaries yet. Create one to use as initial context for catch-up." + "Create Summary" CTA.
- Editing: Form replaces the summary card in-place.

**Responsive:**
- Sidebar: Full-width overlay.
- Full-tab: Side panel content, or a 480px-wide modal.

### 3.26 Truncation Warning

**Context:** Appears inline within the package preview (§3.6) or the between-rounds review (§3.23) when a rendered payload exceeds the soft character-count limit configured for the target edge.

**Layout:**
- An inline warning banner positioned at the top of the package preview content area, above the rendered payload.
- `status-warning-muted` background, `status-warning-border` left border (3px), `space-4` padding.
- Icon: warning triangle.
- Text: "This package exceeds the soft limit for {Target Provider} ({N} / {limit} characters)." in `type-body`.
- **Action options** (presented as buttons in a row below the warning text):
  - "Auto-trim oldest" — secondary button. Removes the oldest context blocks until the payload fits within the limit. The removed blocks animate out (fade + collapse) and the character count updates.
  - "Swap for summary" — secondary button. Visible only if pinned summaries exist for this edge. Replaces older context blocks with a pinned summary, selected from a small dropdown that appears inline. If no pinned summaries exist, this button is replaced by text: "Create a pinned summary to use as a compact alternative." as a link to the pinned summary manager.
  - "Dismiss" — text button. Keeps the oversized payload and hides the warning for this send. A small "Over limit" badge remains in `status-warning-text` next to the character count.

**States:**
- Within limit: Warning is not shown.
- Over limit, not yet acted on: Full warning banner.
- Over limit, dismissed: Compact "Over limit" badge only.
- After trim/swap: Warning disappears, payload updates.

### 3.27 New Messages Indicator

**Context:** Appears at the bottom of the message log when new messages have arrived while the user is scrolled up from the bottom.

**Layout:**
- A floating pill, centered horizontally at the bottom of the message log scroll area, `space-4` above the composer.
- `accent-primary` background, `text-inverse`, `radius-xl`, `shadow-md`.
- Content: Down-arrow icon (12×12px) + "3 new messages" text in `type-caption-strong`.
- If messages are from multiple providers: "3 new from ChatGPT, Claude" (provider names in text, truncated if >2 providers to "3 new from ChatGPT +2").

**Interaction:**
- Click: smoothly scrolls the message log to the bottom (to the newest message) over `duration-slow` with `easing-standard`. The indicator fades out during the scroll.
- Auto-dismiss: when the user manually scrolls to the bottom, the indicator fades out over `duration-fast`.
- Auto-update: the count and provider names update as new messages arrive.

**States:**
- Hidden: user is at or near the bottom of the message log (within 50px of the bottom).
- Visible: user is scrolled up and new messages exist below the viewport.

### 3.28 Multi-Workspace Binding Conflict

**Context:** When the user attempts to bind a provider tab that is already bound in another workspace.

**Layout:**
- A small confirmation popover appears on the binding card or tab-picker dropdown.
- `surface-overlay`, `shadow-md`, `radius-md`, `space-5` padding.
- Text: "This tab is already bound to workspace '{other workspace name}'." in `type-body`.
- Description: "Binding it here will unbind it from that workspace. The other workspace's provider will become disconnected." in `type-caption`, `text-secondary`.
- Actions:
  - "Bind Here" — primary button. Steals the binding.
  - "Cancel" — secondary button.

**Alternative — no conflicts allowed:**
If the system instead requires unique bindings, the popover changes:
- Text: "This tab is bound to '{other workspace name}'." in `type-body`.
- Description: "Open a separate {Provider} tab, or unbind it from the other workspace first." in `type-caption`, `text-secondary`.
- Action: "Open New Tab" — secondary button. "Go to {other workspace}" — text link.

**Decision:** Use the first variant (allow steal with warning). Tabs are a scarce resource (most users won't have 2 ChatGPT tabs), and the warning makes the consequence clear.

---

## 4. Layout System

### 4.1 Sidebar Layout (~360px)

The sidebar is the default surface. All content is arranged in a single vertical column.

**Navigation model:** Stack-based. The user navigates deeper into views and uses a back action to return.

**Navigation stack:**
1. **Workspace list** — top-level.
2. **Active workspace** — message log + composer. Workspace header is sticky at top.
3. **Sub-views** — push onto stack: Message inspection, Provider bindings, Edge policy editor, Delivery cursor inspector, Template manager, Pinned summary manager, Export dialog, Diagnostics, Run configuration sheet, Between-rounds review, Settings.

**Vertical stacking within active workspace:**
```
┌─────────────────────────────┐
│ Workspace Header (sticky)   │  ~80px
├─────────────────────────────┤
│ Run Controls Bar (sticky)   │  ~44px (visible only during runs)
├─────────────────────────────┤
│ Search/Filter Bar (toggle)  │  ~40–100px (visible when search active)
├─────────────────────────────┤
│                             │
│   Message Log (scrollable)  │  flex-fill
│                             │
├─────────────────────────────┤
│ Package Preview (toggle)    │  0–50% of available height
├─────────────────────────────┤
│ Composer (sticky bottom)    │  ~120–200px
└─────────────────────────────┘
```

**Collapse behavior:**
- Sub-views (inspection, bindings, routing, etc.) push as full-width overlays that cover the message log.
- A back button in the top-left of each sub-view returns to the previous view.
- The composer remains visible when viewing bindings or routing (it's still part of the workspace) but is hidden when viewing settings, diagnostics, or full-page dialogs.

### 4.2 Full-Tab Layout (~1200px+)

**Navigation model:** Persistent navigation + multi-panel workspace view.

**Top-level layout:**
```
┌─────────────────────────────────────────────────────────────┐
│ Global Header Bar                                      56px │
├──────────┬──────────────────────────────────────────────────┤
│          │                                                  │
│ Nav Rail │  Content Area (flex-fill)                        │
│  56px    │                                                  │
│  fixed   │                                                  │
│          │                                                  │
└──────────┴──────────────────────────────────────────────────┘
```

**Global Header Bar:**
- Left: Chatmux logo/wordmark + workspace name (acts as breadcrumb).
- Right: Global diagnostics indicator + settings gear + kill switch button.
- **Global diagnostics indicator:** The shield icon from the nav rail, rendered at 16×16px. When no issues exist: `text-secondary` color, no badge. When diagnostic events exist: the icon gets a small numeric badge (count of unread critical+warning events) in a `status-error-solid` circle (12px diameter, `type-caption` white text). Clicking the indicator navigates to the diagnostics panel. The badge clears when the user opens the diagnostics panel.
- **Kill switch button (header variant):** Compact icon-button, 28×28px, stop-octagon icon. Default state: `text-secondary` icon, `surface-raised` background. When automation is running: `status-error-muted` background, `status-error-text` icon color, with tooltip "Kill switch — halt all orchestration". When pressed: immediately triggers the kill switch (same behavior as the settings toggle, but without requiring navigation to settings). After activation: icon pulses once with `status-error-solid` background, then settles to a persistent `status-error-muted` background with a small "HALTED" label in `type-caption` next to the icon until the user re-enables orchestration from settings.

**Nav Rail (left, 56px fixed):**
- Vertical icon strip.
- Icons top-to-bottom: Workspaces (grid), Active workspace (chat bubble), Routing (git-branch icon), Templates (document), Diagnostics (shield), Settings (gear).
- Active nav item: `accent-primary` icon color + vertical left indicator bar (3px).
- Tooltip on hover for each icon.

**Content Area — Active Workspace View:**
```
┌────────────────────────────────────────────────────────────┐
│ Workspace Header                                      56px │
├────────────────────────────────────────────────────────────┤
│ Run Controls Bar (visible only during runs)           44px │
├──────────────────────────────┬─────────────────────────────┤
│                              │                             │
│  Message Log (scrollable)    │  Side Panel (collapsible)   │
│  55–65% width               │  35–45% width               │
│                              │                             │
│                              │  Content: Inspection,       │
│                              │  Bindings, or other         │
│                              │  context panels             │
├──────────────────────────────┤                             │
│  Composer                    │                             │
│  (within message log column) │                             │
└──────────────────────────────┴─────────────────────────────┘
```

**Side Panel:**
- Collapsible. When collapsed, message log takes full width.
- Opens when: a message is clicked (inspection), or user opens bindings, routing, cursors, or templates from the nav rail while a workspace is active.
- Collapse/expand toggle: small arrow icon at the panel's left edge.
- Min width: 300px. Max width: 600px. Resizable via drag handle.

**Routing and Template editor views (full-tab):**
- These are complex enough to take the full content area (not just the side panel).
- When the user navigates to Routing from the nav rail, the message log + side panel layout is replaced by the edge policy editor's dual-zone layout (§3.9).

### 4.3 Panel Collapse/Expand Behavior

**Transition:** `duration-normal`, `easing-standard`. Width animates smoothly. Content within the collapsing panel fades out at `duration-fast` before the width begins animating.

**Collapsed state indicator:** A thin vertical strip (4px wide) with a subtle expand-arrow icon, using `border-subtle` color. Hovering the strip widens it to 8px with `surface-hover`.

**Keyboard:** A keyboard shortcut toggles the side panel (configurable via `commands` API).

### 4.4 Navigation Between Major Views

**Workspace list → Active workspace:** In sidebar, pushes onto stack. In full-tab, the workspace list is the nav rail's "Workspaces" destination; selecting a workspace switches the content area.

**Active workspace → Settings:** In sidebar, settings pushes as an overlay. In full-tab, settings takes the full content area (nav rail "Settings" item is active).

**Active workspace → Diagnostics:** In sidebar, diagnostics pushes as an overlay. In full-tab, diagnostics can be shown in the side panel (filtered to current workspace) or as a full content area (global diagnostics view, nav rail "Diagnostics" item).

**Return to active workspace:** In sidebar, back button. In full-tab, nav rail "Active workspace" icon, which always returns to message log + composer.

---

## 5. Interaction Patterns

### 5.1 Switching Orchestration Modes

**Where:** Mode badge in workspace header (click to open mode selector), or in run configuration before starting a run.

**Flow:**
1. User clicks mode badge or "Configure Run" action.
2. A mode selector panel appears (dropdown in sidebar, popover in full-tab).
3. Options presented as a list of radio items, each with:
   - Mode name (`type-body-strong`).
   - One-line description (`type-caption`, `text-secondary`).
   - Relevant icon.
4. Selecting a mode updates the workspace default immediately if no run is active.
5. If a run is active and paused, changing the mode applies to the next round (with a confirmation toast).
6. If a run is active and running, the selector is disabled with a note: "Pause the run to change the mode."

**Mode categories in the selector:**
- **Manual** section: Broadcast, Directed, Relay-to-One, Relay-to-Many, Draft Only, Copy Only.
- **Autonomous** section: Roundtable, Moderator/Jury, Relay Chain.
- **Moderated Autonomous** toggle: appears when an autonomous mode is selected. Toggles moderated behavior (pause between rounds).

### 5.2 Multi-Phase Workflow (Run Transitions)

This is the pattern for transitioning from one orchestration phase to another within the same workspace — e.g., from independent drafts (broadcast) to conflict mapping (directed) to synthesis (moderator).

**Flow:**
1. User completes (or stops) the current run.
2. Workspace header updates: run status shows "Completed" or "Stopped".
3. Run controls bar shows "New Run" button.
4. User clicks "New Run". The run configuration sheet (§3.24) appears, pre-populated with current workspace settings.
5. User adjusts mode, targets, edge policies, stop conditions, and timing for the new phase.
6. User clicks "Start Run".
7. The new run begins. Previous run's messages remain in the log with their round badges. New run's rounds are numbered starting from 1, prefixed with a visual "Run 2" divider in the message log.

**Run Divider in Message Log:**
- A prominent horizontal divider with "Run 2 — Roundtable" label (`type-subtitle`, `text-secondary`), centered. Timestamp of run start below in `type-caption`.

### 5.3 Export Selection Interaction

**Entering selection mode:**
- User triggers export (toolbar, keyboard shortcut, or overflow menu).
- If scope is "Selected Messages", the export dialog minimizes to a floating toolbar, and the message log enters selection mode.

**Selection mechanics:**
- **Click checkbox:** Toggles that single message.
- **Shift+click:** Selects the range from the last selected message to the clicked one (inclusive).
- **"Select All by Participant" dropdown:** Shows list of participants. Selecting one checks all messages from that participant. Multiple participants can be selected.
- **"Select All":** Checks every visible message.
- **"Invert":** Flips all checkboxes.
- **"Clear":** Unchecks everything.

**Floating selection toolbar:**
- Sticks to the bottom of the message log, `surface-overlay`, `shadow-md`.
- Content: "23 messages selected" count + participant filter buttons + "Invert" + "Clear" + "Continue to Export" primary button.

**Exiting selection mode:**
- "Continue to Export" returns to the export dialog with selection locked.
- "Cancel" or Escape exits selection mode, restoring normal message log behavior.

### 5.4 Edge Policy Configuration

**Entry points:**
1. Sidebar: "Routing" option in workspace overflow menu → pushes routing editor overlay.
2. Full-tab: Nav rail "Routing" icon → full content area routing editor.
3. From run configuration sheet → "Edit Routing" link → opens routing editor contextually.

**Graph interaction (full-tab):**
- Nodes are draggable for user-preferred arrangement (positions persisted per workspace).
- Clicking a node highlights all its outbound and inbound edges.
- Clicking an edge selects it and populates the edge detail panel.
- Double-clicking empty space deselects.

**Edge list interaction (sidebar):**
- Scrollable list of edges, each row: source icon → target icon, toggle switch, expand chevron.
- Toggle switch enables/disables the edge inline.
- Expand chevron shows the edge detail form within the list.

**Source → Target visualization:**
- Each edge is labeled "{Source} → {Target}" with provider icons.
- In the graph view, arrows flow from source to target. Arrow thickness or style does not vary — all enabled edges look the same (solid), all disabled edges look the same (dashed).

### 5.5 Between-Rounds Editing (Moderated Autonomous)

**Flow:**
1. An autonomous run completes a round. If moderated mode is enabled, the run auto-pauses.
2. The run controls bar updates: status shows "Paused" with "Round {N} complete — review before continuing" in `type-caption`.
3. The "Edit Package" button appears (see §3.7).
4. User clicks "Edit Package". The between-rounds review surface (§3.23) opens.
5. The user can:
   - Review the rendered packages for each target.
   - Edit any package's text.
   - Remove context blocks from packages.
   - Skip specific targets for the next round.
   - Inject a user message that will appear in the log and be included as context.
   - Make temporary edge policy overrides for the next round.
   - Change target sets or navigate to the full routing editor for persistent changes.
6. User clicks "Resume with these changes" → the run resumes with the edited packages.
7. Alternatively, "Resume without changes" sends the originally generated packages.
8. If the user needs to change the orchestration mode entirely (e.g., switch from roundtable to moderator), they should "Stop" the run and use "New Run" (§5.2) to start a new phase with a different topology.

### 5.6 Package Preview and Inline Editing

**Flow:**
1. User types a message in the composer and selects targets.
2. User clicks the eye icon (package preview toggle).
3. The package preview panel (§3.6) slides open above the composer input.
4. The preview shows the rendered payload: preamble + context blocks + user message, using the selected template.
5. Each context block (from another provider's messages) is visually delineated by a subtle border and has a remove (X) icon.
6. The user can:
   - Read the full rendered payload.
   - Click into the preview text area to edit directly. The border turns to `status-warning-border` to indicate manual edits.
   - Remove individual context blocks by clicking (X).
   - Switch templates via the template dropdown — the preview re-renders, but manual edits are lost (with a confirmation: "Switching templates will discard manual edits. Continue?").
   - Copy the preview to clipboard.
7. User clicks "Send" (or "Draft" / "Copy"). The edited payload is used.

### 5.7 Provider Binding

**Auto-detect flow:**
1. When the workspace is first opened, or when the user clicks "Detect Provider Tabs" in the empty bindings state:
2. Chatmux scans for open tabs matching known provider URL patterns (§6 of PRD).
3. For each detected provider tab:
   - If the extension has host permission: runs structural probe, if probe passes → auto-binds.
   - If permission is missing: shows the permission request flow (§3.16).
4. Results appear in the provider binding cards with updated health states.

**Manual bind flow:**
1. User clicks "Rebind" on a binding card, or "Bind" on an unbound card.
2. A dropdown lists open tabs matching that provider's URL patterns.
3. User selects a tab → binding is established.
4. If no matching tabs are open: "Open {Provider}" button launches a new tab to the provider's URL.

**Rebind after tab reload:**
- When a bound provider's tab is reloaded, the content script reinjects and re-runs the structural probe.
- If the probe passes, the binding reconnects silently.
- If the probe fails, the binding transitions to `DomMismatch` health state.
- A toast notifies the user of rebind success or failure.

**Multiple candidate tabs:**
- If multiple tabs match a provider's URL patterns, the binding dropdown shows all candidates with their page titles and URLs.
- The user selects one. The others remain available for future rebinding or for binding in another workspace.

### 5.8 Permission Request Integration

**First-use flow:**
1. User installs Chatmux, opens the sidebar/tab.
2. Empty workspace list is shown. User creates a workspace.
3. Empty bindings state is shown. User clicks "Detect Provider Tabs" or attempts to bind a provider.
4. For each provider that needs permission: the permission request overlay (§3.16) is shown in sequence.
5. Each provider is an independent permission — the user can grant some and skip others.

**Later activation:**
- If the user skips a provider during first use, the provider's binding card shows "Permission Missing" health state.
- The card includes a "Grant Permission" button that triggers the same permission overlay.

**Revoked permission:**
- If the user revokes a host permission externally (via browser settings), the next structural probe or send attempt will detect the missing permission.
- The binding card updates to "Permission Missing" with the grant action.

### 5.9 Diagnostics Surfacing

**Three tiers of diagnostics visibility:**

**Tier 1 — Inline Indicators (lowest disruption):**
- Provider binding card health badges change color and label based on health state.
- Message cards show status badges for dispatch outcomes (timeout, error, uncertain capture).
- Workspace header provider status dots change color.
- These are always visible and require no user action to see.

**Tier 2 — Toasts (medium disruption):**
- Provider status changes (Ready → Disconnected, etc.) generate a toast.
- Run state changes generate a toast.
- These appear briefly and auto-dismiss.

**Tier 3 — Diagnostics Panel (full detail):**
- All diagnostic events are persisted and viewable in the diagnostics panel (§3.14).
- Accessible from nav rail, workspace overflow menu, or the health badge on a binding card.
- Filter, search, and inspect full event details.

**Escalation:**
- If a critical diagnostic event occurs during an autonomous run (e.g., a provider enters `SendFailed` or `DomMismatch`), the system:
  1. Pauses the run.
  2. Shows a persistent (non-auto-dismissing) error toast with "Run paused — {Provider} encountered an error."
  3. The run controls bar updates to show the paused state.
  4. The binding card shows the error.

---

## 6. Iconography Requirements

Every icon listed with its semantic meaning and usage locations. Icons should be sourced from a consistent icon set (outlined style, 1.5–2px stroke weight, matching the design's clean-technical aesthetic).

### Navigation and Structure

| Icon | Meaning | Where Used |
|------|---------|-----------|
| Grid / four squares | Workspaces list | Nav rail, breadcrumb |
| Chat bubble | Active workspace / messages | Nav rail |
| Git-branch / fork | Routing / edge policies | Nav rail, workspace menu |
| Document | Templates | Nav rail, template manager |
| Shield with checkmark | Diagnostics (healthy) | Nav rail, empty diagnostics state |
| Shield with exclamation | Diagnostics (issues exist) | Nav rail badge |
| Gear / cog | Settings | Nav rail, global header |
| Arrow left | Back / return | Sidebar stack navigation |
| Chevron down | Expand / dropdown indicator | Dropdowns, collapsible sections |
| Chevron right | Expand / navigate deeper | List items, collapsible rows |
| X / close | Close panel, dismiss, remove | Panel headers, toasts, dialog dismiss |

### Actions

| Icon | Meaning | Where Used |
|------|---------|-----------|
| Plus | Create new (workspace, template, etc.) | CTA buttons, list headers |
| Pencil | Edit / rename | Inline edit triggers, template edit |
| Duplicate / copy-document | Duplicate | Workspace menu, template menu |
| Archive box (down arrow into box) | Archive workspace | Workspace menu |
| Trash can | Delete | Workspace menu, template menu, confirmation dialogs |
| Download arrow | Download / export file | Export dialog |
| Clipboard | Copy to clipboard | Composer, export, package preview |
| Clipboard with checkmark | Copy confirmed | Temporary state after copy |
| Eye | Preview | Package preview toggle |
| Eye with slash | Hide preview | Package preview toggle (active state) |
| Search / magnifying glass | Search | Message log search, search/filter bar |
| Funnel / filter | Filter | Search/filter bar, diagnostics filter, export selection filter |
| Arrow up / arrow down | Navigate search results | Search/filter bar result navigation |
| Bookmark / save | Save filter | Search/filter bar saved filters |

### Run Controls

| Icon | Meaning | Where Used |
|------|---------|-----------|
| Play triangle | Start / resume | Run controls |
| Pause (two bars) | Pause | Run controls |
| Step (play with line) | Step one round | Run controls |
| Stop (square) | Stop | Run controls |
| Stop octagon | Abort | Run controls, kill switch |
| Rewind (double arrows) | Reset cursor | Cursor inspector |
| Snowflake | Freeze cursor | Cursor inspector |

### Provider Health

| Icon | Meaning | Where Used |
|------|---------|-----------|
| Filled circle | Ready / healthy | Binding card |
| Pencil in circle | Composing | Binding card |
| Arrow outgoing | Sending | Binding card |
| Three animated dots / spinner | Generating | Binding card |
| Checkmark in circle | Completed | Binding card, dispatch success |
| Broken link | Disconnected | Binding card |
| Lock / padlock | Permission missing | Binding card |
| Person with X | Login required | Binding card |
| Warning triangle | DOM mismatch / critical error | Binding card, error banners |
| Stop circle | Blocked | Binding card |
| Clock with slash | Rate limited | Binding card |
| X in circle | Send failed | Binding card, message dispatch badge |
| Question mark in circle | Capture uncertain | Binding card, message status |
| Hand (raised) | Degraded / manual only | Binding card |

### Messaging and Composition

| Icon | Meaning | Where Used |
|------|---------|-----------|
| Send / paper plane | Send action | Composer send button |
| Document outline | Draft-only mode | Composer mode selector |
| Clipboard (outlined) | Copy-only mode | Composer mode selector |
| Checkmark (small) | Selected target / item | Target selector chips, checkboxes |
| Tag | Message tags | Message metadata, filter |
| Crosshair / target | Pick messages for context | Composer context pick button |
| Pin | Pinned note / pinned summary | Composer notes bar, pinned summary manager |
| Drag handle (six dots) | Reorderable item | Relay chain order in run config |
| Strikethrough circle | Disabled participant | Run controls active participants row |

### Export

| Icon | Meaning | Where Used |
|------|---------|-----------|
| Document with lines | Markdown format | Export format picker |
| Curly braces `{ }` | JSON format | Export format picker |
| Equals/key-value | TOML format | Export format picker |
| Save / floppy disk | Save profile | Export profile management |
| Folder open | Load profile | Export profile management |

### System and Status

| Icon | Meaning | Where Used |
|------|---------|-----------|
| Info circle | Informational | Diagnostics, tooltips |
| Exclamation in triangle | Warning | Error banners, diagnostics |
| Exclamation in circle | Error / critical | Error banners, diagnostics |
| Checkmark in shield | All clear | Empty diagnostics state |
| Person silhouette | User participant | Message attribution |
| Gear (small) | System participant | Message attribution |
| Puzzle piece | Provider / integration | Empty bindings state |
| Stacked rectangles | Multiple items / workspaces | Empty workspace state |
| Bar chart / meter | Storage usage | Settings storage indicator |
| Keyboard | Shortcuts | Settings, help reference |
| External link | Open in new tab | Binding card "Open Tab" action |

---

## 7. Motion and Transitions

### 7.1 Panel Open/Close

**Side panel (full-tab):**
- Open: Panel width animates from 0 to target width (300–600px) over `duration-normal` with `easing-enter`. Content fades in from 0% to 100% opacity over `duration-fast`, starting when the panel reaches ~50% of target width.
- Close: Content fades to 0% opacity over `duration-fast`. Once faded, width animates to 0 over `duration-normal` with `easing-exit`.

**Overlay panel (sidebar):**
- Open: Panel slides in from the right edge. Translate-X from 100% to 0 over `duration-normal` with `easing-enter`. A scrim fades in behind at 20% black opacity.
- Close: Reverse of open. Translate-X 0 → 100% over `duration-normal` with `easing-exit`. Scrim fades out.

**Collapsible sections:**
- Expand: Height animates from 0 to content height over `duration-normal` with `easing-standard`. Content fades in.
- Collapse: Content fades out over `duration-fast`, then height animates to 0 over `duration-normal`.

### 7.2 Message Arrival

**New message appearing in the log:**
1. A placeholder shimmer area appears at the expected position (matching estimated height) with the provider's `provider-{name}-muted` color.
2. When content is ready, the shimmer cross-fades to the actual message card over `duration-gentle`.
3. The message card enters with a subtle translate-Y from 8px to 0 (sliding up into place) over `duration-gentle` with `easing-enter`.
4. If the user is scrolled to the bottom, the log auto-scrolls to keep the new message visible. If the user has scrolled up, a "New messages ↓" indicator appears at the bottom of the log.

**Multiple messages arriving simultaneously (e.g., broadcast responses):**
- Messages stagger their entrance by 80ms each, creating a cascade effect rather than all appearing at once.

### 7.3 Run State Transitions

**Idle → Running:**
- Run controls bar slides in from above (translate-Y from -100% to 0) over `duration-normal`.
- "Start" button transitions: label fades to "Running", background color cross-fades from `accent-primary` to `surface-sunken`.
- Provider status dots in the workspace header begin a slow pulse animation (opacity oscillates 70%–100% over 2 seconds, looping).

**Running → Paused:**
- Provider status dots stop pulsing (snap to 100% opacity).
- Run status indicator dot transitions from pulsing `status-success` to static `status-warning` over `duration-fast`.
- "Pause" button transitions to "Resume".

**Running/Paused → Completed:**
- A brief checkmark animation plays in the run status area: a small circle expands from the status dot position and fades out, while the dot transitions to `status-info`.
- A toast confirms: "Run completed — {N} rounds."

**Abort:**
- All animations stop immediately (no graceful transition — abort should feel immediate).
- Status dot snaps to `status-error`.
- Run controls bar buttons update instantly.

### 7.4 Provider Status Change

- When a provider's health state changes, the health badge on the binding card:
  1. Briefly scales up to 110% over `duration-fast` with `easing-spring`.
  2. Color cross-fades to the new status color over `duration-fast`.
  3. Scales back to 100% over `duration-fast`.
- The workspace header provider status dot color cross-fades over `duration-fast`.
- A toast fires (see §3.21).

### 7.5 Toast Appearance/Dismissal

**Appearance:**
- Toast slides in from the right (translate-X from 100% to 0) over `duration-gentle` with `easing-enter`.
- Simultaneously fades from 0% to 100% opacity.

**Dismissal (auto or manual):**
- Toast fades to 0% opacity over `duration-normal` with `easing-exit`.
- After fade, the toast is removed from the DOM and remaining toasts slide up to fill the gap over `duration-normal`.

**Stacking update:**
- When a new toast pushes the stack, existing toasts translate-Y upward by the new toast's height over `duration-normal`.

### 7.6 Selection Mode Transitions

**Entering export selection mode:**
- Checkboxes appear on each message card: they fade in from 0% opacity and simultaneously each card's left padding animates to accommodate the checkbox (padding-left increases by ~28px) over `duration-normal`.
- The floating selection toolbar slides up from below the viewport over `duration-normal`.

**Exiting selection mode:**
- Reverse of entering: checkboxes fade out, padding returns, toolbar slides down.

### 7.7 Mode/Strategy Badge Transitions

When the orchestration mode or context strategy changes:
- The badge content cross-fades: old text fades to 0% while new text fades in, over `duration-fast`.
- Badge background color cross-fades if the new mode/strategy has a different associated color.

---

## 8. Accessibility Considerations

These are not optional polish — they inform every component specification above.

### 8.1 Color Contrast

- All `text-primary` on `surface-base` and `surface-raised` must meet WCAG 2.1 AA contrast ratio (≥4.5:1 for normal text, ≥3:1 for large text).
- All `text-secondary` on `surface-base` must meet ≥3:1 minimum.
- Provider identity colors used for text (`provider-{name}-text`) must meet ≥4.5:1 on both `surface-base` and `surface-raised`.
- Status colors used for text must meet the same thresholds.
- Never rely on color alone to communicate state. Every colored status indicator is paired with a label or icon.

### 8.2 Focus Management

- All interactive elements must have a visible focus indicator: a 2px ring offset by 2px, using `accent-primary` color.
- Focus order follows visual reading order (left-to-right, top-to-bottom).
- When panels open, focus moves to the first interactive element in the panel.
- When dialogs open, focus is trapped within the dialog until dismissed.
- When a dialog is dismissed, focus returns to the element that triggered it.

### 8.3 Keyboard Navigation

- All actions available via mouse must be available via keyboard.
- Tab navigates between interactive elements.
- Enter/Space activates buttons and toggles.
- Arrow keys navigate within composite widgets (selector lists, tab bars, radio groups).
- Escape closes the nearest open overlay, dialog, or dropdown.

### 8.4 Screen Reader Support

- All icons are accompanied by text labels or `aria-label` equivalents.
- Provider identity is communicated via text (not just color/icon).
- Health states are announced as text labels.
- Dynamic content changes (toasts, message arrivals, status changes) use live regions.
- The message log is structured as a list with message landmarks.

### 8.5 Reduced Motion

- When the user has `prefers-reduced-motion: reduce`, all animations resolve instantly (duration: 0). Opacity transitions are retained at `duration-fast` for visual continuity but all translate/scale animations are removed.

---

*End of Design Specification.*
