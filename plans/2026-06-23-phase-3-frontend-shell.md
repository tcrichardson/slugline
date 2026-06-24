# Phase 3: Frontend App Shell — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Prerequisites:** Phase 1 (the `web/` Vite+Svelte+TS scaffold with `src/lib/doc/` modules and Vitest) and Phase 2 (the runnable `axum` API) must be complete.

**Goal:** Build the Svelte application shell that ties the API and config to a usable window: an API client, pure tab/navigation/date logic, theme-token application, a header wall-clock, a calendar widget with has-note dots and two-way date sync, and a read-only editor-pane placeholder — all served by the Phase 2 binary.

**Architecture:** All branching logic (tabs, dates, theme tokens, API) lives in **pure, DOM-free TypeScript modules** unit-tested with Vitest. A thin Svelte 5 runes store (`appState.svelte.ts`) holds reactive state and delegates to those pure modules. Svelte components are the integration layer, verified by `svelte-check` and manual run. The editor pane is a read-only `<pre>` placeholder; real editing arrives in Phase 4.

**Tech Stack:** Svelte 5 (runes), Vite (dev proxy `/api` → `127.0.0.1:4747`), TypeScript, Vitest.

> **Scope note / roadmap refinement:** This phase wires calendar clicks to navigation (open/retarget, which materializes notes via the Phase 2 `GET`), so "click-to-open/create" is delivered here rather than in Phase 5. Phase 5 then only adds the Agenda and To Do panels.

---

## File Structure

| File | Responsibility |
|---|---|
| `web/vite.config.ts` (modify) | Add dev proxy for `/api` |
| `web/src/app.css` (replace) | Base layout + CSS-variable fallbacks |
| `web/src/App.svelte` (replace) | Top-level layout + global shortcuts |
| `web/src/lib/dates.ts` | Pure date math: `todayISO`, `addDays`, `monthGrid`, `yearMonth`, `isValidDate` |
| `web/src/lib/tabs.ts` | Pure tab reducer: open/retarget/de-dup/close/next/prev |
| `web/src/lib/theme.ts` | Built-in token maps + `resolveTokens` (pure) + `applyTheme` (DOM) |
| `web/src/lib/types.ts` | `UiConfig` shape (matches `/api/config`) |
| `web/src/lib/api.ts` | `listNotes`/`getNote`/`putNote`/`getConfig` fetch wrappers |
| `web/src/lib/appState.svelte.ts` | Runes store; delegates to pure modules |
| `web/src/lib/components/Header.svelte` | App name + wall-clock + tab bar |
| `web/src/lib/components/Tabs.svelte` | Tab rendering, switch/close |
| `web/src/lib/components/Calendar.svelte` | Month grid, dots, today/selected markers, month nav |
| `web/src/lib/components/Sidebar.svelte` | Calendar + Agenda/To Do placeholders |
| `web/src/lib/components/EditorPane.svelte` | Read-only raw-note placeholder |
| `Makefile` (modify) | Add `dev-web` target |

All commands below run from `web/` unless stated otherwise.

---

## Task 1: Vite dev proxy, base styles, and a minimal shell

**Files:**
- Modify: `web/vite.config.ts`
- Replace: `web/src/app.css`, `web/src/App.svelte`

- [ ] **Step 1: Add the dev proxy to `web/vite.config.ts`**

Replace the file contents with:
```ts
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:4747',
    },
  },
});
```

- [ ] **Step 2: Replace `web/src/app.css` with base styles + CSS-variable fallbacks**

```css
:root {
  --bg: #fbfcfe;
  --fg: #1b2330;
  --muted: #5b6675;
  --accent: #2f6df6;
  --heading-1: #1d4ed8;
  --heading-2: #2563eb;
  --heading-3: #3b82f6;
  --heading-4: #60a5fa;
  --heading-5: #7dabfb;
  --heading-6: #9cc2fc;
  --todo-done: #8a93a3;
  --meta: #6b7686;
  --status-bar: #eef2f9;
  --edit-line-bg: #eaf1ff;
  --cursor: #1b2330;
  --font: Roboto;
}

* { box-sizing: border-box; }

html, body { margin: 0; height: 100%; }

body {
  background: var(--bg);
  color: var(--fg);
  font-family: var(--font), system-ui, -apple-system, sans-serif;
}

#app { height: 100vh; }
```

- [ ] **Step 3: Replace `web/src/App.svelte` with a minimal placeholder shell**

```svelte
<script lang="ts"></script>

<div class="shell">
  <h1>Slugline</h1>
  <p>App shell under construction.</p>
</div>

<style>
  .shell { padding: 1rem; }
</style>
```
(This removes the create-vite demo imports. The demo `src/lib/Counter.svelte` and `src/assets/*` become orphaned and may be deleted, but leaving them does not break the build.)

