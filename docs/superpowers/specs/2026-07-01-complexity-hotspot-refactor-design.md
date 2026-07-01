# Design: Refactor Complexity Hotspots (`scan.ts`, `validateCommand`, `isValidDate` dedup)

**Date:** 2026-07-01
**Status:** Approved

## Summary

A complexity report (`.complexity/complexity_web_20260701_1052.md`) flagged three low-risk, high-value cleanup items in the web frontend:

1. An exact duplicate of `isValidDate` between `web/src/lib/dates.ts` and `web/src/lib/doc/command.ts`.
2. `scanDocument`/`collectBlocks` in `web/src/lib/doc/scan.ts` duplicating a "scan forward for the next heading at or above a level" boundary-search loop, contributing to the file's max nesting depth (4, the highest in the project).
3. `validateCommand` in `web/src/lib/doc/command.ts` using a linear chain of `if` guards per `ArgKind`, giving it the worst complexity-to-size ratio in the project (complexity 12 in 15 lines).

This is a pure internal refactor: no public API shapes (`DocModel`, `Section`, `Block`, `ValidationResult`, `CommandSpec`) change, and no user-visible behavior changes. All existing tests are expected to pass unmodified except where noted below.

---

## 1. Dedupe `isValidDate`

`command.ts` currently redefines `isValidDate` identically to the version in `dates.ts` (confirmed by matching Halstead metrics in the complexity report).

**Change:**
- Delete the `ISO`/`DATE` regex + `isValidDate` definition from `command.ts`.
- Import `isValidDate` from `../dates` and use it in `validateCommand`'s `date` arg-kind check.
- Remove `command.ts`'s `isValidDate` export entirely (no re-export shim).

**Test changes:**
- `web/src/lib/doc/command.test.ts`: remove the `isValidDate` import and its `describe('isValidDate', ...)` block. Coverage for `isValidDate` itself remains fully intact via `web/src/lib/dates.test.ts`.

---

## 2. `scan.ts`: extract shared boundary-scanning helper

`collectBlocks` and `scanDocument` each contain a loop of the form: *scan forward from `i+1` to `to`, and return the index just before the first heading whose level is `<= maxLevel`, or `to` if none is found.* `collectBlocks` uses `maxLevel = 3` (bounding H3 blocks); `scanDocument` uses `maxLevel = 2` (bounding H2 sections).

**Change:** extract a private helper in `scan.ts`:

```ts
function findBoundaryEnd(classified: ClassifiedLine[], from: number, to: number, maxLevel: number): number {
  for (let j = from; j <= to; j++) {
    const c = classified[j];
    if (c.kind === 'heading' && c.level! <= maxLevel) return j - 1;
  }
  return to;
}
```

- `collectBlocks`'s inner "find end of this H3 block" loop is replaced with `const end = findBoundaryEnd(classified, i + 1, to, 3);`.
- `scanDocument`'s "find end of this H2 section" loop is replaced with `const end = findBoundaryEnd(classified, i + 1, classified.length - 1, 2);`.

**Effect:** removes one level of nesting from each call site (max nesting in the file drops from 4 to 3) and removes the duplicated loop body. `findBoundaryEnd` stays module-private — it is not part of the public API of `scan.ts` and is not separately exported.

**Test changes:** none required. `scan.test.ts`'s existing fixtures (`full-day.md`, `malformed.md`, `subsections.md`) already exercise both the H2-section and H3-block boundary logic and assert on the resulting `DocModel` shape, which is unchanged by this refactor.

---

## 3. `command.ts`: per-`ArgKind` validator map

`validateCommand` currently checks `spec.argKind === 'time' | 'date' | 'theme'` via sequential `if` statements, each independently formatting its own error message. This grows linearly with every new `ArgKind`.

**Change:** replace the `if` chain with a lookup map keyed by `ArgKind`:

```ts
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

**Effect:** `validateCommand`'s cyclomatic complexity drops from 12 to roughly 4 (unknown-command guard, argRequired guard, validator-error guard, success). Adding a future `ArgKind` requires one new map entry instead of another `if` branch, and the `'none'`/`'text'` no-op validators make every `ArgKind` explicitly accounted for (a missing case is a TypeScript error, not a silent pass-through).

**Test changes:** none required. Existing `command.test.ts` cases already cover `time`, `date`, `theme`, `none`, and required/optional arg combinations and assert only on the `ValidationResult` output, which is unchanged.

---

## Files Changed

| File | Change |
|---|---|
| `web/src/lib/doc/command.ts` | Remove duplicate `isValidDate`/`ISO`/`DATE`; import `isValidDate` from `../dates`; replace `if` chain in `validateCommand` with `ARG_VALIDATORS` map |
| `web/src/lib/doc/command.test.ts` | Remove `isValidDate` import and its dedicated `describe` block |
| `web/src/lib/doc/scan.ts` | Add private `findBoundaryEnd` helper; use it from `collectBlocks` and `scanDocument` |

No changes to `web/src/lib/dates.ts`, `web/src/lib/doc/types.ts`, `web/src/lib/doc/scan.test.ts`, `web/src/lib/dates.test.ts`, or any Rust backend code.

---

## Test Coverage / Verification

Since all three changes preserve existing external behavior, verification is regression-based rather than new-feature-based:

- Run the full `web` test suite (`scan.test.ts`, `command.test.ts`, `dates.test.ts`, and the rest) after each change and confirm no regressions.
- `command.test.ts` is edited only to drop the now-redundant `isValidDate` block; no new assertions are needed there since `dates.test.ts` already covers `isValidDate` behavior including the impossible-date case (`2026-02-30`).
- No new test files are introduced. `findBoundaryEnd` and `ARG_VALIDATORS` are internal implementation details already exercised indirectly through `scanDocument`/`collectBlocks` and `validateCommand`'s public tests.
