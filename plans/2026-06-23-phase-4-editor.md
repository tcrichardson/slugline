# Phase 4: The Editor — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Prerequisites:** Phases 1 (document model: `classifyLine`, `scanDocument`, `resolveContext`, `nearestHeadingLevel`, `validateCommand`), 2 (file API), and 3 (app shell: store, tabs, calendar, API client) must be complete.

**Goal:** Build the modal vim-style editor over the raw line array: NORMAL/INSERT modes, motions, edits, undo/redo, a shared line-wise register, the `:` command line + interpreter (buffer mutations + app effects), per-line pretty rendering with a single raw "edit line", a centered scroll anchor, and debounced autosave.

**Architecture:** The entire editor is a set of **pure, DOM-free reducers** — `(EditorState, input) -> { state, effect? }` — unit-tested with Vitest. Commands either mutate the buffer (using the Phase 1 parser/scanner) or emit an `AppEffect` (navigation/save/theme) that the store performs. A thin Svelte layer renders each line (active line raw + custom cursor; others pretty via Phase 1) and forwards keystrokes. The store owns side effects: autosave (debounce + flush), navigation, and the register shared across tabs.

**Tech Stack:** TypeScript, Svelte 5 (runes), Vitest. New code lives in `web/src/lib/editor/`.

---

## File Structure

| File | Responsibility |
|---|---|
| `web/src/lib/editor/state.ts` | `EditorState`, `createEditorState`, snapshot/undo/redo, `clampCursor` |
| `web/src/lib/editor/motions.ts` | Cursor motions: `h j k l w b e 0 $ gg G` |
| `web/src/lib/editor/edits.ts` | NORMAL edits: `x dd yy p P t` |
| `web/src/lib/editor/insert.ts` | INSERT ops + mode entry (`i a A o O`, Esc, text, newline, backspace, Ctrl-W, Tab) |
| `web/src/lib/editor/commands.ts` | `runCommand` + buffer helpers + `AppEffect` |
| `web/src/lib/editor/keymap.ts` | `handleKey` dispatcher across NORMAL/INSERT/command modes |
| `web/src/lib/appState.svelte.ts` (modify) | Hold `EditorState`, shared register, effects, autosave |
| `web/src/lib/components/EditorPane.svelte` (replace) | Per-line render, raw active line + cursor, scroll anchor |
| `web/src/lib/components/StatusLine.svelte` (create) | Mode / command line / context / message |
| `web/src/App.svelte` (modify) | Forward keydown to the store; mount StatusLine |

All commands below run from `web/`.

---

## Task 1: Editor state, undo/redo, cursor clamping

**Files:**
- Create: `web/src/lib/editor/state.ts`, `web/src/lib/editor/state.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/editor/state.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { createEditorState, pushUndo, undo, redo, clampCursor } from './state';

describe('editor state', () => {
  it('never has zero lines', () => {
    expect(createEditorState([]).lines).toEqual(['']);
  });

  it('undo/redo round-trips a line change', () => {
    let s = createEditorState(['a', 'b']);
    s = pushUndo(s);
    s = { ...s, lines: ['a', 'b', 'c'] };
    s = undo(s);
    expect(s.lines).toEqual(['a', 'b']);
    s = redo(s);
    expect(s.lines).toEqual(['a', 'b', 'c']);
  });

  it('clamps cursor col to length-1 in normal mode', () => {
    const s = clampCursor({ ...createEditorState(['abc']), cursor: { line: 0, col: 99 } });
    expect(s.cursor.col).toBe(2);
  });

  it('allows cursor col at length in insert mode', () => {
    const base = createEditorState(['abc']);
    const s = clampCursor({ ...base, mode: 'insert', cursor: { line: 0, col: 99 } });
    expect(s.cursor.col).toBe(3);
  });

  it('preserves a provided register (shared across tabs)', () => {
    expect(createEditorState(['x'], ['line']).register).toEqual(['line']);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./state`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/editor/state.ts`:
```ts
export type Mode = 'normal' | 'insert';

export interface Cursor {
  line: number;
  col: number;
}

export interface Snapshot {
  lines: string[];
  cursor: Cursor;
}

export type Pending = '' | 'g' | 'd' | 'y';

export interface EditorState {
  lines: string[];
  cursor: Cursor;
  mode: Mode;
  register: string[]; // line-wise; shared across tabs by the store
  pending: Pending;
  command: string | null; // command-line buffer (text after ':'), null when inactive
  message: string;
  undo: Snapshot[];
  redo: Snapshot[];
}

export function createEditorState(lines: string[], register: string[] = []): EditorState {
  return {
    lines: lines.length > 0 ? lines.slice() : [''],
    cursor: { line: 0, col: 0 },
    mode: 'normal',
    register,
    pending: '',
    command: null,
    message: '',
    undo: [],
    redo: [],
  };
}

export function snapshot(s: EditorState): Snapshot {
  return { lines: s.lines.slice(), cursor: { ...s.cursor } };
}

/** Snapshot the pre-mutation state and clear redo. Call BEFORE applying a mutation. */
export function pushUndo(s: EditorState): EditorState {
  return { ...s, undo: [...s.undo, snapshot(s)], redo: [] };
}

export function undo(s: EditorState): EditorState {
  if (s.undo.length === 0) return { ...s, message: 'Already at oldest change' };
  const prev = s.undo[s.undo.length - 1];
  return {
    ...s,
    lines: prev.lines.slice(),
    cursor: { ...prev.cursor },
    undo: s.undo.slice(0, -1),
    redo: [...s.redo, snapshot(s)],
    message: '',
  };
}

export function redo(s: EditorState): EditorState {
  if (s.redo.length === 0) return { ...s, message: 'Already at newest change' };
  const next = s.redo[s.redo.length - 1];
  return {
    ...s,
    lines: next.lines.slice(),
    cursor: { ...next.cursor },
    redo: s.redo.slice(0, -1),
    undo: [...s.undo, snapshot(s)],
    message: '',
  };
}