- [ ] **Step 4: Verify the build and existing tests still pass**

Run (from `web/`):
```bash
npm run build && npm test
```
Expected: Vite build succeeds; Phase 1 `doc/*` tests still PASS.

- [ ] **Step 5: Commit**

```bash
git add web/vite.config.ts web/src/app.css web/src/App.svelte
git commit -m "chore: add vite api proxy, base theme css vars, minimal shell"
```

---

## Task 2: Pure date utilities

**Files:**
- Create: `web/src/lib/dates.ts`, `web/src/lib/dates.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/dates.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { isValidDate, addDays, todayISO, monthGrid, yearMonth } from './dates';

describe('dates', () => {
  it('validates ISO calendar dates', () => {
    expect(isValidDate('2026-06-23')).toBe(true);
    expect(isValidDate('2026-02-30')).toBe(false);
    expect(isValidDate('2026-6-23')).toBe(false);
  });

  it('adds days across month/year boundaries', () => {
    expect(addDays('2026-12-31', 1)).toBe('2027-01-01');
    expect(addDays('2026-03-01', -1)).toBe('2026-02-28');
  });

  it('formats today from a fixed date', () => {
    expect(todayISO(new Date(2026, 5, 23, 9, 0))).toBe('2026-06-23');
  });

  it('builds a 6x7 month grid with the first of month and out-of-month days', () => {
    const g = monthGrid(2026, 6);
    expect(g.length).toBe(6);
    expect(g[0].length).toBe(7);
    const flat = g.flat();
    expect(flat.find((c) => c.date === '2026-06-01')!.inMonth).toBe(true);
    expect(flat.some((c) => !c.inMonth)).toBe(true);
  });

  it('extracts year/month', () => {
    expect(yearMonth('2026-06-23')).toEqual({ year: 2026, month: 6 });
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./dates`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/dates.ts`:
```ts
const ISO = /^\d{4}-\d{2}-\d{2}$/;

export function isValidDate(s: string): boolean {
  if (!ISO.test(s)) return false;
  const [y, m, d] = s.split('-').map(Number);
  const dt = new Date(Date.UTC(y, m - 1, d));
  return dt.getUTCFullYear() === y && dt.getUTCMonth() === m - 1 && dt.getUTCDate() === d;
}

