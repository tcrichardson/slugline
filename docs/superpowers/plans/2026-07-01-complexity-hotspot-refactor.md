# Complexity Hotspot Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove a duplicated `isValidDate` implementation, extract a shared boundary-scanning helper in `scan.ts`, and replace `validateCommand`'s linear `if` chain with a per-`ArgKind` validator map — with zero behavior change.

**Architecture:** Pure internal refactor of three independent spots in `web/src/lib`. No public type or function signature changes to `DocModel`, `Section`, `Block`, `ValidationResult`, or `CommandSpec`. Each task is a self-contained, independently-committable change verified by the existing test suite (this is a refactor of already-tested code, not new functionality, so each task's "test" step is running the existing suite before and after the change rather than writing new failing tests).

**Tech Stack:** TypeScript, Vitest (run from `web/`).

**Spec:** `docs/superpowers/specs/2026-07-01-complexity-hotspot-refactor-design.md`

---

### Task 1: Dedupe `isValidDate` between `dates.ts` and `command.ts`

**Files:**
- Modify: `web/src/lib/doc/command.ts:55-63` (remove duplicate regex + function, add import)
- Modify: `web/src/lib/doc/command.test.ts:1-2,57-62` (remove `isValidDate` import and its test block)

- [ ] **Step 1: Confirm baseline is green**

Run: `npm test -- doc/command.test.ts doc/scan.test.ts` (workdir: `web/`)
Expected: All tests PASS (this establishes the pre-refactor baseline before any edits).

- [ ] **Step 2: Remove the duplicate `isValidDate` from `command.ts` and import it from `dates.ts`**

In `web/src/lib/doc/command.ts`, the current top-of-file has:

```ts
export interface ParsedCommand {
  name: string;
  arg: string;
}
```

Add an import line above it:

```ts
import { isValidDate } from '../dates';

export interface ParsedCommand {
  name: string;
  arg: string;
}
```

Then find this block (currently at lines 55-63):

```ts
const TIME = /^([01]\d|2[0-3]):[0-5]\d$/;
const DATE = /^\d{4}-\d{2}-\d{2}$/;

export function isValidDate(s: string): boolean {
  if (!DATE.test(s)) return false;
  const [y, m, d] = s.split('-').map(Number);
  const dt = new Date(Date.UTC(y, m - 1, d));
  return dt.getUTCFullYear() === y && dt.getUTCMonth() === m - 1 && dt.getUTCDate() === d;
}
```

Replace it with just:

```ts
const TIME = /^([01]\d|2[0-3]):[0-5]\d$/;
```

The `DATE` regex and the local `isValidDate` function are both deleted — `validateCommand`'s date check (currently `if (spec.argKind === 'date' && !isValidDate(arg))`) now resolves to the imported `isValidDate` from `../dates` instead. No other line in `command.ts` needs to change; the call site is untouched because the imported function has the identical name and signature.

- [ ] **Step 3: Update `command.test.ts` to stop importing/testing the removed local `isValidDate`**

In `web/src/lib/doc/command.test.ts`, change the import line (line 2) from:

```ts
import { parseCommandLine, validateCommand, isValidDate } from './command';
```

to:

```ts
import { parseCommandLine, validateCommand } from './command';
```

Then remove this entire block (currently lines 57-62):

```ts
describe('isValidDate', () => {
  it('rejects impossible calendar dates', () => {
    expect(isValidDate('2026-02-30')).toBe(false);
    expect(isValidDate('2026-02-28')).toBe(true);
  });
});
```

This coverage is not lost — it's already present verbatim in `web/src/lib/dates.test.ts`'s `'validates ISO calendar dates'` test, which asserts the same `2026-02-30` / valid-date cases against the shared `isValidDate`.

- [ ] **Step 4: Run tests to verify no regressions**

Run: `npm test -- doc/command.test.ts dates.test.ts` (workdir: `web/`)
Expected: All tests PASS. If `command.ts` fails to compile because `isValidDate` isn't found, double check the import path is `'../dates'` (relative to `web/src/lib/doc/command.ts`, `dates.ts` lives at `web/src/lib/dates.ts`, i.e. one directory up).

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/doc/command.ts web/src/lib/doc/command.test.ts
git commit -m "refactor: dedupe isValidDate, import from dates.ts in command.ts"
```

---

### Task 2: Extract shared boundary-scanning helper in `scan.ts`

**Files:**
- Modify: `web/src/lib/doc/scan.ts:12-47,63-89`

- [ ] **Step 1: Confirm baseline is green**

Run: `npm test -- doc/scan.test.ts` (workdir: `web/`)
Expected: All 6 tests PASS (baseline before refactor).

- [ ] **Step 2: Add the `findBoundaryEnd` helper**

In `web/src/lib/doc/scan.ts`, immediately after the existing `sectionKind` function (after line 10, before `collectBlocks`), add:

```ts
/**
 * Scans forward from `from` to `to` (inclusive) and returns the index just
 * before the first heading whose level is `<= maxLevel`, or `to` if none is
 * found. Used to find where an H3 block or H2 section ends.
 */
function findBoundaryEnd(classified: ClassifiedLine[], from: number, to: number, maxLevel: number): number {
  for (let j = from; j <= to; j++) {
    const c = classified[j];
    if (c.kind === 'heading' && c.level! <= maxLevel) return j - 1;
  }
  return to;
}
```

- [ ] **Step 3: Use `findBoundaryEnd` in `collectBlocks`**

Replace the inner loop in `collectBlocks` (currently):

```ts
    if (c.kind === 'heading' && c.level === 3) {
      const start = i;
      let end = to;
      for (let j = i + 1; j <= to; j++) {
        const cj = classified[j];
        if (cj.kind === 'heading' && cj.level! <= 3) {
          end = j - 1;
          break;
        }
      }
```

with:

```ts
    if (c.kind === 'heading' && c.level === 3) {
      const start = i;
      const end = findBoundaryEnd(classified, i + 1, to, 3);
```

The rest of `collectBlocks` (the meta-collection loop and `blocks.push(...)`) is unchanged.

- [ ] **Step 4: Use `findBoundaryEnd` in `scanDocument`**

Replace the inner loop in `scanDocument`'s section-finding block (currently):

```ts
    if (c.kind === 'heading' && c.level === 2) {
      const start = i;
      let end = lines.length - 1;
      for (let j = i + 1; j < classified.length; j++) {
        const cj = classified[j];
        if (cj.kind === 'heading' && cj.level! <= 2) {
          end = j - 1;
          break;
        }
      }
```

with:

```ts
    if (c.kind === 'heading' && c.level === 2) {
      const start = i;
      const end = findBoundaryEnd(classified, i + 1, classified.length - 1, 2);
```

The rest of `scanDocument` (building the `Section` object and pushing it) is unchanged.

- [ ] **Step 5: Run tests to verify no regressions**

Run: `npm test -- doc/scan.test.ts` (workdir: `web/`)
Expected: All 6 tests PASS, identical results to Step 1's baseline (title detection, section-kind detection, H3 block/meta collection, boundary bounding, malformed-doc handling, no-meta block handling).

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/doc/scan.ts
git commit -m "refactor: extract findBoundaryEnd helper shared by collectBlocks and scanDocument"
```

---

### Task 3: Replace `validateCommand`'s `if` chain with a per-`ArgKind` validator map

**Files:**
- Modify: `web/src/lib/doc/command.ts:65-79` (after Task 1's edits, this is the `validateCommand` function and everything below the `TIME` regex)

- [ ] **Step 1: Confirm baseline is green**

Run: `npm test -- doc/command.test.ts` (workdir: `web/`)
Expected: All tests PASS (baseline before this refactor; Task 1 must be committed first since this task's before/after code assumes `isValidDate` is imported, not locally defined).

- [ ] **Step 2: Add the `ARG_VALIDATORS` map and simplify `validateCommand`**

In `web/src/lib/doc/command.ts`, replace the current `validateCommand` function and everything from the `TIME` regex down (post-Task-1, this is):

```ts
const TIME = /^([01]\d|2[0-3]):[0-5]\d$/;

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

with:

```ts
const TIME = /^([01]\d|2[0-3]):[0-5]\d$/;

/** One validator per ArgKind. Returns an error message, or null if `arg` is valid for that kind. */
const ARG_VALIDATORS: Record<ArgKind, (arg: string) => string | null> = {
  none: () => null,
  text: () => null,
  time: (arg) => (TIME.test(arg) ? null : 'Expected HH:MM'),
  date: (arg) => (isValidDate(arg) ? null : 'Expected YYYY-MM-DD'),
  theme: (arg) => (arg === '' || arg === 'light' || arg === 'dark' ? null : 'Expected light or dark'),
};

export function validateCommand(input: string): ValidationResult {
  const { name, arg } = parseCommandLine(input);
  const resolved = ALIASES[name] ?? name;
  if (!(resolved in COMMANDS)) return { ok: false, error: `Unknown command: :${name}` };
  const spec = COMMANDS[resolved as CommandName];

  if (spec.argRequired && arg === '') return { ok: false, error: `:${name} requires an argument` };
  const error = ARG_VALIDATORS[spec.argKind](arg);
  if (error) return { ok: false, error };

  return { ok: true, command: spec.name, arg };
}
```

Note `ARG_VALIDATORS` must be defined above `validateCommand` (or anywhere in module scope, since function bodies are evaluated at call time, but keep it directly above `validateCommand` for readability) and below the `ArgKind` type definition (already defined earlier in the file at line 19).

- [ ] **Step 3: Run tests to verify no regressions**

Run: `npm test -- doc/command.test.ts` (workdir: `web/`)
Expected: All tests PASS, identical results to Step 1's baseline — specifically the `validateCommand` describe block's 9 cases (valid text command, unknown command, missing required arg, HH:MM validation, YYYY-MM-DD validation, theme validation, no-arg commands, optional theme arg) and the `:people`/`:p` alias cases.

- [ ] **Step 4: Commit**

```bash
git add web/src/lib/doc/command.ts
git commit -m "refactor: replace validateCommand if-chain with per-ArgKind validator map"
```

---

### Task 4: Full verification pass

**Files:** none (verification only)

- [ ] **Step 1: Run the full web test suite**

Run: `npm test` (workdir: `web/`)
Expected: All tests PASS (this is `vitest run --passWithNoTests` across every `*.test.ts` file in `web/src`, confirming Tasks 1-3 didn't regress anything outside the directly-touched files, e.g. `editor/commands.ts`'s `runCommand` which calls `validateCommand`).

- [ ] **Step 2: Run the TypeScript/Svelte type checker**

Run: `npm run check` (workdir: `web/`)
Expected: No type errors. This confirms `ARG_VALIDATORS: Record<ArgKind, ...>` covers every `ArgKind` variant (a missing key would be a TypeScript error) and that the `isValidDate` import resolves correctly.

- [ ] **Step 3: Re-run the complexity report (optional sanity check)**

If a complexity analysis command/script is available in the project (see `.complexity/` directory for prior output naming convention), re-run it and confirm:
- `web/src/lib/doc/command.ts`'s `isValidDate` no longer appears as a structural duplication candidate.
- `validateCommand`'s complexity score has dropped from 12.
- `web/src/lib/doc/scan.ts`'s max nesting depth has dropped from 4 to 3.

If no such command is readily available, skip this step — it's a nice-to-have confirmation, not a correctness requirement.
