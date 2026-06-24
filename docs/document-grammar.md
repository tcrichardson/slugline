# Slugline Document Grammar (v1)

The **source of truth is the raw Markdown line array.** All structure is *derived* from it.
This document defines the line types and structural rules. Both the TypeScript parser and any
future Rust parser MUST conform to this spec and pass the `fixtures/` corpus.

## Line types (per single line, in priority order)

1. **blank** — the line is empty or whitespace-only.
2. **heading** — `^(#{1,6})\s+(.*)$`. Level = count of `#` (1–6). Text = remainder, trimmed.
3. **task** — `^- \[([ xX])\]\s?(.*)$`. `done` = captured char is `x` or `X`. Text = remainder.
4. **meta** — `^meta:(\S+)(?: (.*))?$`. Key = chars after `meta:` up to first space. Value = remainder after the first space, trimmed (may be empty).
5. **list** — `^\s*[-*+]\s+(.*)$` or `^\s*\d+\.\s+(.*)$`. Text = remainder.
6. **paragraph** — anything else. Text = the raw line.

Task is matched before list (a task is a special list item). Meta is matched before list.

## Document structure

- **Title** = the text of the *first* H1 (`# ...`). Display-only (carries the weekday); never the filename.
- **Sections** = H2 headings. A section spans from its `##` line to the line before the next `##`/`#` (or EOF).
  Recognized section kinds by case-insensitive title: `To Do`/`Todo` → `todo`, `Meetings` → `meetings`,
  `Notes` → `notes`, anything else → `other`.
- **Blocks** = H3 headings (`### Name`) inside a `meetings` or `notes` section. A block spans from its `###`
  line to the line before the next `###`/`##`/`#` (or section end).
- **Meta region** of a block = the run of consecutive `meta:` lines *immediately* after the `###` heading.
  The region ends at the first non-meta line. Keys are unique per block (commands upsert).
  Known keys: meetings → `purpose`, `scheduled`, `started`, `ended`; notes → `topic`. Unknown keys are
  preserved and ignored.
- **Sub-sections** = H4–H6 headings nested inside a block. They do not change which meeting/note you are in;
  context resolution for metadata always walks back to the enclosing H3.

## Times and dates

- Times are 24-hour `HH:MM` (`^([01]\d|2[0-3]):[0-5]\d$`).
- Dates are `YYYY-MM-DD` and must be real calendar dates.

## Tolerance

Parsing is best-effort: malformed or non-canonical documents (missing standard sections, stray content)
must never throw. Missing sections simply yield empty derived data.
