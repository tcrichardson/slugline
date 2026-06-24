# Phase 5: Sidebar Features (Agenda + To Do) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Prerequisites:** Phases 1 (`scanDocument`, `classifyLine`), 2 (file API), 3 (store, `api`, `dates`, `Sidebar`), and 4 (editor state + `clampCursor`, `goToDate`) must be complete.

**Goal:** Fill in the two remaining sidebar panels — **Agenda** (scheduled meetings for the active note, click-to-jump) and **To Do** (a 7-day aggregation grouped by date, read-only + click-to-navigate).

**Architecture:** Both panels are driven by **pure, DOM-free derivation functions** unit-tested with Vitest. The Agenda derives live from the active editor buffer (`app.editor.lines`). The To Do aggregation reads the existing notes in a 7-day window from the API; the orchestration lives in the store and re-runs on navigation and after autosave. Clicking either panel jumps the editor cursor (and, for To Do, navigates to the right day first).

> **Scope note:** Calendar click-to-open/create was delivered in Phase 3. This phase adds only the Agenda and To Do panels. Toggling todo done-state directly from the sidebar remains a deliberately deferred later iteration (clicks navigate to the item instead).

**Tech Stack:** TypeScript, Svelte 5 (runes), Vitest.

---

## File Structure

| File | Responsibility |
|---|---|
| `web/src/lib/agenda.ts` | `deriveAgenda(lines)` → scheduled meetings, sorted |
| `web/src/lib/todos.ts` | `extractTodos(lines)`, `windowDates(date)`, `TodoGroup` |
| `web/src/lib/appState.svelte.ts` (modify) | `todoGroups` state, `refreshTodos`, `jumpToLine`, `goToDateAndLine` |
| `web/src/lib/components/Agenda.svelte` (create) | Render agenda rows, click-to-jump |
| `web/src/lib/components/TodoList.svelte` (create) | Render grouped todos, click-to-navigate |
| `web/src/lib/components/Sidebar.svelte` (replace) | Mount Calendar + Agenda + TodoList |

All commands below run from `web/`.

---

## Task 1: Agenda derivation

**Files:**
- Create: `web/src/lib/agenda.ts`, `web/src/lib/agenda.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/agenda.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { deriveAgenda } from './agenda';
import { fixtureLines } from './doc/__fixtures__/load';

describe('deriveAgenda', () => {
  it('lists scheduled meetings sorted by time', () => {
    const items = deriveAgenda(fixtureLines('full-day.md'));
    expect(items.map((i) => i.name)).toEqual(['Standup', 'Weekly Sync']);
    expect(items[0].time).toBe('09:00');
  });

  it('captures started/ended status when present', () => {
    const sync = deriveAgenda(fixtureLines('full-day.md')).find((i) => i.name === 'Weekly Sync')!;
    expect(sync.ended).toBe('15:02');
  });

  it('omits meetings without a scheduled time', () => {
    const lines = ['## Meetings', '### A', 'meta:scheduled 10:00', '### B', ''];
    expect(deriveAgenda(lines).map((i) => i.name)).toEqual(['A']);
  });

  it('returns empty when there is no Meetings section', () => {
    expect(deriveAgenda(['# T', '', '## Notes', ''])).toEqual([]);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./agenda`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/agenda.ts`:
```ts
import { scanDocument } from './doc/scan';

export interface AgendaItem {
  time: string;
  name: string;
  headingLineIndex: number;
  started?: string;
  ended?: string;
}

