# `:people` Command Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `:people` command (`:p` shortcut) that appends a comma-separated `meta:people` tag to meeting and note blocks.

**Architecture:** Two files change. `command.ts` gains a `ALIASES` map and a `'people'` command entry; `validateCommand` resolves aliases before COMMANDS lookup. `commands.ts` gains an `appendMeta` helper (appends to existing meta value) and a `case 'people'` handler that works in both meeting and note contexts.

**Tech Stack:** TypeScript, Vitest

---

## Task 1: Alias system and `people` command spec in `command.ts`

**Files:**
- Modify: `web/src/lib/doc/command.ts`
- Test: `web/src/lib/doc/command.test.ts`

- [ ] **Step 1: Write the failing tests**

Open `web/src/lib/doc/command.test.ts` and add a new `describe` block at the bottom of the file:

```typescript
describe('ALIASES and :people command', () => {
  it(':p Alice resolves to command people via validateCommand', () => {
    const r = validateCommand('p Alice Smith');
    expect(r.ok).toBe(true);
    if (r.ok) {
      expect(r.command).toBe('people');
      expect(r.arg).toBe('Alice Smith');
    }
  });

  it(':p with no argument fails validation', () => {
    const r = validateCommand('p');
    expect(r.ok).toBe(false);
  });

  it(':people resolves directly', () => {
    const r = validateCommand('people Bob Jones');
    expect(r.ok).toBe(true);
    if (r.ok) {
      expect(r.command).toBe('people');
      expect(r.arg).toBe('Bob Jones');
    }
  });
});
```

- [ ] **Step 2: Run to verify tests fail**

```bash
cd web && npm test -- --reporter=verbose 2>&1 | grep -A3 'ALIASES'
```

Expected: 3 failures — `Unknown command: :p`, `Unknown command: :people`

- [ ] **Step 3: Implement the alias system and `people` entry**

Replace the current content of `web/src/lib/doc/command.ts` with:

```typescript
export interface ParsedCommand {
  name: string;
  arg: string;
}

/** Parse the text typed after the leading ':' (the colon is not included). */
export function parseCommandLine(input: string): ParsedCommand {
  const trimmed = input.replace(/^\s+/, '');
  const sp = trimmed.indexOf(' ');
  if (sp === -1) return { name: trimmed.toLowerCase(), arg: '' };
  return { name: trimmed.slice(0, sp).toLowerCase(), arg: trimmed.slice(sp + 1).trim() };
}

export type CommandName =
  | 'meeting' | 'note' | 'section' | 'todo'
  | 'start' | 'end' | 'scheduled' | 'purpose' | 'topic' | 'people'
  | 'goto' | 'today' | 'tab' | 'close' | 'w' | 'theme';

export type ArgKind = 'none' | 'text' | 'time' | 'date' | 'theme';

export interface CommandSpec {
  name: CommandName;
  argKind: ArgKind;
  argRequired: boolean;
}

export const COMMANDS: Record<CommandName, CommandSpec> = {
  meeting:   { name: 'meeting',   argKind: 'text',  argRequired: true  },
  note:      { name: 'note',      argKind: 'text',  argRequired: true  },
  section:   { name: 'section',   argKind: 'text',  argRequired: true  },
  todo:      { name: 'todo',      argKind: 'text',  argRequired: true  },
  start:     { name: 'start',     argKind: 'none',  argRequired: false },
  end:       { name: 'end',       argKind: 'none',  argRequired: false },
  scheduled: { name: 'scheduled', argKind: 'time',  argRequired: true  },
  purpose:   { name: 'purpose',   argKind: 'text',  argRequired: true  },
  topic:     { name: 'topic',     argKind: 'text',  argRequired: true  },
  people:    { name: 'people',    argKind: 'text',  argRequired: true  },
  goto:      { name: 'goto',      argKind: 'date',  argRequired: true  },
  today:     { name: 'today',     argKind: 'none',  argRequired: false },
  tab:       { name: 'tab',       argKind: 'date',  argRequired: true  },
  close:     { name: 'close',     argKind: 'none',  argRequired: false },
  w:         { name: 'w',         argKind: 'none',  argRequired: false },
  theme:     { name: 'theme',     argKind: 'theme', argRequired: false },
};

/** Short aliases resolved before COMMANDS lookup. Add future shortcuts here. */
export const ALIASES: Record<string, CommandName> = {
  p: 'people',
};

export type ValidationResult =
  | { ok: true; command: CommandName; arg: string }
  | { ok: false; error: string };

const TIME = /^([01]\d|2[0-3]):[0-5]\d$/;
const DATE = /^\d{4}-\d{2}-\d{2}$/;

export function isValidDate(s: string): boolean {
  if (!DATE.test(s)) return false;
  const [y, m, d] = s.split('-').map(Number);
  const dt = new Date(Date.UTC(y, m - 1, d));
  return dt.getUTCFullYear() === y && dt.getUTCMonth() === m - 1 && dt.getUTCDate() === d;
}

export function validateCommand(input: string): ValidationResult {
  const { name, arg } = parseCommandLine(input);
  const resolved = ALIASES[name] ?? name;
  if (!(resolved in COMMANDS)) return { ok: false, error: `Unknown command: :${name}` };
  const spec = COMMANDS[resolved as CommandName];

  if (spec.argRequired && arg === '') return { ok: false, error: `:${name} requires an argument` };
  if (spec.argKind === 'time' && !TIME.test(arg)) return { ok: false, error: 'Expected HH:MM' };
  if (spec.argKind === 'date' && !isValidDate(arg)) return { ok: false, error: 'Expected YYYY-MM-DD' };
  if (spec.argKind === 'theme' && arg !== '' && arg !== 'light' && arg !== 'dark') {
    return { ok: false, error: 'Expected light or dark' };
  }

  return { ok: true, command: spec.name, arg };
}
```

