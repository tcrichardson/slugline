# Header Redesign — 2026-06-25

## Overview

Three targeted visual changes to the application header: remove the app name, increase the clock font size, and flatten the tab styling so the active tab matches the editor's edit-bar highlight color.

## Changes

### 1. Remove application name

- Delete `<div class="name">Slugline</div>` from `Header.svelte`
- Delete the `.name` CSS rule from `Header.svelte`
- The tab bar expands left to fill the vacated space naturally via flex layout

### 2. Increase clock font size

- Change `.clock` `font-size` from `0.78rem` → `1.1rem` in `Header.svelte`
- All other clock styles remain unchanged (column layout, right-align, `--muted` color, tabular-nums on time span)

### 3. Flat tabs with edit-bar active state

In `Tabs.svelte`:

- Remove `border-radius: 6px 6px 0 0` from `.tab` (straight rectangular edges)
- Replace `.tab.active` styles:
  - Remove `background: var(--bg)`
  - Remove `box-shadow: inset 0 -2px 0 var(--accent)`
  - Set `background: var(--edit-bar-bg)` (resolves to `#e2ebff`)
  - Keep `color: var(--fg)`

## Files

| File | Change |
|---|---|
| `web/src/lib/components/Header.svelte` | Remove `.name` div + CSS; change clock font-size |
| `web/src/lib/components/Tabs.svelte` | Flatten tab border-radius; update active tab bg/shadow |

## Out of Scope

- No changes to `app.css` (no new tokens needed; `--edit-bar-bg` already exists)
- No changes to `EditorPane.svelte`, `StatusLine.svelte`, or `App.svelte`
- No changes to tab logic or behavior
