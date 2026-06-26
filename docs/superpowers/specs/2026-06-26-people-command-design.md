# Design: `:people` Command for Meetings and Notes

**Date:** 2026-06-26  
**Status:** Approved

## Summary

Add a `:people` command (with `:p` shortcut) to tag meetings and notes with the people associated with them. Multiple people are supported via comma-separated input. Repeated invocations append to the existing list rather than replacing it.

---

## Command Spec

| Property | Value |
|---|---|
| Command name | `people` |
| Shortcut alias | `p` |
| Argument kind | `text` |
| Argument required | yes |
| Context | `meeting` or `note` block |

**Usage examples:**
```
:people Alice Smith
:people Bob, Carol
:p Dave
```

---

## Alias System

A new `ALIASES` map is added to `web/src/lib/doc/command.ts`:

```typescript
export const ALIASES: Partial<Record<string, CommandName>> = {
  p: 'people',
};
```

`parseCommandLine()` resolves the raw command word through `ALIASES` before looking it up in `COMMANDS`. This means `:p Alice` and `:people Alice` are fully equivalent upstream of validation and dispatch.

`CommandName` grows by one entry (`'people'`), not two. The alias system is intentionally open-ended — future shortcuts can be added by extending `ALIASES` alone.

---

## Context Sensitivity

`:people` works inside **both** `meeting` and `note` blocks. This distinguishes it from `:purpose` (meetings-only) and `:topic` (notes-only).

If the cursor is not inside a meeting or note block, the command returns an error:

> `:people requires a meeting or note block`

---

## Storage Format

A single `meta:people` line in the block's meta region, comma-separated:

```
### Weekly Sync
meta:scheduled 14:30
meta:people Alice Smith, Bob Jones, Carol
```

The `meta:people` line follows the existing meta-region rules: it must appear in the consecutive block of `meta:` lines immediately after the `###` heading. Existing `scanDocument` and `classifyLine` logic handles this without modification.

---

## Append Behavior

Repeated calls to `:people` **append** to the existing list rather than replacing it. This is implemented via a new `appendMeta` helper in `web/src/lib/editor/commands.ts`:

- No existing `meta:people` → creates the line with the provided names
- Existing `meta:people` → trims both sides and joins with `, ` separator

**Example sequence:**

```
:people Alice          →  meta:people Alice
:people Bob, Carol     →  meta:people Alice, Bob, Carol
:p Dave                →  meta:people Alice, Bob, Carol, Dave
```

Duplicate names are **not** deduplicated — the value is free text appended as-is. This is consistent with the field's free-text nature and avoids premature complexity.

---

## Files Changed

| File | Change |
|---|---|
| `web/src/lib/doc/command.ts` | Add `'people'` to `CommandName`; add `people` entry to `COMMANDS`; add `ALIASES` export; update `parseCommandLine` to resolve aliases |
| `web/src/lib/editor/commands.ts` | Add `appendMeta` helper; add `case 'people'` handler in `runCommand` |

No changes required to `types.ts`, `scan.ts`, `classify.ts`, or any Rust backend code.

---

## Test Coverage

- `parseCommandLine('p Alice')` resolves to command `people` with arg `Alice`
- `parseCommandLine('people Alice')` resolves to command `people` with arg `Alice`
- `:people` in a meeting block with no prior value sets `meta:people <names>`
- `:people` in a meeting block with existing value appends `, <names>`
- `:people` in a note block works identically to meeting block
- `:people` outside a meeting or note block returns an error
- `:p` shortcut behaves identically to `:people` in all cases above