- [ ] **Step 4: Run tests and verify they pass**

```bash
cd web && npm test -- --reporter=verbose 2>&1 | grep -E '(PASS|FAIL|people|ALIASES)'
```

Expected: all existing tests pass, 3 new tests pass.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/doc/command.ts web/src/lib/doc/command.test.ts
git commit -m "feat: add ALIASES map and :people command to command registry"
```

---

## Task 2: `appendMeta` helper in `commands.ts`

**Files:**
- Modify: `web/src/lib/editor/commands.ts`
- Test: `web/src/lib/editor/commands.test.ts`

- [ ] **Step 1: Write the failing tests**

Open `web/src/lib/editor/commands.test.ts`. At line 3, add `appendMeta` to the import:

```typescript
import { ensureSection, appendBlock, appendLineToSection, upsertMeta, appendMeta, endOfEnclosingSection, runCommand } from './commands';
```

Then add a new `describe` block after the existing `'command helpers'` describe block (after line 45):

```typescript
describe('appendMeta', () => {
  it('inserts meta:people when no prior value exists', () => {
    const lines = ['## Meetings', '### Sync', ''];
    const block = scanDocument(lines).sections[0].blocks[0];
    const { lines: out } = appendMeta(lines, block, 'people', 'Alice');
    expect(out).toContain('meta:people Alice');
  });

  it('appends comma-separated to an existing meta:people value', () => {
    const lines = ['## Meetings', '### Sync', 'meta:people Alice', ''];
    const block = scanDocument(lines).sections[0].blocks[0];
    const { lines: out } = appendMeta(lines, block, 'people', 'Bob');
    expect(out).toContain('meta:people Alice, Bob');
    expect(out.filter((l) => l.startsWith('meta:people')).length).toBe(1);
  });

  it('trims whitespace from the new value before appending', () => {
    const lines = ['## Meetings', '### Sync', 'meta:people Alice', ''];
    const block = scanDocument(lines).sections[0].blocks[0];
    const { lines: out } = appendMeta(lines, block, 'people', '  Bob  ');
    expect(out).toContain('meta:people Alice, Bob');
  });
});
```

- [ ] **Step 2: Run to verify tests fail**

```bash
cd web && npm test -- --reporter=verbose 2>&1 | grep -A3 'appendMeta'
```

Expected: 3 failures — `appendMeta is not a function` (or similar export error).

- [ ] **Step 3: Implement `appendMeta`**

Open `web/src/lib/editor/commands.ts`. After the closing `}` of `upsertMeta` (after line 59), insert:

```typescript
/** Append a new value to an existing `meta:key` line, or create it if absent.
 *  Values are joined with ", ".
 */