/** Format a Date as a LOCAL `YYYY-MM-DD` string. */
export function toISODate(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

export function todayISO(now: Date = new Date()): string {
  return toISODate(now);
}

/** Add `n` days to an ISO date string, returning a new ISO date string. */
export function addDays(date: string, n: number): string {
  const [y, m, d] = date.split('-').map(Number);
  const dt = new Date(Date.UTC(y, m - 1, d));
  dt.setUTCDate(dt.getUTCDate() + n);
  const yy = dt.getUTCFullYear();
  const mm = String(dt.getUTCMonth() + 1).padStart(2, '0');
  const dd = String(dt.getUTCDate()).padStart(2, '0');
  return `${yy}-${mm}-${dd}`;
}

export interface MonthCell {
  date: string;
  inMonth: boolean;
}

/** A 6x7 grid (weeks start Sunday) covering the month. `month` is 1-12. */
export function monthGrid(year: number, month: number): MonthCell[][] {
  const first = new Date(Date.UTC(year, month - 1, 1));
  const cursor = new Date(first);
  cursor.setUTCDate(1 - first.getUTCDay());

  const weeks: MonthCell[][] = [];
  for (let w = 0; w < 6; w++) {
    const row: MonthCell[] = [];
    for (let i = 0; i < 7; i++) {
      const yy = cursor.getUTCFullYear();
      const mm = String(cursor.getUTCMonth() + 1).padStart(2, '0');
      const dd = String(cursor.getUTCDate()).padStart(2, '0');
      row.push({ date: `${yy}-${mm}-${dd}`, inMonth: cursor.getUTCMonth() === month - 1 });
      cursor.setUTCDate(cursor.getUTCDate() + 1);
    }
    weeks.push(row);
  }
  return weeks;
}

export function yearMonth(date: string): { year: number; month: number } {
  const [y, m] = date.split('-').map(Number);
  return { year: y, month: m };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `dates` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/dates.ts web/src/lib/dates.test.ts
git commit -m "feat: add pure date utilities (validation, add-days, month grid)"
```

---

## Task 3: Pure tab reducer

**Files:**
- Create: `web/src/lib/tabs.ts`, `web/src/lib/tabs.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/tabs.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { initTabs, retarget, openNewTab, closeTab, nextTab, prevTab, activeDate } from './tabs';

describe('tabs', () => {
  it('initializes with today as the only tab', () => {
    const s = initTabs('2026-06-23');
    expect(s.tabs).toEqual(['2026-06-23']);
    expect(activeDate(s)).toBe('2026-06-23');
  });

  it('retargets the active tab in place', () => {
    const s = retarget(initTabs('2026-06-23'), '2026-06-22');
    expect(s.tabs).toEqual(['2026-06-22']);
    expect(s.activeIndex).toBe(0);
  });

  it('focuses an existing tab instead of duplicating on retarget', () => {
    let s = openNewTab(initTabs('2026-06-23'), '2026-06-22'); // [23, 22] active 1
    s = retarget(s, '2026-06-23'); // 23 already open at index 0
    expect(s.tabs).toEqual(['2026-06-23', '2026-06-22']);
    expect(s.activeIndex).toBe(0);
  });

  it('opens new tabs appended right and focuses them', () => {
    const s = openNewTab(initTabs('2026-06-23'), '2026-06-24');
    expect(s.tabs).toEqual(['2026-06-23', '2026-06-24']);
    expect(s.activeIndex).toBe(1);
  });

  it('closes a tab and always keeps at least one', () => {
    let s = openNewTab(initTabs('2026-06-23'), '2026-06-24'); // [23, 24] active 1
    s = closeTab(s, 1, '2026-06-25');
    expect(s.tabs).toEqual(['2026-06-23']);
    expect(s.activeIndex).toBe(0);
    s = closeTab(s, 0, '2026-06-25'); // removing the last falls back to today
    expect(s.tabs).toEqual(['2026-06-25']);
    expect(s.activeIndex).toBe(0);
  });

  it('cycles tabs with wrap-around', () => {
    let s = openNewTab(initTabs('a'), 'b'); // [a, b] active 1
    s = nextTab(s);
    expect(s.activeIndex).toBe(0);
    s = prevTab(s);
    expect(s.activeIndex).toBe(1);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./tabs`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/tabs.ts`:
```ts
export interface TabsState {
  tabs: string[]; // date strings, de-duplicated
  activeIndex: number;
}

export function initTabs(today: string): TabsState {
  return { tabs: [today], activeIndex: 0 };
}

export function activeDate(state: TabsState): string {
  return state.tabs[state.activeIndex];
}

/** Retarget the active tab to `date` in place; if `date` is already open, focus that tab. */
export function retarget(state: TabsState, date: string): TabsState {
  const existing = state.tabs.indexOf(date);
  if (existing !== -1) {
    return { tabs: state.tabs, activeIndex: existing };
  }
  const tabs = state.tabs.slice();
  tabs[state.activeIndex] = date;
  return { tabs, activeIndex: state.activeIndex };
}

/** Open `date` in a new tab (appended right) and focus it; focus an existing tab if present. */
export function openNewTab(state: TabsState, date: string): TabsState {
  const existing = state.tabs.indexOf(date);
  if (existing !== -1) {
    return { tabs: state.tabs, activeIndex: existing };
  }
  const tabs = [...state.tabs, date];
  return { tabs, activeIndex: tabs.length - 1 };
}

/** Close the tab at `index`; guarantees >=1 tab by falling back to `today`. */
export function closeTab(state: TabsState, index: number, today: string): TabsState {
  if (index < 0 || index >= state.tabs.length) return state;
  const tabs = state.tabs.slice();
  tabs.splice(index, 1);
  if (tabs.length === 0) {
    return { tabs: [today], activeIndex: 0 };
  }
  let activeIndex = state.activeIndex;
  if (index < activeIndex) activeIndex -= 1;
  else if (index === activeIndex) activeIndex = Math.min(activeIndex, tabs.length - 1);
  return { tabs, activeIndex };
}

export function nextTab(state: TabsState): TabsState {
  return { tabs: state.tabs, activeIndex: (state.activeIndex + 1) % state.tabs.length };
}

export function prevTab(state: TabsState): TabsState {
  const n = state.tabs.length;
  return { tabs: state.tabs, activeIndex: (state.activeIndex - 1 + n) % n };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `tabs` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/tabs.ts web/src/lib/tabs.test.ts
git commit -m "feat: add pure tab reducer (open/retarget/dedup/close/cycle)"
```

---

## Task 4: Theme tokens

**Files:**
- Create: `web/src/lib/theme.ts`, `web/src/lib/theme.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/theme.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { resolveTokens, LIGHT, DARK } from './theme';

describe('theme', () => {
  it('returns built-in light tokens by default', () => {
    expect(resolveTokens('light')['--bg']).toBe(LIGHT['--bg']);
  });

  it('returns dark tokens for the dark theme', () => {
    expect(resolveTokens('dark')['--bg']).toBe(DARK['--bg']);
  });

  it('falls back to light for unknown themes', () => {
    expect(resolveTokens('neon')['--bg']).toBe(LIGHT['--bg']);
  });

  it('applies per-theme config overrides over the base', () => {
    const t = resolveTokens('dark', { dark: { '--bg': '#000000' } });
    expect(t['--bg']).toBe('#000000');
    expect(t['--fg']).toBe(DARK['--fg']);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./theme`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/theme.ts`:
```ts
export type Tokens = Record<string, string>;

export const LIGHT: Tokens = {
  '--bg': '#fbfcfe',
  '--fg': '#1b2330',
  '--muted': '#5b6675',
  '--accent': '#2f6df6',
  '--heading-1': '#1d4ed8',
  '--heading-2': '#2563eb',
  '--heading-3': '#3b82f6',
  '--heading-4': '#60a5fa',
  '--heading-5': '#7dabfb',
  '--heading-6': '#9cc2fc',
  '--todo-done': '#8a93a3',
  '--meta': '#6b7686',
  '--status-bar': '#eef2f9',
  '--edit-line-bg': '#eaf1ff',
  '--cursor': '#1b2330',
};

export const DARK: Tokens = {
  '--bg': '#161a26',
  '--fg': '#e7ecf5',
  '--muted': '#97a1b3',
  '--accent': '#6f9bff',
  '--heading-1': '#9cc2fc',
  '--heading-2': '#7dabfb',
  '--heading-3': '#60a5fa',
  '--heading-4': '#3b82f6',
  '--heading-5': '#2563eb',
  '--heading-6': '#1d4ed8',
  '--todo-done': '#6b7686',
  '--meta': '#97a1b3',
  '--status-bar': '#1f2535',
  '--edit-line-bg': '#222a3d',
  '--cursor': '#e7ecf5',
};

export function builtinTokens(theme: string): Tokens {
  return theme === 'dark' ? { ...DARK } : { ...LIGHT };
}

/** Merge built-in tokens with per-theme overrides from config. */
export function resolveTokens(
  theme: string,
  overrides: Record<string, Record<string, string>> = {},
): Tokens {
  return { ...builtinTokens(theme), ...(overrides[theme] ?? {}) };
}

/** Apply tokens + font to the document root. DOM side-effect; not unit-tested. */
export function applyTheme(
  theme: string,
  font: string,
  overrides: Record<string, Record<string, string>> = {},
): void {
  const tokens = resolveTokens(theme, overrides);
  const root = document.documentElement;
  for (const [k, v] of Object.entries(tokens)) root.style.setProperty(k, v);
  root.style.setProperty('--font', font);
  root.dataset.theme = theme;
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `theme` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/theme.ts web/src/lib/theme.test.ts
git commit -m "feat: add theme token maps and pure token resolution"
```

---

## Task 5: Types + API client

**Files:**
- Create: `web/src/lib/types.ts`, `web/src/lib/api.ts`, `web/src/lib/api.test.ts`

- [ ] **Step 1: Create the UI types**

Create `web/src/lib/types.ts`:
```ts
/** Mirrors the JSON returned by `GET /api/config` (serde-serialized UiConfig). */
export interface UiConfig {
  theme: string;
  font: string;
  edit_line_position: number;
  colors: Record<string, Record<string, string>>;
}
```

- [ ] **Step 2: Write the failing test**

Create `web/src/lib/api.test.ts`:
```ts
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import * as api from './api';

describe('api client', () => {
  const fetchMock = vi.fn();

  beforeEach(() => {
    vi.stubGlobal('fetch', fetchMock);
    fetchMock.mockReset();
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('getNote returns text from the right URL', async () => {
    fetchMock.mockResolvedValue({ ok: true, text: async () => '# hi\n' });
    expect(await api.getNote('2026-06-23')).toBe('# hi\n');
    expect(fetchMock).toHaveBeenCalledWith('/api/notes/2026-06-23');
  });

  it('listNotes parses a JSON array', async () => {
    fetchMock.mockResolvedValue({ ok: true, json: async () => ['2026-06-23'] });
    expect(await api.listNotes()).toEqual(['2026-06-23']);
  });

  it('putNote sends a PUT with the body', async () => {
    fetchMock.mockResolvedValue({ ok: true });
    await api.putNote('2026-06-23', '# hi\n');
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/notes/2026-06-23',
      expect.objectContaining({ method: 'PUT', body: '# hi\n' }),
    );
  });

  it('throws on a non-ok response', async () => {
    fetchMock.mockResolvedValue({ ok: false, status: 500 });
    await expect(api.getConfig()).rejects.toThrow();
  });
});
```

- [ ] **Step 3: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./api`.

- [ ] **Step 4: Write the implementation**

Create `web/src/lib/api.ts`:
```ts
import type { UiConfig } from './types';

export async function listNotes(): Promise<string[]> {
  const res = await fetch('/api/notes');
  if (!res.ok) throw new Error(`listNotes failed: ${res.status}`);
  return res.json();
}

export async function getNote(date: string): Promise<string> {
  const res = await fetch(`/api/notes/${date}`);
  if (!res.ok) throw new Error(`getNote failed: ${res.status}`);
  return res.text();
}

export async function putNote(date: string, content: string): Promise<void> {
  const res = await fetch(`/api/notes/${date}`, {
    method: 'PUT',
    headers: { 'content-type': 'text/markdown' },
    body: content,
  });
  if (!res.ok) throw new Error(`putNote failed: ${res.status}`);
}

export async function getConfig(): Promise<UiConfig> {
  const res = await fetch('/api/config');
  if (!res.ok) throw new Error(`getConfig failed: ${res.status}`);
  return res.json();
}
```

- [ ] **Step 5: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `api` tests green.

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/types.ts web/src/lib/api.ts web/src/lib/api.test.ts
git commit -m "feat: add UI types and API client"
```

---

## Task 6: Runes app store

**Files:**
- Create: `web/src/lib/appState.svelte.ts`

- [ ] **Step 1: Write the store**

Create `web/src/lib/appState.svelte.ts`:
```ts
import {
  initTabs,
  retarget,
  openNewTab,
  closeTab,
  nextTab,
  prevTab,
  activeDate,
  type TabsState,
} from './tabs';
import { todayISO, addDays, yearMonth } from './dates';
import { applyTheme } from './theme';
import { getConfig, listNotes, getNote } from './api';
import type { UiConfig } from './types';

class AppStore {
  tabsState = $state<TabsState>(initTabs(todayISO()));
  noteContent = $state<string>('');
  notesWithFiles = $state<string[]>([]);
  config = $state<UiConfig | null>(null);
  now = $state<Date>(new Date());
  calendar = $state<{ year: number; month: number }>(yearMonth(todayISO()));

  get activeDate(): string {
    return activeDate(this.tabsState);
  }

  async init(): Promise<void> {
    try {
      this.config = await getConfig();
      applyTheme(this.config.theme, this.config.font, this.config.colors);
    } catch (e) {
      console.error(e);
    }
    await this.refreshNotesList();
    await this.loadActive();
    setInterval(() => {
      this.now = new Date();
    }, 30_000);
  }

  async refreshNotesList(): Promise<void> {
    try {
      this.notesWithFiles = await listNotes();
    } catch (e) {
      console.error(e);
    }
  }

  async loadActive(): Promise<void> {
    const date = this.activeDate;
    try {
      this.noteContent = await getNote(date);
      this.calendar = yearMonth(date);
      await this.refreshNotesList(); // a freshly materialized date gets its dot
    } catch (e) {
      console.error(e);
    }
  }

  async goToDate(date: string): Promise<void> {
    this.tabsState = retarget(this.tabsState, date);
    await this.loadActive();
  }

  async openInNewTab(date: string): Promise<void> {
    this.tabsState = openNewTab(this.tabsState, date);
    await this.loadActive();
  }

  async goToday(): Promise<void> {
    await this.goToDate(todayISO());
  }
  async prevDay(): Promise<void> {
    await this.goToDate(addDays(this.activeDate, -1));
  }
  async nextDay(): Promise<void> {
    await this.goToDate(addDays(this.activeDate, 1));
  }

  async switchTab(index: number): Promise<void> {
    this.tabsState = { tabs: this.tabsState.tabs, activeIndex: index };
    await this.loadActive();
  }
  async cycleNext(): Promise<void> {
    this.tabsState = nextTab(this.tabsState);
    await this.loadActive();
  }
  async cyclePrev(): Promise<void> {
    this.tabsState = prevTab(this.tabsState);
    await this.loadActive();
  }
  async closeAt(index: number): Promise<void> {
    this.tabsState = closeTab(this.tabsState, index, todayISO());
    await this.loadActive();
  }

  prevMonth(): void {
    let { year, month } = this.calendar;
    month -= 1;
    if (month < 1) {
      month = 12;
      year -= 1;
    }
    this.calendar = { year, month };
  }
  nextMonth(): void {
    let { year, month } = this.calendar;
    month += 1;
    if (month > 12) {
      month = 1;
      year += 1;
    }
    this.calendar = { year, month };
  }
}

export const app = new AppStore();
```

- [ ] **Step 2: Type-check the store and project**

Run (from `web/`):
```bash
npm run check
```
Expected: PASS — no type errors. (The scaffold provides a `check` script running `svelte-check`. If the script name differs, run `npx svelte-check --tsconfig ./tsconfig.app.json`.)

- [ ] **Step 3: Commit**

```bash
git add web/src/lib/appState.svelte.ts
git commit -m "feat: add runes app store delegating to pure modules"
```

---

## Task 7: Components & layout

**Files:**
- Create: `web/src/lib/components/Header.svelte`, `Tabs.svelte`, `Calendar.svelte`, `Sidebar.svelte`, `EditorPane.svelte`
- Replace: `web/src/App.svelte`

- [ ] **Step 1: Create `web/src/lib/components/Tabs.svelte`**

```svelte
<script lang="ts">
  import { app } from '../appState.svelte';
</script>

<nav class="tabs">
  {#each app.tabsState.tabs as date, i (date)}
    <button
      class="tab"
      class:active={i === app.tabsState.activeIndex}
      onclick={() => app.switchTab(i)}
    >
      <span class="label">{date}</span>
      <span
        class="close"
        role="button"
        tabindex="0"
        aria-label="Close tab"
        onclick={(e) => {
          e.stopPropagation();
          app.closeAt(i);
        }}
        onkeydown={(e) => {
          if (e.key === 'Enter') {
            e.stopPropagation();
            app.closeAt(i);
          }
        }}>×</span
      >
    </button>
  {/each}
</nav>

<style>
  .tabs { display: flex; gap: 0.25rem; align-items: flex-end; overflow-x: auto; }
  .tab {
    display: inline-flex; align-items: center; gap: 0.4rem;
    border: none; cursor: pointer; padding: 0.3rem 0.6rem;
    border-radius: 6px 6px 0 0; background: transparent; color: var(--muted);
    font: inherit; font-size: 0.85rem; white-space: nowrap;
  }
  .tab.active { background: var(--bg); color: var(--fg); box-shadow: inset 0 -2px 0 var(--accent); }
  .close { opacity: 0.6; }
  .close:hover { opacity: 1; }
</style>
```

- [ ] **Step 2: Create `web/src/lib/components/Header.svelte`**

```svelte
<script lang="ts">
  import { app } from '../appState.svelte';
  import Tabs from './Tabs.svelte';

  const dateStr = $derived(
    `${app.now.getFullYear()}-${String(app.now.getMonth() + 1).padStart(2, '0')}-${String(
      app.now.getDate(),
    ).padStart(2, '0')}`,
  );
  const timeStr = $derived(
    `${String(app.now.getHours()).padStart(2, '0')}:${String(app.now.getMinutes()).padStart(2, '0')}`,
  );
</script>

<header class="header">
  <div class="brand">
    <div class="name">Slugline</div>
    <div class="clock">
      <span>{dateStr}</span>
      <span class="time">{timeStr}</span>
    </div>
  </div>
  <Tabs />
</header>

<style>
  .header {
    display: flex; align-items: flex-end; gap: 1.5rem;
    padding: 0.5rem 1rem 0; border-bottom: 1px solid var(--status-bar);
    background: var(--status-bar);
  }
  .name { font-weight: 700; font-size: 1.1rem; color: var(--heading-1); }
  .clock { display: flex; flex-direction: column; font-size: 0.8rem; color: var(--muted); line-height: 1.1; }
  .time { font-variant-numeric: tabular-nums; }
</style>
```

- [ ] **Step 3: Create `web/src/lib/components/Calendar.svelte`**

```svelte
<script lang="ts">
  import { app } from '../appState.svelte';
  import { monthGrid, todayISO } from '../dates';

  const weeks = $derived(monthGrid(app.calendar.year, app.calendar.month));
  const today = todayISO();
  const fileSet = $derived(new Set(app.notesWithFiles));
  const monthLabel = $derived(
    new Date(Date.UTC(app.calendar.year, app.calendar.month - 1, 1)).toLocaleDateString(undefined, {
      month: 'long',
      year: 'numeric',
      timeZone: 'UTC',
    }),
  );

  function onCellClick(e: MouseEvent, date: string) {
    if (e.metaKey || e.ctrlKey) app.openInNewTab(date);
    else app.goToDate(date);
  }
</script>

<div class="calendar">
  <div class="cal-head">
    <button onclick={() => app.prevMonth()} aria-label="Previous month">‹</button>
    <span class="month">{monthLabel}</span>
    <button onclick={() => app.nextMonth()} aria-label="Next month">›</button>
  </div>
  <div class="cal-grid">
    {#each ['S', 'M', 'T', 'W', 'T', 'F', 'S'] as d, i (i)}
      <div class="dow">{d}</div>
    {/each}
    {#each weeks as week, wi (wi)}
      {#each week as cell (cell.date)}
        <button
          class="cell"
          class:out={!cell.inMonth}
          class:today={cell.date === today}
          class:selected={cell.date === app.activeDate}
          onclick={(e) => onCellClick(e, cell.date)}
        >
          <span class="num">{Number(cell.date.slice(8, 10))}</span>
          {#if fileSet.has(cell.date)}<span class="dot"></span>{/if}
        </button>
      {/each}
    {/each}
  </div>
</div>

<style>
  .calendar { padding: 0.75rem; }
  .cal-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem; }
  .cal-head button { border: none; background: transparent; color: var(--fg); cursor: pointer; font-size: 1rem; }
  .month { font-size: 0.85rem; font-weight: 600; }
  .cal-grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; }
  .dow { text-align: center; font-size: 0.7rem; color: var(--muted); padding-bottom: 0.25rem; }
  .cell {
    position: relative; aspect-ratio: 1; border: none; cursor: pointer;
    background: transparent; color: var(--fg); border-radius: 6px; font: inherit; font-size: 0.8rem;
  }
  .cell:hover { background: var(--edit-line-bg); }
  .cell.out { color: var(--muted); opacity: 0.5; }
  .cell.today { outline: 1px solid var(--accent); }
  .cell.selected { background: var(--accent); color: #fff; }
  .dot {
    position: absolute; bottom: 4px; left: 50%; transform: translateX(-50%);
    width: 4px; height: 4px; border-radius: 50%; background: var(--accent);
  }
  .cell.selected .dot { background: #fff; }
</style>
```

- [ ] **Step 4: Create `web/src/lib/components/Sidebar.svelte`**

```svelte
<script lang="ts">
  import Calendar from './Calendar.svelte';
</script>

<aside class="sidebar">
  <Calendar />
  <section class="panel">
    <h2>Agenda</h2>
    <p class="empty">Coming in Phase 5</p>
  </section>
  <section class="panel">
    <h2>To Do</h2>
    <p class="empty">Coming in Phase 5</p>
  </section>
</aside>

<style>
  .sidebar {
    width: 280px; flex-shrink: 0; border-left: 1px solid var(--status-bar);
    overflow-y: auto; background: var(--bg);
  }
  .panel { padding: 0.75rem; border-top: 1px solid var(--status-bar); }
  .panel h2 { margin: 0 0 0.5rem; font-size: 0.9rem; color: var(--heading-2); }
  .empty { color: var(--muted); font-size: 0.8rem; margin: 0; }
</style>
```

- [ ] **Step 5: Create `web/src/lib/components/EditorPane.svelte` (read-only placeholder)**

```svelte
<script lang="ts">
  import { app } from '../appState.svelte';
</script>

<main class="editor">
  <pre>{app.noteContent}</pre>
</main>

<style>
  .editor { flex: 1; min-width: 0; overflow-y: auto; padding: 1rem 1.5rem; }
  pre {
    margin: 0; white-space: pre-wrap; word-break: break-word;
    font-family: var(--font), system-ui, sans-serif; font-size: 0.95rem; line-height: 1.5;
  }
</style>
```

- [ ] **Step 6: Replace `web/src/App.svelte` with the full layout + global shortcuts**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import Header from './lib/components/Header.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import EditorPane from './lib/components/EditorPane.svelte';
  import { app } from './lib/appState.svelte';

  onMount(() => {
    app.init();
  });

  function onKeydown(e: KeyboardEvent) {
    const target = e.target;
    const typing = target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;

    if ((e.ctrlKey || e.metaKey) && (e.key === 't' || e.key === 'T')) {
      e.preventDefault();
      app.goToday();
      return;
    }
    if (typing) return;
    if (e.key === '[') {
      e.preventDefault();
      app.prevDay();
    } else if (e.key === ']') {
      e.preventDefault();
      app.nextDay();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="app">
  <Header />
  <div class="body">
    <EditorPane />
    <Sidebar />
  </div>
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
  .body { display: flex; flex: 1; min-height: 0; }
</style>
```
Note: `Ctrl/Cmd-T` may be intercepted by the browser (new browser tab) and can't always be prevented; `[` / `]` and calendar clicks are the reliable navigation in this phase. Phase 4 will route these keys through the editor mode machine (and bind `gt`/`gT`, `:close`, `:goto`, etc.).

- [ ] **Step 7: Type-check and build**

Run (from `web/`):
```bash
npm run check && npm run build && npm test
```
Expected: `svelte-check` reports no errors; Vite build succeeds; all unit tests still PASS.

- [ ] **Step 8: Commit**

```bash
git add web/src/lib/components web/src/App.svelte
git commit -m "feat: add header, tabs, calendar, sidebar, and editor-pane shell"
```

---

## Task 8: Dev workflow, manual verification & production build

**Files:**
- Modify: `Makefile`

- [ ] **Step 1: Add a frontend dev target to the `Makefile`**

Add this target to the root `Makefile` (keep the existing targets from Phase 2):
```makefile
dev-web:
	cd web && npm run dev
```
(Recipe line must be **tab**-indented.) Also add `dev-web` to the `.PHONY` line.

- [ ] **Step 2: Run backend + frontend together (dev loop)**

In terminal 1 (repo root), start the API against a throwaway notes dir:
```bash
cargo run -- --notes-dir ./dev-notes --no-open
```
In terminal 2 (repo root), start the Vite dev server:
```bash
make dev-web
```
Then open the Vite URL it prints (typically `http://localhost:5173`).

- [ ] **Step 3: Manually verify the shell**

Confirm in the browser:
- Header shows **Slugline** with the **current wall-clock date and `HH:MM`** beneath it (wait up to 30s and confirm the minute updates).
- A single tab labelled with **today's date** is present.
- The **editor pane** shows today's raw note (the template: `# YYYY-MM-DD-DDD`, `## To Do`, `## Meetings`, `## Notes`).
- The **calendar** shows the current month with **today** marked and **today's date selected**; today's cell has a **dot** (it was just materialized).
- Click a **different date** → the editor pane loads that date's note (created on open), the calendar **selection moves**, and a **dot** appears on it.
- **Cmd/Ctrl-click** a date → it opens in a **new tab** (a second tab appears); clicking the tab's **×** closes it; closing the last tab leaves today open.
- Press `]` / `[` → the active tab moves to the next / previous day (editor + calendar follow).
- Use the calendar **‹ / ›** to change months; selecting a date in another month works.

- [ ] **Step 4: Verify the production single-binary build serves the real SPA**

From the repo root:
```bash
make build
./target/release/slugline --notes-dir ./dev-notes --no-open --port 4747
```
Open `http://127.0.0.1:4747` and confirm the **full app shell** (not the Phase 2 placeholder) renders and behaves as in Step 3. Then stop the server.

- [ ] **Step 5: Commit**

```bash
git add Makefile
git commit -m "chore: add frontend dev target; phase 3 app shell complete"
```

---

## Phase 3 Done Criteria

- `cd web && npm test` is green (dates, tabs, theme, api suites added; Phase 1 doc suites still pass).
- `npm run check` reports no type errors; `npm run build` succeeds.
- The dev loop (`cargo run` + `make dev-web`) renders the header wall-clock, tab bar, calendar with has-note dots, and the read-only editor pane.
- Navigation works end-to-end: calendar click (open/create + retarget), Cmd/Ctrl-click (new tab), tab switch/close (≥1 tab guaranteed), `[`/`]` day stepping, month nav — all keep the calendar selection and editor in sync.
- `make build` produces a single binary that serves the real SPA at `127.0.0.1:4747`.
- Theme tokens + font from `/api/config` are applied to the document root on load.

## Self-Review Notes (performed during authoring)

- **Spec coverage (roadmap Phase 3 row):** API client (`api.ts`), tab store with open/retarget/de-dup/`gt`/`gT`-ready cycle/close/≥1 (`tabs.ts` + store), header wall-clock (`Header.svelte`, 30s tick), layout (`App.svelte`), config load + theme tokens applied (`getConfig` + `applyTheme`), calendar grid with has-note dots + two-way date sync (`Calendar.svelte` + store) — all present. Calendar click-to-open is delivered here (roadmap refinement noted up top); `gt`/`gT` binding is deferred to Phase 4 (the editor owns NORMAL mode), but `cycleNext`/`cyclePrev` are implemented and ready.
- **Type consistency:** `TabsState` (Task 3) is consumed by the store (Task 6) unchanged; `UiConfig` (Task 5) is used by `api.getConfig` and the store; `MonthCell`/`monthGrid` (Task 2) feed `Calendar.svelte`. Store method names referenced by components (`switchTab`, `closeAt`, `goToDate`, `openInNewTab`, `prevMonth`, `nextMonth`, `prevDay`, `nextDay`, `goToday`) all exist on `AppStore`.
- **Pure vs. integration split:** every branching decision is in a unit-tested pure module; components/store are thin and verified by `svelte-check` + manual run (E2E deferred per roadmap).
- **No placeholders:** every step contains complete code and exact commands with expected output.