/** Clamp cursor to valid bounds for the current mode. */
export function clampCursor(s: EditorState): EditorState {
  const line = Math.max(0, Math.min(s.cursor.line, s.lines.length - 1));
  const text = s.lines[line] ?? '';
  const maxCol = s.mode === 'insert' ? text.length : Math.max(0, text.length - 1);
  const col = Math.max(0, Math.min(s.cursor.col, maxCol));
  return { ...s, cursor: { line, col } };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `state` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/editor/state.ts web/src/lib/editor/state.test.ts
git commit -m "feat: add editor state with undo/redo and cursor clamping"
```

---

## Task 2: Cursor motions

**Files:**
- Create: `web/src/lib/editor/motions.ts`, `web/src/lib/editor/motions.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/editor/motions.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { createEditorState } from './state';
import { moveRight, moveDown, lineEnd, lastLine, wordForward, wordBackward, wordEnd } from './motions';

const at = (lines: string[], line: number, col: number) => ({
  ...createEditorState(lines),
  cursor: { line, col },
});

describe('motions', () => {
  it('moveRight stops at the last char in normal mode', () => {
    expect(moveRight(at(['ab'], 0, 1)).cursor.col).toBe(1);
  });

  it('moveDown stays within bounds', () => {
    expect(moveDown(at(['a', 'b'], 1, 0)).cursor.line).toBe(1);
  });

  it('lineEnd goes to the last char (normal)', () => {
    expect(lineEnd(at(['hello'], 0, 0)).cursor.col).toBe(4);
  });

  it('lastLine jumps to the final line', () => {
    expect(lastLine(at(['a', 'b', 'c'], 0, 0)).cursor.line).toBe(2);
  });

  it('wordForward jumps to the next word start', () => {
    expect(wordForward(at(['foo bar'], 0, 0)).cursor.col).toBe(4);
  });

  it('wordBackward jumps to the previous word start', () => {
    expect(wordBackward(at(['foo bar'], 0, 4)).cursor.col).toBe(0);
  });

  it('wordEnd jumps to the end of the next word', () => {
    expect(wordEnd(at(['foo bar'], 0, 0)).cursor.col).toBe(2);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./motions`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/editor/motions.ts`:
```ts
import type { EditorState } from './state';
import { clampCursor } from './state';

export function moveLeft(s: EditorState): EditorState {
  return clampCursor({ ...s, cursor: { ...s.cursor, col: s.cursor.col - 1 } });
}
export function moveRight(s: EditorState): EditorState {
  return clampCursor({ ...s, cursor: { ...s.cursor, col: s.cursor.col + 1 } });
}
export function moveUp(s: EditorState): EditorState {
  return clampCursor({ ...s, cursor: { line: Math.max(0, s.cursor.line - 1), col: s.cursor.col } });
}
export function moveDown(s: EditorState): EditorState {
  const line = Math.min(s.lines.length - 1, s.cursor.line + 1);
  return clampCursor({ ...s, cursor: { line, col: s.cursor.col } });
}
export function lineStart(s: EditorState): EditorState {
  return { ...s, cursor: { ...s.cursor, col: 0 } };
}
export function lineEnd(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const maxCol = s.mode === 'insert' ? text.length : Math.max(0, text.length - 1);
  return { ...s, cursor: { ...s.cursor, col: maxCol } };
}
export function firstLine(s: EditorState): EditorState {
  return clampCursor({ ...s, cursor: { line: 0, col: s.cursor.col } });
}
export function lastLine(s: EditorState): EditorState {
  return clampCursor({ ...s, cursor: { line: s.lines.length - 1, col: s.cursor.col } });
}

// vim "word": a run of word chars (\w) OR a run of punctuation, separated by whitespace.
function classOf(ch: string | undefined): number {
  if (ch === undefined || /\s/.test(ch)) return 0;
  if (/\w/.test(ch)) return 1;
  return 2;
}

export function wordForward(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const n = text.length;
  let i = s.cursor.col;
  if (i >= n) return s;
  const startClass = classOf(text[i]);
  if (startClass !== 0) while (i < n && classOf(text[i]) === startClass) i++;
  while (i < n && classOf(text[i]) === 0) i++;
  return { ...s, cursor: { ...s.cursor, col: Math.min(i, Math.max(0, n - 1)) } };
}

export function wordBackward(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  let i = s.cursor.col;
  if (i <= 0) return s;
  i--;
  while (i > 0 && classOf(text[i]) === 0) i--;
  const cl = classOf(text[i]);
  while (i > 0 && classOf(text[i - 1]) === cl && cl !== 0) i--;
  return { ...s, cursor: { ...s.cursor, col: Math.max(0, i) } };
}

export function wordEnd(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const n = text.length;
  let i = s.cursor.col;
  if (i >= n - 1) return s;
  i++;
  while (i < n && classOf(text[i]) === 0) i++;
  const cl = classOf(text[i]);
  while (i < n - 1 && classOf(text[i + 1]) === cl && cl !== 0) i++;
  return { ...s, cursor: { ...s.cursor, col: Math.min(i, n - 1) } };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `motions` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/editor/motions.ts web/src/lib/editor/motions.test.ts
git commit -m "feat: add cursor motions (hjkl, wbe, 0$, gg/G)"
```

---

## Task 3: NORMAL-mode edits

**Files:**
- Create: `web/src/lib/editor/edits.ts`, `web/src/lib/editor/edits.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/editor/edits.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { createEditorState } from './state';
import { deleteChar, deleteLine, yankLine, pasteBelow, pasteAbove, toggleTodo } from './edits';

const at = (lines: string[], line = 0, col = 0) => ({
  ...createEditorState(lines),
  cursor: { line, col },
});

describe('normal edits', () => {
  it('x deletes the char under the cursor', () => {
    expect(deleteChar(at(['abc'], 0, 1)).lines[0]).toBe('ac');
  });

  it('dd deletes and yanks the line', () => {
    const s = deleteLine(at(['a', 'b'], 0, 0));
    expect(s.lines).toEqual(['b']);
    expect(s.register).toEqual(['a']);
    expect(s.undo.length).toBe(1);
  });

  it('yy yanks; p pastes below', () => {
    let s = yankLine(at(['a', 'b'], 0, 0));
    s = { ...s, cursor: { line: 1, col: 0 } };
    s = pasteBelow(s);
    expect(s.lines).toEqual(['a', 'b', 'a']);
  });

  it('P pastes above', () => {
    let s = yankLine(at(['a', 'b'], 1, 0));
    s = pasteAbove({ ...s, cursor: { line: 0, col: 0 } });
    expect(s.lines).toEqual(['b', 'a', 'b']);
  });

  it('t toggles a task item only', () => {
    expect(toggleTodo(at(['- [ ] x'], 0, 0)).lines[0]).toBe('- [x] x');
    expect(toggleTodo(at(['- [x] x'], 0, 0)).lines[0]).toBe('- [ ] x');
    expect(toggleTodo(at(['plain'], 0, 0)).lines[0]).toBe('plain');
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./edits`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/editor/edits.ts`:
```ts
import type { EditorState } from './state';
import { clampCursor, pushUndo } from './state';

export function deleteChar(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  if (text.length === 0) return s;
  const ns = pushUndo(s);
  const lines = s.lines.slice();
  lines[s.cursor.line] = text.slice(0, s.cursor.col) + text.slice(s.cursor.col + 1);
  return clampCursor({ ...ns, lines });
}

export function deleteLine(s: EditorState): EditorState {
  const ns = pushUndo(s);
  const lines = s.lines.slice();
  const removed = lines.splice(s.cursor.line, 1);
  if (lines.length === 0) lines.push('');
  const line = Math.min(s.cursor.line, lines.length - 1);
  return clampCursor({ ...ns, lines, register: removed, cursor: { line, col: 0 } });
}

export function yankLine(s: EditorState): EditorState {
  return { ...s, register: [s.lines[s.cursor.line] ?? ''], message: '1 line yanked' };
}

export function pasteBelow(s: EditorState): EditorState {
  if (s.register.length === 0) return s;
  const ns = pushUndo(s);
  const lines = s.lines.slice();
  lines.splice(s.cursor.line + 1, 0, ...s.register);
  return clampCursor({ ...ns, lines, cursor: { line: s.cursor.line + 1, col: 0 } });
}

export function pasteAbove(s: EditorState): EditorState {
  if (s.register.length === 0) return s;
  const ns = pushUndo(s);
  const lines = s.lines.slice();
  lines.splice(s.cursor.line, 0, ...s.register);
  return clampCursor({ ...ns, lines, cursor: { line: s.cursor.line, col: 0 } });
}

const TASK_RE = /^(\s*- \[)([ xX])(\] )/;

export function toggleTodo(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const m = TASK_RE.exec(text);
  if (!m) return s; // no-op on non-task lines
  const ns = pushUndo(s);
  const next = m[2] === ' ' ? 'x' : ' ';
  const lines = s.lines.slice();
  lines[s.cursor.line] = text.replace(TASK_RE, `$1${next}$3`);
  return { ...ns, lines };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `edits` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/editor/edits.ts web/src/lib/editor/edits.test.ts
git commit -m "feat: add normal-mode edits (x, dd, yy, p, P, t)"
```

---

## Task 4: INSERT-mode operations & mode entry

**Files:**
- Create: `web/src/lib/editor/insert.ts`, `web/src/lib/editor/insert.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/editor/insert.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { createEditorState } from './state';
import {
  enterInsert,
  enterInsertAfter,
  openBelow,
  exitInsert,
  insertText,
  insertNewline,
  backspace,
  deleteWordBefore,
} from './insert';

const at = (lines: string[], line = 0, col = 0) => ({
  ...createEditorState(lines),
  cursor: { line, col },
});
const ins = (lines: string[], line = 0, col = 0) => ({ ...at(lines, line, col), mode: 'insert' as const });

describe('insert ops', () => {
  it('enterInsert switches mode and pushes one undo snapshot', () => {
    const s = enterInsert(at(['abc'], 0, 1));
    expect(s.mode).toBe('insert');
    expect(s.undo.length).toBe(1);
  });

  it('enterInsertAfter moves one past the cursor', () => {
    const s = enterInsertAfter(at(['abc'], 0, 1));
    expect(s.cursor.col).toBe(2);
  });

  it('insertText inserts at the cursor and advances', () => {
    const s = insertText(ins(['ac'], 0, 1), 'b');
    expect(s.lines[0]).toBe('abc');
    expect(s.cursor.col).toBe(2);
  });

  it('insertNewline splits the line', () => {
    const s = insertNewline(ins(['ab'], 0, 1));
    expect(s.lines).toEqual(['a', 'b']);
    expect(s.cursor).toEqual({ line: 1, col: 0 });
  });

  it('backspace joins lines at col 0', () => {
    const s = backspace(ins(['a', 'b'], 1, 0));
    expect(s.lines).toEqual(['ab']);
    expect(s.cursor).toEqual({ line: 0, col: 1 });
  });

  it('openBelow inserts a blank line and enters insert', () => {
    const s = openBelow(at(['a'], 0, 0));
    expect(s.lines).toEqual(['a', '']);
    expect(s.mode).toBe('insert');
    expect(s.cursor).toEqual({ line: 1, col: 0 });
  });

  it('deleteWordBefore removes the previous word', () => {
    const s = deleteWordBefore(ins(['foo bar'], 0, 7));
    expect(s.lines[0]).toBe('foo ');
  });

  it('exitInsert clamps the cursor back into normal bounds', () => {
    const s = exitInsert(ins(['abc'], 0, 3));
    expect(s.mode).toBe('normal');
    expect(s.cursor.col).toBe(2);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./insert`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/editor/insert.ts`:
```ts
import type { EditorState } from './state';
import { clampCursor, pushUndo } from './state';

// Mode entry: each pushes ONE undo snapshot for the whole insert session.
export function enterInsert(s: EditorState): EditorState {
  return { ...pushUndo(s), mode: 'insert' };
}
export function enterInsertAfter(s: EditorState): EditorState {
  const ns = pushUndo(s);
  const text = ns.lines[ns.cursor.line] ?? '';
  const col = Math.min(ns.cursor.col + (text.length > 0 ? 1 : 0), text.length);
  return { ...ns, mode: 'insert', cursor: { ...ns.cursor, col } };
}
export function enterInsertLineEnd(s: EditorState): EditorState {
  const ns = pushUndo(s);
  const text = ns.lines[ns.cursor.line] ?? '';
  return { ...ns, mode: 'insert', cursor: { ...ns.cursor, col: text.length } };
}
export function openBelow(s: EditorState): EditorState {
  const ns = pushUndo(s);
  const lines = ns.lines.slice();
  lines.splice(ns.cursor.line + 1, 0, '');
  return { ...ns, lines, mode: 'insert', cursor: { line: ns.cursor.line + 1, col: 0 } };
}
export function openAbove(s: EditorState): EditorState {
  const ns = pushUndo(s);
  const lines = ns.lines.slice();
  lines.splice(ns.cursor.line, 0, '');
  return { ...ns, lines, mode: 'insert', cursor: { line: ns.cursor.line, col: 0 } };
}

export function exitInsert(s: EditorState): EditorState {
  return clampCursor({ ...s, mode: 'normal' });
}

// In-session edits: NO undo push (the session snapshot was taken on entry).
export function insertText(s: EditorState, ch: string): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const lines = s.lines.slice();
  lines[s.cursor.line] = text.slice(0, s.cursor.col) + ch + text.slice(s.cursor.col);
  return { ...s, lines, cursor: { ...s.cursor, col: s.cursor.col + ch.length } };
}
export function insertTab(s: EditorState): EditorState {
  return insertText(s, '  ');
}
export function insertNewline(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  const lines = s.lines.slice();
  lines.splice(s.cursor.line, 1, text.slice(0, s.cursor.col), text.slice(s.cursor.col));
  return { ...s, lines, cursor: { line: s.cursor.line + 1, col: 0 } };
}
export function backspace(s: EditorState): EditorState {
  const { line, col } = s.cursor;
  if (col > 0) {
    const text = s.lines[line];
    const lines = s.lines.slice();
    lines[line] = text.slice(0, col - 1) + text.slice(col);
    return { ...s, lines, cursor: { line, col: col - 1 } };
  }
  if (line > 0) {
    const prev = s.lines[line - 1];
    const lines = s.lines.slice();
    lines.splice(line - 1, 2, prev + s.lines[line]);
    return { ...s, lines, cursor: { line: line - 1, col: prev.length } };
  }
  return s;
}
export function deleteWordBefore(s: EditorState): EditorState {
  const text = s.lines[s.cursor.line] ?? '';
  let i = s.cursor.col;
  while (i > 0 && /\s/.test(text[i - 1])) i--;
  while (i > 0 && !/\s/.test(text[i - 1])) i--;
  const lines = s.lines.slice();
  lines[s.cursor.line] = text.slice(0, i) + text.slice(s.cursor.col);
  return { ...s, lines, cursor: { ...s.cursor, col: i } };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `insert` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/editor/insert.ts web/src/lib/editor/insert.test.ts
git commit -m "feat: add insert-mode operations and mode entry"
```

---

## Task 5: Command buffer helpers

**Files:**
- Create: `web/src/lib/editor/commands.ts`, `web/src/lib/editor/commands.test.ts`

This task creates `commands.ts` with the **pure line-manipulation helpers** and their tests. Task 6 adds `runCommand` to the same file.

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/editor/commands.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { scanDocument } from '../doc/scan';
import { ensureSection, appendBlock, appendLineToSection, upsertMeta, endOfEnclosingSection } from './commands';

const TEMPLATE = ['# 2026-06-23-TUE', '', '## To Do', '', '## Meetings', '', '## Notes', ''];

describe('command helpers', () => {
  it('appendBlock adds an H3 at the end of a section', () => {
    const meetings = scanDocument(TEMPLATE).sections.find((s) => s.kind === 'meetings')!;
    const { lines, headingIndex } = appendBlock(TEMPLATE, meetings, '### Sync');
    expect(lines[headingIndex]).toBe('### Sync');
  });

  it('appendLineToSection adds after the last non-blank line', () => {
    const todo = scanDocument(TEMPLATE).sections.find((s) => s.kind === 'todo')!;
    const { lines, index } = appendLineToSection(TEMPLATE, todo, '- [ ] Buy milk');
    expect(lines[index]).toBe('- [ ] Buy milk');
    // it stays within the To Do section (before ## Meetings)
    expect(lines.indexOf('## Meetings')).toBeGreaterThan(index);
  });

  it('upsertMeta inserts then updates in place', () => {
    let lines = ['## Meetings', '### Sync', ''];
    let block = scanDocument(lines).sections[0].blocks[0];
    ({ lines } = upsertMeta(lines, block, 'scheduled', '14:30'));
    expect(lines).toContain('meta:scheduled 14:30');
    block = scanDocument(lines).sections[0].blocks[0];
    ({ lines } = upsertMeta(lines, block, 'scheduled', '15:00'));
    expect(lines.filter((l) => l.startsWith('meta:scheduled')).length).toBe(1);
    expect(lines).toContain('meta:scheduled 15:00');
  });

  it('ensureSection recreates a missing section in canonical order', () => {
    const { lines, section } = ensureSection(['# T', '', '## Notes', ''], 'meetings');
    const kinds = scanDocument(lines).sections.map((s) => s.kind);
    expect(kinds).toContain('meetings');
    expect(kinds.indexOf('meetings')).toBeLessThan(kinds.indexOf('notes'));
    expect(section.kind).toBe('meetings');
  });

  it('endOfEnclosingSection finds the next same/shallower heading', () => {
    expect(endOfEnclosingSection(['### A', 'body', '### B'], 1, 3)).toBe(2);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./commands`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/editor/commands.ts`:
```ts
import { scanDocument } from '../doc/scan';
import type { Section, Block } from '../doc/types';

const TITLES: Record<'todo' | 'meetings' | 'notes', string> = {
  todo: '## To Do',
  meetings: '## Meetings',
  notes: '## Notes',
};
const ORDER: ('todo' | 'meetings' | 'notes')[] = ['todo', 'meetings', 'notes'];

/** Append an H3 (or any heading text) at the end of a section's content. */
export function appendBlock(
  lines: string[],
  section: Section,
  heading: string,
): { lines: string[]; headingIndex: number } {
  const idx = section.endLine + 1;
  const out = lines.slice();
  out.splice(idx, 0, heading, '');
  return { lines: out, headingIndex: idx };
}

/** Append a single line after the last non-blank line of a section (or right after its heading). */
export function appendLineToSection(
  lines: string[],
  section: Section,
  text: string,
): { lines: string[]; index: number } {
  let insertAt = section.startLine + 1;
  for (let i = section.startLine + 1; i <= section.endLine; i++) {
    if ((lines[i] ?? '').trim() !== '') insertAt = i + 1;
  }
  const out = lines.slice();
  out.splice(insertAt, 0, text);
  return { lines: out, index: insertAt };
}

/** Insert or update a `meta:key value` line within a block's meta region. */
export function upsertMeta(
  lines: string[],
  block: Block,
  key: string,
  value: string,
): { lines: string[]; index: number } {
  const metaLine = `meta:${key} ${value}`;
  const existing = block.meta.find((m) => m.key === key);
  const out = lines.slice();
  if (existing) {
    out[existing.lineIndex] = metaLine;
    return { lines: out, index: existing.lineIndex };
  }
  const insertAt = block.metaEndLine + 1; // metaEndLine === heading when no meta yet
  out.splice(insertAt, 0, metaLine);
  return { lines: out, index: insertAt };
}

/** Ensure a standard section exists; if missing, insert it in canonical order. */
export function ensureSection(
  lines: string[],
  kind: 'todo' | 'meetings' | 'notes',
): { lines: string[]; section: Section } {
  let model = scanDocument(lines);
  const found = model.sections.find((s) => s.kind === kind);
  if (found) return { lines, section: found };

  const orderIdx = ORDER.indexOf(kind);
  let insertAt = lines.length;
  let placed = false;
  for (let i = orderIdx - 1; i >= 0 && !placed; i--) {
    const prev = model.sections.find((s) => s.kind === ORDER[i]);
    if (prev) {
      insertAt = prev.endLine + 1;
      placed = true;
    }
  }
  if (!placed) insertAt = model.titleLineIndex !== null ? model.titleLineIndex + 1 : 0;

  const out = lines.slice();
  out.splice(insertAt, 0, '', TITLES[kind], '');
  model = scanDocument(out);
  return { lines: out, section: model.sections.find((s) => s.kind === kind)! };
}

/** Index just past the enclosing heading's content (before the next same/shallower heading, or EOF). */
export function endOfEnclosingSection(lines: string[], cursorLine: number, level: number): number {
  const H = /^(#{1,6})\s/;
  let start = cursorLine;
  while (start >= 0) {
    const m = H.exec(lines[start] ?? '');
    if (m && m[1].length === level) break;
    start--;
  }
  if (start < 0) start = cursorLine;
  for (let i = start + 1; i < lines.length; i++) {
    const m = H.exec(lines[i] ?? '');
    if (m && m[1].length <= level) return i;
  }
  return lines.length;
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `command helpers` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/editor/commands.ts web/src/lib/editor/commands.test.ts
git commit -m "feat: add command buffer-manipulation helpers"
```

---

## Task 6: `runCommand` (buffer mutations + app effects)

**Files:**
- Modify: `web/src/lib/editor/commands.ts`, `web/src/lib/editor/commands.test.ts`

- [ ] **Step 1: Add the failing tests**

Append this `describe` block to `web/src/lib/editor/commands.test.ts`:
```ts
import { createEditorState } from './state';
import { runCommand } from './commands';

const TPL = ['# 2026-06-23-TUE', '', '## To Do', '', '## Meetings', '', '## Notes', ''];
const withCmd = (lines: string[], cmd: string, line = 0) => ({
  ...createEditorState(lines),
  command: cmd,
  cursor: { line, col: 0 },
});
const ctx = { nowHHMM: '09:30' };

describe('runCommand', () => {
  it('reports unknown commands and clears the command line', () => {
    const r = runCommand(withCmd(TPL, 'meetng x'), ctx);
    expect(r.state.command).toBeNull();
    expect(r.state.message).toContain('Unknown command');
  });

  it(':meeting adds a heading and moves the cursor to it', () => {
    const r = runCommand(withCmd(TPL, 'meeting Daily Standup'), ctx);
    expect(r.state.lines[r.state.cursor.line]).toBe('### Daily Standup');
  });

  it(':todo appends to To Do and keeps the cursor', () => {
    const r = runCommand(withCmd(TPL, 'todo Buy milk'), ctx);
    expect(r.state.lines).toContain('- [ ] Buy milk');
    expect(r.state.cursor.line).toBe(0);
  });

  it(':todo inside a meeting tags the meeting name', () => {
    const lines = ['# T', '', '## To Do', '', '## Meetings', '### Sync', '', '## Notes', ''];
    const r = runCommand({ ...createEditorState(lines), command: 'todo Prep', cursor: { line: 5, col: 0 } }, ctx);
    expect(r.state.lines.some((l) => l === '- [ ] Prep _(Sync)_')).toBe(true);
  });

  it(':scheduled errors when not in a meeting', () => {
    const r = runCommand(withCmd(TPL, 'scheduled 14:30', 2), ctx);
    expect(r.state.message).toBe('Not in a meeting');
  });

  it(':start records the current time on the enclosing meeting', () => {
    const lines = ['# T', '', '## Meetings', '### Sync', '', '## Notes', ''];
    const r = runCommand({ ...createEditorState(lines), command: 'start', cursor: { line: 4, col: 0 } }, ctx);
    expect(r.state.lines).toContain('meta:started 09:30');
  });

  it(':section nests one level deeper than the enclosing heading', () => {
    const lines = ['## Meetings', '### Sync', 'body', '## Notes', ''];
    const r = runCommand({ ...createEditorState(lines), command: 'section Risks', cursor: { line: 2, col: 0 } }, ctx);
    expect(r.state.lines.some((l) => l === '#### Risks')).toBe(true);
  });

  it(':goto emits an effect without mutating the buffer', () => {
    const r = runCommand(withCmd(TPL, 'goto 2026-06-01'), ctx);
    expect(r.effect).toEqual({ type: 'goto', date: '2026-06-01' });
    expect(r.state.lines).toEqual(TPL);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — `runCommand` is not exported.

- [ ] **Step 3: Add the implementation to `web/src/lib/editor/commands.ts`**

Add these imports at the top of `commands.ts` (alongside the existing imports):
```ts
import type { EditorState } from './state';
import { pushUndo, clampCursor } from './state';
import { resolveContext, nearestHeadingLevel } from '../doc/context';
import { validateCommand } from '../doc/command';
```

Append the following to the end of `commands.ts`:
```ts
export type AppEffect =
  | { type: 'goto'; date: string }
  | { type: 'today' }
  | { type: 'tab'; date: string }
  | { type: 'close' }
  | { type: 'save' }
  | { type: 'theme'; theme: string }
  | { type: 'prevDay' }
  | { type: 'nextDay' }
  | { type: 'tabNext' }
  | { type: 'tabPrev' };

export interface CommandCtx {
  nowHHMM: string;
}
export interface CommandResult {
  state: EditorState;
  effect?: AppEffect;
}

function addBlock(state: EditorState, kind: 'meetings' | 'notes', name: string): EditorState {
  const ns = pushUndo(state);
  const ensured = ensureSection(ns.lines, kind);
  const { lines, headingIndex } = appendBlock(ensured.lines, ensured.section, `### ${name}`);
  return clampCursor({ ...ns, lines, cursor: { line: headingIndex, col: 0 }, message: '' });
}

function addTodo(state: EditorState, text: string): EditorState {
  const ns = pushUndo(state);
  const ctx = resolveContext(scanDocument(ns.lines), ns.cursor.line);
  const suffix = ctx.kind === 'meeting' ? ` _(${ctx.block.name})_` : '';
  const ensured = ensureSection(ns.lines, 'todo');
  const { lines } = appendLineToSection(ensured.lines, ensured.section, `- [ ] ${text}${suffix}`);
  return { ...ns, lines, message: '' }; // cursor stays
}

function addSubsection(state: EditorState, name: string): EditorState {
  const level = nearestHeadingLevel(state.lines, state.cursor.line);
  if (level === null) return { ...state, message: 'No enclosing heading' };
  if (level >= 6) return { ...state, message: 'Max heading depth' };
  const ns = pushUndo(state);
  const heading = `${'#'.repeat(level + 1)} ${name}`;
  const insertAt = endOfEnclosingSection(ns.lines, ns.cursor.line, level);
  const lines = ns.lines.slice();
  lines.splice(insertAt, 0, heading, '');
  return clampCursor({ ...ns, lines, cursor: { line: insertAt, col: 0 }, message: '' });
}

function setMeta(
  state: EditorState,
  required: 'meeting' | 'note',
  key: string,
  value: string,
): EditorState {
  const ctx = resolveContext(scanDocument(state.lines), state.cursor.line);
  if (ctx.kind !== required) {
    return { ...state, message: required === 'meeting' ? 'Not in a meeting' : 'Not in a note' };
  }
  const ns = pushUndo(state);
  const { lines } = upsertMeta(ns.lines, ctx.block, key, value);
  return { ...ns, lines, message: '' };
}

export function runCommand(state: EditorState, ctx: CommandCtx): CommandResult {
  const base: EditorState = { ...state, command: null };
  const v = validateCommand(state.command ?? '');
  if (!v.ok) return { state: { ...base, message: v.error } };
  const { command, arg } = v;
  switch (command) {
    case 'goto':
      return { state: { ...base, message: '' }, effect: { type: 'goto', date: arg } };
    case 'today':
      return { state: { ...base, message: '' }, effect: { type: 'today' } };
    case 'tab':
      return { state: { ...base, message: '' }, effect: { type: 'tab', date: arg } };
    case 'close':
      return { state: { ...base, message: '' }, effect: { type: 'close' } };
    case 'w':
      return { state: { ...base, message: 'Written' }, effect: { type: 'save' } };
    case 'theme':
      return { state: { ...base, message: `Theme: ${arg}` }, effect: { type: 'theme', theme: arg } };
    case 'meeting':
      return { state: addBlock(base, 'meetings', arg) };
    case 'note':
      return { state: addBlock(base, 'notes', arg) };
    case 'todo':
      return { state: addTodo(base, arg) };
    case 'section':
      return { state: addSubsection(base, arg) };
    case 'scheduled':
      return { state: setMeta(base, 'meeting', 'scheduled', arg) };
    case 'purpose':
      return { state: setMeta(base, 'meeting', 'purpose', arg) };
    case 'start':
      return { state: setMeta(base, 'meeting', 'started', ctx.nowHHMM) };
    case 'end':
      return { state: setMeta(base, 'meeting', 'ended', ctx.nowHHMM) };
    case 'topic':
      return { state: setMeta(base, 'note', 'topic', arg) };
  }
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `runCommand` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/editor/commands.ts web/src/lib/editor/commands.test.ts
git commit -m "feat: add runCommand with buffer mutations and app effects"
```

---

## Task 7: Key dispatcher

**Files:**
- Create: `web/src/lib/editor/keymap.ts`, `web/src/lib/editor/keymap.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/editor/keymap.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { createEditorState } from './state';
import { handleKey, type KeyInput } from './keymap';

const k = (key: string, mods: Partial<KeyInput> = {}): KeyInput => ({
  key,
  ctrl: false,
  meta: false,
  shift: false,
  ...mods,
});
const ctx = { nowHHMM: '09:30' };

describe('handleKey', () => {
  it('j moves down in normal mode', () => {
    expect(handleKey(createEditorState(['a', 'b']), k('j'), ctx).state.cursor.line).toBe(1);
  });

  it('i enters insert, typing inserts, Escape exits', () => {
    let s = handleKey(createEditorState(['']), k('i'), ctx).state;
    expect(s.mode).toBe('insert');
    s = handleKey(s, k('x'), ctx).state;
    expect(s.lines[0]).toBe('x');
    s = handleKey(s, k('Escape'), ctx).state;
    expect(s.mode).toBe('normal');
  });

  it('dd deletes a line via the pending operator', () => {
    let s = createEditorState(['a', 'b']);
    s = handleKey(s, k('d'), ctx).state;
    s = handleKey(s, k('d'), ctx).state;
    expect(s.lines).toEqual(['b']);
  });

  it('gg jumps to the first line', () => {
    let s = { ...createEditorState(['a', 'b', 'c']), cursor: { line: 2, col: 0 } };
    s = handleKey(s, k('g'), ctx).state;
    s = handleKey(s, k('g'), ctx).state;
    expect(s.cursor.line).toBe(0);
  });

  it(': opens the command line and Enter runs it', () => {
    let s = createEditorState(['# T', '', '## To Do', '']);
    s = handleKey(s, k(':'), ctx).state;
    expect(s.command).toBe('');
    for (const ch of ['t', 'o', 'd', 'o', ' ', 'm']) s = handleKey(s, k(ch), ctx).state;
    const r = handleKey(s, k('Enter'), ctx);
    expect(r.state.lines).toContain('- [ ] m');
    expect(r.state.command).toBeNull();
  });

  it('gt emits a tabNext effect', () => {
    let s = createEditorState(['a']);
    s = handleKey(s, k('g'), ctx).state;
    expect(handleKey(s, k('t'), ctx).effect).toEqual({ type: 'tabNext' });
  });

  it('] emits nextDay; Ctrl-r redoes', () => {
    expect(handleKey(createEditorState(['a']), k(']'), ctx).effect).toEqual({ type: 'nextDay' });
    expect(handleKey(createEditorState(['a']), k('r', { ctrl: true }), ctx).state.message).toContain('newest');
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
npm test
```
Expected: FAIL — cannot resolve `./keymap`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/editor/keymap.ts`:
```ts
import type { EditorState } from './state';
import { undo, redo } from './state';
import * as M from './motions';
import * as E from './edits';
import * as I from './insert';
import { runCommand, type AppEffect, type CommandCtx } from './commands';

export interface KeyInput {
  key: string;
  ctrl: boolean;
  meta: boolean;
  shift: boolean;
}
export interface KeyResult {
  state: EditorState;
  effect?: AppEffect;
}

export function handleKey(state: EditorState, key: KeyInput, ctx: CommandCtx): KeyResult {
  if (state.command !== null) return handleCommandMode(state, key, ctx);
  if (state.mode === 'insert') return handleInsertMode(state, key);
  return handleNormalMode(state, key, ctx);
}

function handleCommandMode(state: EditorState, key: KeyInput, ctx: CommandCtx): KeyResult {
  if (key.key === 'Escape') return { state: { ...state, command: null, message: '' } };
  if (key.key === 'Enter') return runCommand(state, ctx);
  if (key.key === 'Backspace') return { state: { ...state, command: (state.command ?? '').slice(0, -1) } };
  if (key.key.length === 1 && !key.ctrl && !key.meta) {
    return { state: { ...state, command: (state.command ?? '') + key.key } };
  }
  return { state };
}

function handleInsertMode(state: EditorState, key: KeyInput): KeyResult {
  switch (key.key) {
    case 'Escape':
      return { state: I.exitInsert(state) };
    case 'Backspace':
      return { state: I.backspace(state) };
    case 'Enter':
      return { state: I.insertNewline(state) };
    case 'Tab':
      return { state: I.insertTab(state) };
    case 'ArrowLeft':
      return { state: M.moveLeft(state) };
    case 'ArrowRight':
      return { state: M.moveRight(state) };
    case 'ArrowUp':
      return { state: M.moveUp(state) };
    case 'ArrowDown':
      return { state: M.moveDown(state) };
  }
  if (key.ctrl && (key.key === 'w' || key.key === 'W')) return { state: I.deleteWordBefore(state) };
  if (key.key.length === 1 && !key.ctrl && !key.meta) return { state: I.insertText(state, key.key) };
  return { state };
}

function handleNormalMode(state: EditorState, key: KeyInput, ctx: CommandCtx): KeyResult {
  if (state.pending === 'g') {
    const s = { ...state, pending: '' as const };
    if (key.key === 'g') return { state: M.firstLine(s) };
    if (key.key === 't') return { state: s, effect: { type: 'tabNext' } };
    if (key.key === 'T') return { state: s, effect: { type: 'tabPrev' } };
    return handleNormalMode(s, key, ctx);
  }
  if (state.pending === 'd') {
    const s = { ...state, pending: '' as const };
    if (key.key === 'd') return { state: E.deleteLine(s) };
    return handleNormalMode(s, key, ctx);
  }
  if (state.pending === 'y') {
    const s = { ...state, pending: '' as const };
    if (key.key === 'y') return { state: E.yankLine(s) };
    return handleNormalMode(s, key, ctx);
  }

  switch (key.key) {
    case 'h':
    case 'ArrowLeft':
      return { state: M.moveLeft(state) };
    case 'l':
    case 'ArrowRight':
      return { state: M.moveRight(state) };
    case 'j':
    case 'ArrowDown':
      return { state: M.moveDown(state) };
    case 'k':
    case 'ArrowUp':
      return { state: M.moveUp(state) };
    case 'w':
      return { state: M.wordForward(state) };
    case 'b':
      return { state: M.wordBackward(state) };
    case 'e':
      return { state: M.wordEnd(state) };
    case '0':
      return { state: M.lineStart(state) };
    case '$':
      return { state: M.lineEnd(state) };
    case 'G':
      return { state: M.lastLine(state) };
    case 'g':
      return { state: { ...state, pending: 'g' } };
    case 'd':
      return { state: { ...state, pending: 'd' } };
    case 'y':
      return { state: { ...state, pending: 'y' } };
    case 'x':
      return { state: E.deleteChar(state) };
    case 'p':
      return { state: E.pasteBelow(state) };
    case 'P':
      return { state: E.pasteAbove(state) };
    case 't':
      return { state: E.toggleTodo(state) };
    case 'u':
      return { state: undo(state) };
    case 'i':
      return { state: I.enterInsert(state) };
    case 'a':
      return { state: I.enterInsertAfter(state) };
    case 'A':
      return { state: I.enterInsertLineEnd(state) };
    case 'o':
      return { state: I.openBelow(state) };
    case 'O':
      return { state: I.openAbove(state) };
    case ':':
      return { state: { ...state, command: '', message: '' } };
    case 'Enter':
      return { state: M.moveDown(state) };
    case '[':
      return { state, effect: { type: 'prevDay' } };
    case ']':
      return { state, effect: { type: 'nextDay' } };
    case 'Escape':
      return { state: { ...state, pending: '', message: '' } };
  }
  if (key.ctrl && (key.key === 'r' || key.key === 'R')) return { state: redo(state) };
  if (key.ctrl && (key.key === 't' || key.key === 'T')) return { state, effect: { type: 'today' } };
  return { state };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run:
```bash
npm test
```
Expected: PASS — all `handleKey` tests green.

- [ ] **Step 5: Run the full unit suite**

Run:
```bash
npm test
```
Expected: PASS — Phase 1 doc suites, Phase 3 suites, and all editor suites (state/motions/edits/insert/commands/keymap) green.

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/editor/keymap.ts web/src/lib/editor/keymap.test.ts
git commit -m "feat: add key dispatcher across normal/insert/command modes"
```

---

## Task 8: Store integration (editor state, effects, autosave)

**Files:**
- Replace: `web/src/lib/appState.svelte.ts`

This rewrites the Phase 3 store to own an `EditorState`, forward keystrokes through `handleKey`, run `AppEffect`s, share the register across tabs, and autosave (debounce + flush). The `noteContent` field is replaced by `editor`.

- [ ] **Step 1: Replace `web/src/lib/appState.svelte.ts` with:**

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
import { getConfig, listNotes, getNote, putNote } from './api';
import type { UiConfig } from './types';
import { createEditorState, type EditorState } from './editor/state';
import { handleKey, type KeyInput } from './editor/keymap';
import type { AppEffect } from './editor/commands';

function nowHHMM(d: Date): string {
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
}

class AppStore {
  tabsState = $state<TabsState>(initTabs(todayISO()));
  editor = $state<EditorState>(createEditorState(['']));
  notesWithFiles = $state<string[]>([]);
  config = $state<UiConfig | null>(null);
  now = $state<Date>(new Date());
  calendar = $state<{ year: number; month: number }>(yearMonth(todayISO()));

  private sharedRegister: string[] = [];
  private lastSaved = '';
  private saveTimer: ReturnType<typeof setTimeout> | null = null;

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
      const content = await getNote(date);
      this.lastSaved = content;
      this.editor = createEditorState(content.split('\n'), this.sharedRegister);
      this.calendar = yearMonth(date);
      await this.refreshNotesList();
    } catch (e) {
      console.error(e);
    }
  }

  // ---- keyboard ----
  onKey(input: KeyInput): void {
    const ctx = { nowHHMM: nowHHMM(this.now) };
    const before = this.editor.lines;
    const { state, effect } = handleKey(this.editor, input, ctx);
    this.editor = state;
    this.sharedRegister = state.register;
    if (state.lines !== before) this.scheduleSave();
    if (effect) void this.runEffect(effect);
  }

  private content(): string {
    const body = this.editor.lines.join('\n');
    return body.endsWith('\n') ? body : body + '\n';
  }

  private normalized(s: string): string {
    return s.endsWith('\n') ? s : s + '\n';
  }

  private scheduleSave(): void {
    if (this.saveTimer) clearTimeout(this.saveTimer);
    this.saveTimer = setTimeout(() => void this.flush(), 750);
  }

  async flush(): Promise<void> {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
      this.saveTimer = null;
    }
    const content = this.content();
    if (content === this.normalized(this.lastSaved)) return;
    try {
      await putNote(this.activeDate, content);
      this.lastSaved = content;
    } catch (e) {
      this.editor = { ...this.editor, message: 'Save failed' };
      console.error(e);
    }
  }

  private async runEffect(effect: AppEffect): Promise<void> {
    switch (effect.type) {
      case 'goto':
        return this.goToDate(effect.date);
      case 'today':
        return this.goToDate(todayISO());
      case 'tab':
        return this.openInNewTab(effect.date);
      case 'close':
        return this.closeActive();
      case 'save':
        return this.flush();
      case 'prevDay':
        return this.goToDate(addDays(this.activeDate, -1));
      case 'nextDay':
        return this.goToDate(addDays(this.activeDate, 1));
      case 'tabNext':
        await this.flush();
        this.tabsState = nextTab(this.tabsState);
        return this.loadActive();
      case 'tabPrev':
        await this.flush();
        this.tabsState = prevTab(this.tabsState);
        return this.loadActive();
      case 'theme':
        if (this.config) {
          this.config = { ...this.config, theme: effect.theme };
          applyTheme(effect.theme, this.config.font, this.config.colors);
        }
        return;
    }
  }

  // ---- navigation (flush the current buffer first) ----
  async goToDate(date: string): Promise<void> {
    await this.flush();
    this.tabsState = retarget(this.tabsState, date);
    await this.loadActive();
  }
  async openInNewTab(date: string): Promise<void> {
    await this.flush();
    this.tabsState = openNewTab(this.tabsState, date);
    await this.loadActive();
  }
  async switchTab(index: number): Promise<void> {
    await this.flush();
    this.tabsState = { tabs: this.tabsState.tabs, activeIndex: index };
    await this.loadActive();
  }
  async closeActive(): Promise<void> {
    await this.flush();
    this.tabsState = closeTab(this.tabsState, this.tabsState.activeIndex, todayISO());
    await this.loadActive();
  }
  async closeAt(index: number): Promise<void> {
    await this.flush();
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

- [ ] **Step 2: Type-check**

Run (from `web/`):
```bash
npm run check
```
Expected: PASS. (If `svelte-check` flags an unused import in `Calendar.svelte`/`Tabs.svelte`, none should change — they still use `goToDate`/`openInNewTab`/`switchTab`/`closeAt`/`prevMonth`/`nextMonth`, all retained.)

- [ ] **Step 3: Commit**

```bash
git add web/src/lib/appState.svelte.ts
git commit -m "feat: wire editor state, effects, and autosave into the store"
```

---

## Task 9: EditorPane, StatusLine & keyboard wiring

**Files:**
- Replace: `web/src/lib/components/EditorPane.svelte`
- Create: `web/src/lib/components/StatusLine.svelte`
- Modify: `web/src/App.svelte`

- [ ] **Step 1: Replace `web/src/lib/components/EditorPane.svelte`**

```svelte
<script lang="ts">
  import { app } from '../appState.svelte';
  import { classifyLine } from '../doc/classify';
  import { renderInline } from '../doc/renderInline';

  let container: HTMLDivElement;
  let activeEl: HTMLDivElement | null = $state(null);

  function prettyHtml(raw: string): string {
    const c = classifyLine(raw);
    switch (c.kind) {
      case 'blank':
        return '&nbsp;';
      case 'heading':
        return `<span class="h h${c.level}">${renderInline(c.text)}</span>`;
      case 'task':
        return `<span class="task ${c.done ? 'done' : ''}"><span class="box">${
          c.done ? '☑' : '☐'
        }</span> ${renderInline(c.text)}</span>`;
      case 'meta':
        return `<span class="meta"><span class="mk">${renderInline(c.metaKey ?? '')}</span> ${renderInline(
          c.text,
        )}</span>`;
      case 'list':
        return `<span class="li">• ${renderInline(c.text)}</span>`;
      default:
        return renderInline(c.text);
    }
  }

  const cur = $derived(app.editor.cursor);
  const activeText = $derived(app.editor.lines[cur.line] ?? '');
  const before = $derived(activeText.slice(0, cur.col));
  const cursorChar = $derived(activeText.slice(cur.col, cur.col + 1) || ' ');
  const after = $derived(activeText.slice(cur.col + 1));

  $effect(() => {
    void app.editor.cursor.line; // re-run when the active line changes
    if (!container || !activeEl) return;
    const pos = app.config?.edit_line_position ?? 0.5;
    const target = activeEl.offsetTop - container.clientHeight * pos;
    container.scrollTop = Math.max(0, target);
  });
</script>

<div class="editor" bind:this={container}>
  {#each app.editor.lines as line, i (i)}
    {#if i === cur.line}
      <div class="line active" bind:this={activeEl}>
        <span class="raw">{before}<span class="cursor {app.editor.mode}">{cursorChar}</span>{after}</span>
      </div>
    {:else}
      <div class="line">{@html prettyHtml(line)}</div>
    {/if}
  {/each}
</div>

<style>
  .editor { flex: 1; min-width: 0; overflow-y: auto; padding: 1rem 1.5rem; line-height: 1.6; }
  .line { white-space: pre-wrap; word-break: break-word; min-height: 1.6em; }
  .line.active .raw { font-family: ui-monospace, 'SF Mono', monospace; }
  .cursor.normal { background: var(--cursor); color: var(--bg); }
  .cursor.insert { border-left: 2px solid var(--cursor); margin-left: -1px; }
  :global(.h) { font-weight: 700; }
  :global(.h1) { font-size: 1.5rem; color: var(--heading-1); }
  :global(.h2) { font-size: 1.3rem; color: var(--heading-2); }
  :global(.h3) { font-size: 1.15rem; color: var(--heading-3); }
  :global(.h4) { font-size: 1.05rem; color: var(--heading-4); }
  :global(.h5) { font-size: 1rem; color: var(--heading-5); }
  :global(.h6) { font-size: 0.95rem; color: var(--heading-6); }
  :global(.task.done) { color: var(--todo-done); text-decoration: line-through; }
  :global(.meta) { color: var(--meta); font-size: 0.85em; }
  :global(.mk) { text-transform: uppercase; letter-spacing: 0.05em; font-weight: 600; margin-right: 0.25em; }
</style>
```
Note: rendered line HTML uses global class names (the markup is injected via `{@html}`), so the heading/task/meta styles are marked `:global(...)`.

- [ ] **Step 2: Create `web/src/lib/components/StatusLine.svelte`**

```svelte
<script lang="ts">
  import { app } from '../appState.svelte';
  import { scanDocument } from '../doc/scan';
  import { resolveContext } from '../doc/context';

  const modeLabel = $derived(app.editor.mode === 'insert' ? '-- INSERT --' : '-- NORMAL --');

  const context = $derived.by(() => {
    const ctx = resolveContext(scanDocument(app.editor.lines), app.editor.cursor.line);
    switch (ctx.kind) {
      case 'todo':
        return 'To Do';
      case 'meeting':
        return `Meetings › ${ctx.block.name}`;
      case 'note':
        return `Notes › ${ctx.block.name}`;
      case 'other':
        return ctx.section.title;
      default:
        return '';
    }
  });
</script>

<footer class="status">
  {#if app.editor.command !== null}
    <span class="cmd">:{app.editor.command}</span>
  {:else}
    <span class="mode">{modeLabel}</span>
    <span class="ctx">{context}</span>
  {/if}
  <span class="msg">{app.editor.message}</span>
</footer>

<style>
  .status {
    display: flex; gap: 1rem; align-items: center;
    padding: 0.25rem 1rem; background: var(--status-bar); color: var(--muted);
    font-size: 0.8rem; font-family: ui-monospace, 'SF Mono', monospace;
    border-top: 1px solid var(--status-bar);
  }
  .mode { font-weight: 700; color: var(--fg); }
  .cmd { color: var(--fg); }
  .msg { margin-left: auto; color: var(--accent); }
</style>
```

- [ ] **Step 3: Update `web/src/App.svelte` to forward keystrokes and mount the status line**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import Header from './lib/components/Header.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import EditorPane from './lib/components/EditorPane.svelte';
  import StatusLine from './lib/components/StatusLine.svelte';
  import { app } from './lib/appState.svelte';

  onMount(() => {
    app.init();
  });

  function onKeydown(e: KeyboardEvent) {
    // Let browser-level Cmd/Meta shortcuts (reload, devtools) and function keys through.
    if (e.metaKey) return;
    if (/^F\d{1,2}$/.test(e.key)) return;
    if (e.key === 'Shift' || e.key === 'Control' || e.key === 'Alt' || e.key === 'Meta') return;
    e.preventDefault();
    app.onKey({ key: e.key, ctrl: e.ctrlKey, meta: e.metaKey, shift: e.shiftKey });
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="app">
  <Header />
  <div class="body">
    <EditorPane />
    <Sidebar />
  </div>
  <StatusLine />
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
  .body { display: flex; flex: 1; min-height: 0; }
</style>
```
Caveat: with `e.preventDefault()` the app captures most keys globally (intended for a keyboard-first tool). `Ctrl-T` (today) may still be intercepted by the browser on some platforms; `:today` is the reliable alternative.

- [ ] **Step 4: Type-check and build**

Run (from `web/`):
```bash
npm run check && npm run build && npm test
```
Expected: `svelte-check` clean; Vite build succeeds; all unit tests pass.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/components/EditorPane.svelte web/src/lib/components/StatusLine.svelte web/src/App.svelte
git commit -m "feat: render modal editor with per-line view, cursor, status line"
```

---

## Task 10: End-to-end manual verification

**Files:** none (verification only)

- [ ] **Step 1: Start the backend and frontend**

Terminal 1 (repo root):
```bash
cargo run -- --notes-dir ./dev-notes --no-open
```
Terminal 2 (repo root):
```bash
make dev-web
```
Open the Vite URL (typically `http://localhost:5173`). Click into the window so it has focus.

- [ ] **Step 2: Verify NORMAL-mode navigation & rendering**

- The note renders **pretty** (heading styled, sections visible) except the **active line**, which shows **raw** markup with a **block cursor**; the status line shows `-- NORMAL --` and the context (e.g. `To Do`).
- `j`/`k`/`h`/`l` and arrows move the cursor; the active (raw) line follows and stays near the vertical center; `gg`/`G` jump to first/last line; `w`/`b`/`e` move by word; `0`/`$` jump to line ends.

- [ ] **Step 3: Verify INSERT mode & edits**

- `i`/`a`/`A`/`o`/`O` enter INSERT (status shows `-- INSERT --`, cursor becomes a bar); typing inserts text; `Enter` splits lines; `Backspace` joins; `Tab` inserts two spaces; `Ctrl-W` deletes the previous word; `Esc` returns to NORMAL.
- `x` deletes a char; `dd` deletes+yanks a line; `yy` then `p`/`P` paste; `t` toggles a `- [ ]` task; `u` undoes and `Ctrl-R` redoes (one INSERT session = one undo unit; each NORMAL edit and each command = one unit).

- [ ] **Step 4: Verify the command line**

- `:` opens the command line in the status bar; type and `Enter`:
  - `:meeting Daily Standup` → a `### Daily Standup` appears under Meetings and the cursor moves there.
  - `:scheduled 14:30`, `:purpose Plan launch`, `:start`, `:end` → `meta:` lines upsert under the meeting (run `:scheduled` twice → still one line, updated).
  - From a non-meeting line, `:scheduled` shows `Not in a meeting`.
  - `:note Architecture` then `:topic Caching` → note + meta.
  - `:section Risks` inside a meeting → `#### Risks` appended; cursor moves in; context still shows the meeting.
  - `:todo Buy milk` from To Do → appends `- [ ] Buy milk` (cursor stays); from inside a meeting → appends `- [ ] X _(Meeting Name)_`.
  - `:theme dark` / `:theme light` switch theme live; `:w` writes immediately.
  - An unknown command shows `Unknown command: :...`.

- [ ] **Step 5: Verify navigation, tabs, and autosave-to-disk**

- `]`/`[` step to next/previous day (buffer flushes first); `gt`/`gT` cycle tabs; `:goto 2026-06-01`, `:tab 2026-06-02` (new tab), `:close` work; calendar clicks and Cmd/Ctrl-click still navigate/open tabs.
- Make an edit, wait ~1s, then in a third terminal confirm the file changed on disk:
  ```bash
  cat ./dev-notes/$(date +%F).md
  ```
  Expected: your edits are present with a trailing newline. Switching tabs/days also flushes immediately.
- Yank a line in one day (`yy`), switch days, `p` → the **shared register** pastes it into the other day.

- [ ] **Step 6: Verify the production build**

```bash
make build
./target/release/slugline --notes-dir ./dev-notes --no-open --port 4747
```
Open `http://127.0.0.1:4747` and re-check a few editing + command + autosave behaviors against the embedded SPA. Stop the server when done.

---

## Phase 4 Done Criteria

- `cd web && npm test` is green across all editor suites (state, motions, edits, insert, commands, keymap) plus prior phases.
- `npm run check` is clean; `npm run build` succeeds.
- The full editing loop works against real files: modal editing, all listed motions/edits, undo/`Ctrl-R`, the shared register across tabs, every `:` command (buffer mutations + navigation/theme/save effects), per-line pretty rendering with a single raw edit line and centered scroll, and debounced autosave + flush-on-navigation verified on disk.

## Self-Review Notes (performed during authoring)

- **Spec coverage (roadmap Phase 4 row):** NORMAL/INSERT (state+keymap), motions `h j k l w b e 0 $ gg G` (motions+keymap), edits `x dd yy p P o O i a A t` (edits/insert/keymap), undo + `Ctrl-R` (state+keymap), shared line-wise register (state.register + store.sharedRegister), per-line pretty render + single raw edit line + centered scroll (EditorPane), `:` command line + interpreter for `meeting/note/section/todo/start/end/scheduled/purpose/topic/goto/today/tab/close/w/theme` (commands+keymap), autosave debounce+flush (store) — all present. `[`/`]`/`Ctrl-T`/`gt`/`gT` are bound here (moving the Phase 3 placeholder global keys into the editor mode machine, as Phase 3 noted).
- **Type consistency:** `EditorState`/`Cursor`/`Pending` defined once (state.ts); motions/edits/insert all consume and return `EditorState`. `AppEffect`/`CommandCtx`/`runCommand` (commands.ts) are consumed by keymap and the store. `KeyInput` (keymap.ts) is produced by `App.svelte` and consumed by `store.onKey`. Store methods referenced by Phase 3 components (`goToDate`, `openInNewTab`, `switchTab`, `closeAt`, `prevMonth`, `nextMonth`) are all retained.
- **Undo granularity:** mode-entry functions (`enterInsert`/`a`/`A`/`o`/`O`) push exactly one snapshot per INSERT session; in-session edits push none; each NORMAL edit and each mutating command pushes one — matching the agreed model.
- **No placeholders:** every code step contains complete code; UI behavior that can't be unit-tested is covered by the explicit manual checklist in Task 10.