export function appendMeta(
  lines: string[],
  block: Block,
  key: string,
  newValue: string,
): { lines: string[]; index: number } {
  const trimmed = newValue.trim();
  const existing = block.meta.find((m) => m.key === key);
  if (existing && existing.value.trim() !== '') {
    return upsertMeta(lines, block, key, `${existing.value.trim()}, ${trimmed}`);
  }
  return upsertMeta(lines, block, key, trimmed);
}
```

- [ ] **Step 4: Run tests and verify they pass**

```bash
cd web && npm test -- --reporter=verbose 2>&1 | grep -E '(PASS|FAIL|appendMeta)'
```

Expected: all 3 new tests pass, no regressions.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/editor/commands.ts web/src/lib/editor/commands.test.ts
git commit -m "feat: add appendMeta helper for cumulative meta tag values"
```

---

## Task 3: `:people` handler in `runCommand`

**Files:**
- Modify: `web/src/lib/editor/commands.ts`
- Test: `web/src/lib/editor/commands.test.ts`

- [ ] **Step 1: Write the failing tests**

Open `web/src/lib/editor/commands.test.ts`. Add a new `describe` block at the end of the file:

```typescript
describe(':people command', () => {
  // Cursor at line 4 = body of "### Sync" meeting
  const meetingLines = ['# T', '', '## Meetings', '### Sync', '', '## Notes', ''];
  // Cursor at line 6 = body of "### Retro" note
  const noteLines = ['# T', '', '## Meetings', '', '## Notes', '### Retro', '', ''];

  it('sets meta:people in a meeting block', () => {
    const r = runCommand(
      { ...createEditorState(meetingLines), command: 'people Alice', cursor: { line: 4, col: 0 } },
      ctx,
    );
    expect(r.state.lines).toContain('meta:people Alice');
    expect(r.state.message).toBe('');
  });

  it('appends to existing meta:people in a meeting block', () => {
    const lines = ['# T', '', '## Meetings', '### Sync', 'meta:people Alice', '', '## Notes', ''];
    const r = runCommand(
      { ...createEditorState(lines), command: 'people Bob', cursor: { line: 5, col: 0 } },
      ctx,
    );
    expect(r.state.lines).toContain('meta:people Alice, Bob');
  });

  it('sets meta:people in a note block', () => {
    const r = runCommand(
      { ...createEditorState(noteLines), command: 'people Alice', cursor: { line: 6, col: 0 } },
      ctx,
    );
    expect(r.state.lines).toContain('meta:people Alice');
    expect(r.state.message).toBe('');
  });

  it('errors when not in a meeting or note block', () => {
    // TPL cursor line 2 = "## To Do" section heading, not inside any block
    const r = runCommand(withCmd(TPL, 'people Alice', 2), ctx);
    expect(r.state.message).toContain('meeting or note');
  });

  it(':p shortcut works end-to-end through runCommand', () => {
    const r = runCommand(
      { ...createEditorState(meetingLines), command: 'p Alice', cursor: { line: 4, col: 0 } },
      ctx,
    );
    expect(r.state.lines).toContain('meta:people Alice');
  });
});
```

- [ ] **Step 2: Run to verify tests fail**

```bash
cd web && npm test -- --reporter=verbose 2>&1 | grep -E '(PASS|FAIL|people command)'
```

Expected: 5 failures — the `runCommand` switch has no `'people'` case so it falls off the switch and returns `undefined`.

- [ ] **Step 3: Add `case 'people'` to `runCommand`**

Open `web/src/lib/editor/commands.ts`. Inside `runCommand`, add the new case immediately after the `case 'topic':` line (after line 203):

```typescript
    case 'people': {
      const pCtx = resolveContext(scanDocument(base.lines), base.cursor.line);
      if (pCtx.kind !== 'meeting' && pCtx.kind !== 'note') {
        return { state: { ...base, message: 'Not in a meeting or note' } };
      }
      const ns = pushUndo(base);
      const { lines } = appendMeta(ns.lines, pCtx.block, 'people', arg);
      return { state: { ...ns, lines, message: '' } };
    }
```

- [ ] **Step 4: Run tests and verify they pass**

```bash
cd web && npm test
```

Expected: all tests pass, no regressions across the entire suite.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/editor/commands.ts web/src/lib/editor/commands.test.ts
git commit -m "feat: add :people / :p command for meeting and note blocks"
```