/** Scheduled meetings for a note, sorted ascending by HH:MM. Meetings without a scheduled time are omitted. */
export function deriveAgenda(lines: string[]): AgendaItem[] {
  const meetings = scanDocument(lines).sections.find((s) => s.kind === 'meetings');
  if (!meetings) return [];

  const items: AgendaItem[] = [];
  for (const block of meetings.blocks) {
    const scheduled = block.meta.find((m) => m.key === 'scheduled');
    if (!scheduled || scheduled.value.trim() === '') continue;
    items.push({
      time: scheduled.value.trim(),
      name: block.name,
      headingLineIndex: block.headingLineIndex,
      started: block.meta.find((m) => m.key === 'started')?.value,
      ended: block.meta.find((m) => m.key === 'ended')?.value,
    });
  }
  items.sort((a, b) => a.time.localeCompare(b.time));
  return items;
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `deriveAgenda` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/agenda.ts web/src/lib/agenda.test.ts
git commit -m "feat: add agenda derivation from scheduled meetings"
```

---

## Task 2: To Do extraction & window

**Files:**
- Create: `web/src/lib/todos.ts`, `web/src/lib/todos.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/todos.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { extractTodos, windowDates } from './todos';
import { fixtureLines } from './doc/__fixtures__/load';

describe('extractTodos', () => {
  it('extracts task items with done state and line indices', () => {
    const todos = extractTodos(fixtureLines('full-day.md'));
    expect(todos.map((t) => t.text)).toEqual([
      'Buy milk',
      'Send invoice',
      'Prep deck _(Weekly Sync)_',
    ]);
    expect(todos.map((t) => t.done)).toEqual([false, true, false]);
    expect(todos[0].lineIndex).toBe(4);
  });

  it('returns empty without a To Do section', () => {
    expect(extractTodos(['# T', '', '## Notes', ''])).toEqual([]);
  });
});

describe('windowDates', () => {
  it('returns 7 dates, most-recent first, ending on the active date', () => {
    const d = windowDates('2026-06-23');
    expect(d.length).toBe(7);
    expect(d[0]).toBe('2026-06-23');
    expect(d[6]).toBe('2026-06-17');
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./todos`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/todos.ts`:
```ts
import { scanDocument } from './doc/scan';
import { classifyLine } from './doc/classify';
import { addDays } from './dates';

export interface TodoItem {
  text: string;
  done: boolean;
  lineIndex: number;
}

export interface TodoGroup {
  date: string;
  todos: TodoItem[];
}

/** Task items in the `## To Do` section (both states), skipping blanks. */
export function extractTodos(lines: string[]): TodoItem[] {
  const section = scanDocument(lines).sections.find((s) => s.kind === 'todo');
  if (!section) return [];
  const out: TodoItem[] = [];
  for (let i = section.startLine + 1; i <= section.endLine; i++) {
    const c = classifyLine(lines[i] ?? '');
    if (c.kind === 'task' && c.text.trim() !== '') {
      out.push({ text: c.text, done: !!c.done, lineIndex: i });
    }
  }
  return out;
}

/** The `days` dates ending on `activeDate` (inclusive), most-recent first. */
export function windowDates(activeDate: string, days = 7): string[] {
  const out: string[] = [];
  for (let i = 0; i < days; i++) out.push(addDays(activeDate, -i));
  return out;
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `extractTodos`/`windowDates` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/todos.ts web/src/lib/todos.test.ts
git commit -m "feat: add todo extraction and 7-day window helper"
```

---

## Task 3: Store integration (todo aggregation + jump)

**Files:**
- Modify: `web/src/lib/appState.svelte.ts`

These are targeted additions to the Phase 4 store. Apply each edit at the indicated anchor.

- [ ] **Step 1: Add imports**

Add to the import block at the top of `appState.svelte.ts`:
```ts
import { clampCursor } from './editor/state';
import { extractTodos, windowDates, type TodoGroup } from './todos';
```

- [ ] **Step 2: Add the `todoGroups` reactive field**

Add alongside the other `$state` fields (e.g. just after the `calendar` field):
```ts
  todoGroups = $state<TodoGroup[]>([]);
```

- [ ] **Step 3: Add aggregation + jump methods**

Add these methods to the `AppStore` class (e.g. just before `prevMonth`):
```ts
  async refreshTodos(): Promise<void> {
    const active = this.activeDate;
    const existing = new Set(this.notesWithFiles);
    const groups: TodoGroup[] = [];
    for (const date of windowDates(active)) {
      if (date !== active && !existing.has(date)) continue; // never materialize other days
      try {
        const content = await getNote(date);
        const todos = extractTodos(content.split('\n'));
        if (todos.length > 0) groups.push({ date, todos });
      } catch (e) {
        console.error(e);
      }
    }
    this.todoGroups = groups;
  }

  jumpToLine(line: number): void {
    const clamped = Math.max(0, Math.min(line, this.editor.lines.length - 1));
    this.editor = clampCursor({ ...this.editor, cursor: { line: clamped, col: 0 } });
  }

  async goToDateAndLine(date: string, line: number): Promise<void> {
    if (date === this.activeDate) {
      this.jumpToLine(line);
      return;
    }
    await this.goToDate(date);
    this.jumpToLine(line);
  }
```

- [ ] **Step 4: Refresh todos on load and after save**

In `loadActive`, immediately after `await this.refreshNotesList();` add:
```ts
      await this.refreshTodos();
```

In `flush`, in the success branch immediately after `this.lastSaved = content;` add:
```ts
      await this.refreshTodos();
```

- [ ] **Step 5: Type-check**

Run (from `web/`):
```bash
npm run check
```
Expected: PASS — no type errors.

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/appState.svelte.ts
git commit -m "feat: aggregate 7-day todos and add cursor-jump navigation"
```

---

## Task 4: Agenda & To Do components

**Files:**
- Create: `web/src/lib/components/Agenda.svelte`, `web/src/lib/components/TodoList.svelte`
- Replace: `web/src/lib/components/Sidebar.svelte`

- [ ] **Step 1: Create `web/src/lib/components/Agenda.svelte`**

```svelte
<script lang="ts">
  import { app } from '../appState.svelte';
  import { deriveAgenda } from '../agenda';

  const items = $derived(deriveAgenda(app.editor.lines));
</script>

<section class="panel">
  <h2>Agenda</h2>
  {#if items.length === 0}
    <p class="empty">No scheduled meetings</p>
  {:else}
    <ul>
      {#each items as item (item.headingLineIndex)}
        <li class:done={!!item.ended}>
          <button onclick={() => app.jumpToLine(item.headingLineIndex)}>
            <span class="time">{item.time}</span>
            <span class="name">{item.name}</span>
            {#if item.ended}<span class="badge" title="Ended {item.ended}">✓</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .panel { padding: 0.75rem; border-top: 1px solid var(--status-bar); }
  .panel h2 { margin: 0 0 0.5rem; font-size: 0.9rem; color: var(--heading-2); }
  .empty { color: var(--muted); font-size: 0.8rem; margin: 0; }
  ul { list-style: none; margin: 0; padding: 0; }
  li button {
    display: flex; align-items: baseline; gap: 0.5rem; width: 100%;
    border: none; background: transparent; cursor: pointer; text-align: left;
    padding: 0.2rem 0.25rem; border-radius: 4px; color: var(--fg); font: inherit; font-size: 0.85rem;
  }
  li button:hover { background: var(--edit-line-bg); }
  .time { font-variant-numeric: tabular-nums; color: var(--accent); flex-shrink: 0; }
  li.done .name { color: var(--todo-done); text-decoration: line-through; }
  .badge { margin-left: auto; color: var(--todo-done); }
</style>
```

- [ ] **Step 2: Create `web/src/lib/components/TodoList.svelte`**

```svelte
<script lang="ts">
  import { app } from '../appState.svelte';
</script>

<section class="panel">
  <h2>To Do</h2>
  {#if app.todoGroups.length === 0}
    <p class="empty">No to dos in the last 7 days</p>
  {:else}
    {#each app.todoGroups as group (group.date)}
      <div class="group">
        <h3>{group.date}</h3>
        <ul>
          {#each group.todos as todo (todo.lineIndex)}
            <li class:done={todo.done}>
              <button onclick={() => app.goToDateAndLine(group.date, todo.lineIndex)}>
                <span class="box">{todo.done ? '☑' : '☐'}</span>
                <span class="text">{todo.text}</span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  {/if}
</section>

<style>
  .panel { padding: 0.75rem; border-top: 1px solid var(--status-bar); }
  .panel h2 { margin: 0 0 0.5rem; font-size: 0.9rem; color: var(--heading-2); }
  .empty { color: var(--muted); font-size: 0.8rem; margin: 0; }
  .group h3 { margin: 0.5rem 0 0.25rem; font-size: 0.75rem; color: var(--muted); font-variant-numeric: tabular-nums; }
  ul { list-style: none; margin: 0; padding: 0; }
  li button {
    display: flex; align-items: baseline; gap: 0.4rem; width: 100%;
    border: none; background: transparent; cursor: pointer; text-align: left;
    padding: 0.15rem 0.25rem; border-radius: 4px; color: var(--fg); font: inherit; font-size: 0.85rem;
  }
  li button:hover { background: var(--edit-line-bg); }
  .box { flex-shrink: 0; }
  li.done .text { color: var(--todo-done); text-decoration: line-through; }
</style>
```

- [ ] **Step 3: Replace `web/src/lib/components/Sidebar.svelte`**

```svelte
<script lang="ts">
  import Calendar from './Calendar.svelte';
  import Agenda from './Agenda.svelte';
  import TodoList from './TodoList.svelte';
</script>

<aside class="sidebar">
  <Calendar />
  <Agenda />
  <TodoList />
</aside>

<style>
  .sidebar {
    width: 280px; flex-shrink: 0; border-left: 1px solid var(--status-bar);
    overflow-y: auto; background: var(--bg);
  }
</style>
```

- [ ] **Step 4: Type-check and build**

Run (from `web/`):
```bash
npm run check && npm run build && npm test
```
Expected: `svelte-check` clean; Vite build succeeds; all unit tests pass.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/components/Agenda.svelte web/src/lib/components/TodoList.svelte web/src/lib/components/Sidebar.svelte
git commit -m "feat: render agenda and 7-day todo panels with click navigation"
```

---

## Task 5: End-to-end manual verification

**Files:** none (verification only)

- [ ] **Step 1: Seed a couple of days, then start the app**

Terminal 1 (repo root):
```bash
cargo run -- --notes-dir ./dev-notes --no-open
```
Terminal 2:
```bash
make dev-web
```
Open the Vite URL and focus the window.

- [ ] **Step 2: Verify the Agenda**

- On today's note, add meetings with scheduled times via the command line: `:meeting Standup`, then `:scheduled 09:00`; `:meeting Weekly Sync`, then `:scheduled 14:30`.
- The **Agenda** panel lists them **sorted by time** (Standup before Weekly Sync), each showing `HH:MM` + name.
- Run `:start` then `:end` inside Weekly Sync → the row reflects ended status (✓ / strikethrough).
- A meeting with no `:scheduled` does **not** appear. With no scheduled meetings the panel shows **"No scheduled meetings"**.
- **Click an agenda row** → the editor cursor jumps to that meeting's heading and scrolls it to the edit-line anchor.

- [ ] **Step 3: Verify the To Do panel & 7-day window**

- Add todos to today via `:todo Buy milk`, `:todo Call bank`; toggle one done with `t`. After autosave (~1s) the **To Do** panel shows today's group with both items (the done one dimmed/strikethrough).
- Navigate to a previous day with `[`, add a todo there; go back with `]`. The To Do panel now shows **two date groups, most-recent first**, each headed by its date.
- Days with no note or no todos are **absent** from the list. With nothing in the window it shows **"No to dos in the last 7 days"**.
- **Click a todo in another day's group** → the app navigates to that day and the cursor lands on that todo line. **Clicking a todo in today's group** jumps without reloading (cursor moves, undo history preserved).

- [ ] **Step 4: Verify the production build**

```bash
make build
./target/release/slugline --notes-dir ./dev-notes --no-open --port 4747
```
Open `http://127.0.0.1:4747` and re-check the Agenda and To Do panels against the embedded SPA. Stop the server when done.

---

## Phase 5 Done Criteria

- `cd web && npm test` is green including the new `agenda` and `todos` suites (plus all prior phases).
- `npm run check` is clean; `npm run build` succeeds.
- The Agenda lists the active note's scheduled meetings sorted by time, reflects started/ended status, click-jumps to the heading, and shows the empty state when appropriate.
- The To Do panel aggregates the 7-day window ending on the active date, grouped by date most-recent-first, shows both done/undone items (done dimmed), skips empty days, click-navigates to the item, and updates after navigation and autosave.

## Self-Review Notes (performed during authoring)

- **Spec coverage (roadmap Phase 5 row):** Agenda — active-note only (derives from `app.editor.lines`), scheduled-only + sorted (`deriveAgenda`), started/ended status, click-to-jump (`jumpToLine`); To Do — 7-day window ending on the active date (`windowDates`), grouped by date most-recent-first, all items with done dimmed, skip empty days, read-only + click-to-navigate (`goToDateAndLine`). Calendar wiring was completed in Phase 3 (noted up top); sidebar toggling remains deferred.
- **Type consistency:** `AgendaItem` (agenda.ts) consumed by `Agenda.svelte`; `TodoItem`/`TodoGroup` (todos.ts) consumed by the store and `TodoList.svelte`. Store additions reuse existing `getNote`, `goToDate`, `editor`, `notesWithFiles`, and `clampCursor` — names verified against Phases 3/4.
- **Freshness model:** the active day's todos reflect saved state; `refreshTodos` runs on `loadActive` and after `flush`, so a `:todo` shows in the sidebar within the autosave debounce (~750ms). Other days are read from disk and never materialized (existence gated by `notesWithFiles`).
- **No placeholders:** every code step contains complete code; UI behavior not unit-testable is covered by the manual checklist in Task 5.

