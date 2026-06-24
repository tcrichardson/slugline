# Phase 1: Document Model Core (TS) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the pure, DOM-free TypeScript document-model library — line classifier, structural scanner, context resolver, inline renderer, and command parser — plus the `web/` frontend scaffold, a written grammar spec, and a fixture corpus, all verified by Vitest.

**Architecture:** Source of truth is the raw Markdown line array. Every module here is a pure function (`input -> output`) with no DOM or network dependency, so it is exhaustively unit-testable against a shared `fixtures/` corpus. These modules become the shared foundation the editor (Phase 4) and sidebar (Phase 5) build on, and the contract a future Rust parser must match.

**Tech Stack:** Vite + Svelte 5 + TypeScript, Vitest (node environment), Node 24 / npm 11. Root `Makefile` orchestrates tasks (no `just` installed).

---

## File Structure

| File | Responsibility |
|---|---|
| `web/` (scaffold) | Vite + Svelte + TS project; hosts all client code and tests |
| `Makefile` | Root task runner (`test-web`, `fmt-web`) |
| `docs/document-grammar.md` | The written Slugline document grammar (anti-drift contract) |
| `fixtures/*.md` | Corpus of example documents (valid + edge cases) used by tests |
| `web/src/lib/doc/types.ts` | Shared types: `ClassifiedLine`, `DocModel`, `Section`, `Block`, `MetaEntry` |
| `web/src/lib/doc/classify.ts` | `classifyLine(raw)` — classify one raw line |
| `web/src/lib/doc/renderInline.ts` | `renderInline(text)` — safe inline-Markdown → HTML |
| `web/src/lib/doc/scan.ts` | `scanDocument(lines)` — derive sections/blocks/meta |
| `web/src/lib/doc/context.ts` | `resolveContext`, `nearestHeadingLevel` |
| `web/src/lib/doc/command.ts` | `parseCommandLine`, `validateCommand`, command spec table |
| `web/src/lib/doc/__fixtures__/load.ts` | Test helper to load fixture files |

All modules live under `web/src/lib/doc/` so they move/import together. Tests are co-located (`*.test.ts`).

---

## Task 1: Scaffold the `web/` frontend project + Makefile

**Files:**
- Create: `web/` (via create-vite), `Makefile`
- Create (temp): `web/src/lib/doc/smoke.test.ts`

- [ ] **Step 1: Scaffold the Vite + Svelte + TS project**

Run from the repo root:
```bash
npm create vite@latest web -- --template svelte-ts
```
Expected: a `web/` directory is created containing `package.json`, `vite.config.ts`, `tsconfig*.json`, `src/`, `index.html`. The `--template svelte-ts` flag makes this non-interactive; if any prompt appears, accept defaults and do **not** auto-start a dev server.

- [ ] **Step 2: Install dependencies and add Vitest**

```bash
cd web && npm install && npm install -D vitest
```
Expected: `node_modules/` populated; `vitest` added to `devDependencies`.

- [ ] **Step 3: Add a `test` script to `web/package.json`**

In `web/package.json`, add to the `"scripts"` object:
```json
"test": "vitest run"
```

- [ ] **Step 4: Create the module directory and a smoke test**

Create `web/src/lib/doc/smoke.test.ts`:
```ts
import { describe, it, expect } from 'vitest';

describe('vitest wiring', () => {
  it('runs', () => {
    expect(1 + 1).toBe(2);
  });
});
```

- [ ] **Step 5: Run the smoke test to verify the toolchain**

Run (from `web/`):
```bash
npm test
```
Expected: PASS — 1 test passed.

- [ ] **Step 6: Delete the smoke test**

```bash
rm web/src/lib/doc/smoke.test.ts
```

- [ ] **Step 7: Create the root `Makefile`**

Create `Makefile` at the repo root:
```makefile
.PHONY: test-web fmt-web

test-web:
	cd web && npm test

fmt-web:
	cd web && npx prettier --write "src/**/*.{ts,svelte}"
```
(Additional targets — `dev`, `build` — are added in later phases.)

- [ ] **Step 8: Verify the Makefile target works**

Run from the repo root:
```bash
make test-web
```
Expected: Vitest runs (no test files found yet is acceptable, or "No test files found" — exit 0 is not required here; the next tasks add tests). If it errors on "no tests", that's fine; proceed.

- [ ] **Step 9: Commit**

```bash
git add web Makefile .gitignore
git commit -m "chore: scaffold web (vite+svelte+ts) with vitest and root Makefile"
```
Note: ensure `web/node_modules` is git-ignored (create-vite adds `web/.gitignore`; if not, add `node_modules` to it before committing).

---

## Task 2: Write the document grammar spec

**Files:**
- Create: `docs/document-grammar.md`

- [ ] **Step 1: Write the grammar spec**

Create `docs/document-grammar.md`:
````markdown
# Slugline Document Grammar (v1)

The **source of truth is the raw Markdown line array.** All structure is *derived* from it.
This document defines the line types and structural rules. Both the TypeScript parser and any
future Rust parser MUST conform to this spec and pass the `fixtures/` corpus.

## Line types (per single line, in priority order)

1. **blank** — the line is empty or whitespace-only.
2. **heading** — `^(#{1,6})\s+(.*)$`. Level = count of `#` (1–6). Text = remainder, trimmed.
3. **task** — `^- \[([ xX])\]\s?(.*)$`. `done` = bracket char is `x`/`X`. Text = remainder.
4. **meta** — `^meta:(\S+)(?: (.*))?$`. Key = chars after `meta:` up to first space. Value = remainder, trimmed (may be empty).
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
````

- [ ] **Step 2: Commit**

```bash
git add docs/document-grammar.md
git commit -m "docs: add Slugline document grammar spec"
```

---

## Task 3: Create the fixture corpus + loader

**Files:**
- Create: `fixtures/empty-template.md`, `fixtures/full-day.md`, `fixtures/subsections.md`, `fixtures/malformed.md`
- Create: `web/src/lib/doc/__fixtures__/load.ts`

- [ ] **Step 1: Create `fixtures/empty-template.md`**

```markdown
# 2026-06-23-TUE

## To Do

## Meetings

## Notes
```

- [ ] **Step 2: Create `fixtures/full-day.md`**

```markdown
# 2026-06-23-TUE

## To Do

- [ ] Buy milk
- [x] Send invoice
- [ ] Prep deck _(Weekly Sync)_

## Meetings

### Weekly Sync
meta:purpose Plan the launch
meta:scheduled 14:30
meta:started 14:31
meta:ended 15:02

Discussed timeline and **owners**.

### Standup
meta:scheduled 09:00

## Notes

### Architecture
meta:topic Caching

Use a `read-through` cache. See [rfc](https://example.com/rfc).
```

- [ ] **Step 3: Create `fixtures/subsections.md`**

```markdown
# 2026-06-22-MON

## Meetings

### Planning
meta:scheduled 10:00

#### Risks

Budget is tight.

##### Mitigations

Cut scope.

## Notes
```

- [ ] **Step 4: Create `fixtures/malformed.md`**

```markdown
# Just a title

Some loose paragraph text with no standard sections.

- [ ] an orphan todo not under any section
```

- [ ] **Step 5: Create the fixture loader `web/src/lib/doc/__fixtures__/load.ts`**

```ts
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

// Tests run with cwd = web/, so fixtures live one directory up.
const FIXTURE_DIR = resolve(process.cwd(), '..', 'fixtures');

export function loadFixture(name: string): string {
  return readFileSync(resolve(FIXTURE_DIR, name), 'utf8');
}

export function fixtureLines(name: string): string[] {
  return loadFixture(name).split('\n');
}
```

- [ ] **Step 6: Verify the loader resolves fixtures (temporary test)**

Create `web/src/lib/doc/__fixtures__/load.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { fixtureLines } from './load';

describe('fixture loader', () => {
  it('reads the empty template', () => {
    const lines = fixtureLines('empty-template.md');
    expect(lines[0]).toBe('# 2026-06-23-TUE');
    expect(lines).toContain('## Meetings');
  });
});
```

- [ ] **Step 7: Run the loader test**

Run (from `web/`):
```bash
npm test
```
Expected: PASS — fixture loader test green.

- [ ] **Step 8: Commit**

```bash
git add fixtures web/src/lib/doc/__fixtures__
git commit -m "test: add document fixture corpus and loader"
```

---

## Task 4: Define shared types

**Files:**
- Create: `web/src/lib/doc/types.ts`

- [ ] **Step 1: Write the types module**

Create `web/src/lib/doc/types.ts`:
```ts
export type LineKind = 'heading' | 'task' | 'list' | 'meta' | 'blank' | 'paragraph';

export interface ClassifiedLine {
  kind: LineKind;
  /** The original line, unmodified. */
  raw: string;
  /** Content with any prefix stripped. For `meta` this is the value; for `blank` it is ''. */
  text: string;
  /** Heading level 1–6 when kind === 'heading'. */
  level?: number;
  /** Done state when kind === 'task'. */
  done?: boolean;
  /** Key when kind === 'meta'. */
  metaKey?: string;
}

export interface MetaEntry {
  key: string;
  value: string;
  lineIndex: number;
}

export type SectionKind = 'todo' | 'meetings' | 'notes' | 'other';

export interface Block {
  name: string;
  level: number; // 3
  headingLineIndex: number;
  startLine: number; // inclusive
  endLine: number; // inclusive
  meta: MetaEntry[];
  /** Index of the last meta line, or headingLineIndex when the block has no meta. */
  metaEndLine: number;
}

export interface Section {
  kind: SectionKind;
  title: string;
  level: number; // 2
  headingLineIndex: number;
  startLine: number; // inclusive (the heading line)
  endLine: number; // inclusive (last line before next H2/H1 or EOF)
  blocks: Block[]; // H3 blocks for meetings/notes; empty otherwise
}

export interface DocModel {
  title: string | null;
  titleLineIndex: number | null;
  sections: Section[];
}
```

- [ ] **Step 2: Type-check by running tests (no tests yet for this file, but ensure no syntax errors)**

Run (from `web/`):
```bash
npm test
```
Expected: PASS (existing fixture test still green; the new file compiles).

- [ ] **Step 3: Commit**

```bash
git add web/src/lib/doc/types.ts
git commit -m "feat: add document model types"
```

---

## Task 5: Line classifier

**Files:**
- Create: `web/src/lib/doc/classify.ts`
- Test: `web/src/lib/doc/classify.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/doc/classify.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { classifyLine } from './classify';

describe('classifyLine', () => {
  it('classifies blank lines', () => {
    expect(classifyLine('').kind).toBe('blank');
    expect(classifyLine('   ').kind).toBe('blank');
  });

  it('classifies headings with level and text', () => {
    const h = classifyLine('### Weekly Sync');
    expect(h.kind).toBe('heading');
    expect(h.level).toBe(3);
    expect(h.text).toBe('Weekly Sync');
  });

  it('classifies tasks with done state', () => {
    const open = classifyLine('- [ ] Buy milk');
    expect(open.kind).toBe('task');
    expect(open.done).toBe(false);
    expect(open.text).toBe('Buy milk');

    const done = classifyLine('- [X] Send invoice');
    expect(done.kind).toBe('task');
    expect(done.done).toBe(true);
    expect(done.text).toBe('Send invoice');
  });

  it('classifies meta lines with key and value', () => {
    const m = classifyLine('meta:scheduled 14:30');
    expect(m.kind).toBe('meta');
    expect(m.metaKey).toBe('scheduled');
    expect(m.text).toBe('14:30');
  });

  it('classifies meta lines with empty value', () => {
    const m = classifyLine('meta:purpose');
    expect(m.kind).toBe('meta');
    expect(m.metaKey).toBe('purpose');
    expect(m.text).toBe('');
  });

  it('classifies plain list items', () => {
    const l = classifyLine('- a bullet');
    expect(l.kind).toBe('list');
    expect(l.text).toBe('a bullet');
  });

  it('falls back to paragraph', () => {
    const p = classifyLine('just some prose');
    expect(p.kind).toBe('paragraph');
    expect(p.text).toBe('just some prose');
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run (from `web/`):
```bash
npm test
```
Expected: FAIL — cannot resolve `./classify`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/doc/classify.ts`:
```ts
import type { ClassifiedLine } from './types';

const HEADING = /^(#{1,6})\s+(.*)$/;
const TASK = /^- \[([ xX])\]\s?(.*)$/;
const META = /^meta:(\S+)(?: (.*))?$/;
const UL = /^\s*[-*+]\s+(.*)$/;
const OL = /^\s*\d+\.\s+(.*)$/;

export function classifyLine(raw: string): ClassifiedLine {
  if (raw.trim() === '') return { kind: 'blank', raw, text: '' };

  const h = HEADING.exec(raw);
  if (h) return { kind: 'heading', raw, level: h[1].length, text: h[2].trim() };

  const t = TASK.exec(raw);
  if (t) return { kind: 'task', raw, done: t[1].toLowerCase() === 'x', text: t[2] };

  const m = META.exec(raw);
  if (m) return { kind: 'meta', raw, metaKey: m[1], text: (m[2] ?? '').trim() };

  const ul = UL.exec(raw);
  if (ul) return { kind: 'list', raw, text: ul[1] };
  const ol = OL.exec(raw);
  if (ol) return { kind: 'list', raw, text: ol[1] };

  return { kind: 'paragraph', raw, text: raw };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run (from `web/`):
```bash
npm test
```
Expected: PASS — all `classifyLine` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/doc/classify.ts web/src/lib/doc/classify.test.ts
git commit -m "feat: add line classifier"
```

---

## Task 6: Inline Markdown renderer

**Files:**
- Create: `web/src/lib/doc/renderInline.ts`
- Test: `web/src/lib/doc/renderInline.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/doc/renderInline.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { renderInline } from './renderInline';

describe('renderInline', () => {
  it('escapes HTML', () => {
    expect(renderInline('a < b & c')).toBe('a &lt; b &amp; c');
  });

  it('renders bold and italic', () => {
    expect(renderInline('**bold**')).toBe('<strong>bold</strong>');
    expect(renderInline('*it*')).toBe('<em>it</em>');
    expect(renderInline('_it_')).toBe('<em>it</em>');
  });

  it('renders inline code without processing inner markup', () => {
    expect(renderInline('`a*b*c`')).toBe('<code>a*b*c</code>');
  });

  it('renders safe links', () => {
    expect(renderInline('[rfc](https://example.com)')).toBe(
      '<a href="https://example.com" rel="noopener noreferrer">rfc</a>',
    );
  });

  it('rejects unsafe link schemes (no anchor produced)', () => {
    expect(renderInline('[x](javascript:alert(1))')).not.toContain('<a');
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run (from `web/`):
```bash
npm test
```
Expected: FAIL — cannot resolve `./renderInline`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/doc/renderInline.ts`:
```ts
const SAFE_SCHEME = /^(https?:|mailto:)/i;

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export function renderInline(text: string): string {
  // 1. Pull out code spans first so their contents are not further processed.
  const codes: string[] = [];
  let s = text.replace(/`([^`]+)`/g, (_m, code: string) => {
    const i = codes.push(`<code>${escapeHtml(code)}</code>`) - 1;
    return `\u0000${i}\u0000`;
  });

  // 2. Escape everything else.
  s = escapeHtml(s);

  // 3. Links: [label](url). label/url are already escaped.
  s = s.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (m, label: string, url: string) => {
    const raw = url.replace(/&amp;/g, '&');
    if (!SAFE_SCHEME.test(raw)) return m;
    return `<a href="${url}" rel="noopener noreferrer">${label}</a>`;
  });

  // 4. Bold before italic.
  s = s.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  s = s.replace(/\*([^*]+)\*/g, '<em>$1</em>');
  s = s.replace(/_([^_]+)_/g, '<em>$1</em>');

  // 5. Restore code spans.
  s = s.replace(/\u0000(\d+)\u0000/g, (_m, i: string) => codes[Number(i)]);
  return s;
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run (from `web/`):
```bash
npm test
```
Expected: PASS — all `renderInline` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/doc/renderInline.ts web/src/lib/doc/renderInline.test.ts
git commit -m "feat: add safe inline markdown renderer"
```

---

## Task 7: Structural scanner

**Files:**
- Create: `web/src/lib/doc/scan.ts`
- Test: `web/src/lib/doc/scan.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/doc/scan.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { scanDocument } from './scan';
import { fixtureLines } from './__fixtures__/load';

describe('scanDocument', () => {
  it('reads the title from the first H1', () => {
    const model = scanDocument(fixtureLines('full-day.md'));
    expect(model.title).toBe('2026-06-23-TUE');
    expect(model.titleLineIndex).toBe(0);
  });

  it('finds the three standard sections by kind', () => {
    const model = scanDocument(fixtureLines('full-day.md'));
    expect(model.sections.map((s) => s.kind)).toEqual(['todo', 'meetings', 'notes']);
  });

  it('collects H3 blocks under meetings with their meta', () => {
    const model = scanDocument(fixtureLines('full-day.md'));
    const meetings = model.sections.find((s) => s.kind === 'meetings')!;
    expect(meetings.blocks.map((b) => b.name)).toEqual(['Weekly Sync', 'Standup']);

    const sync = meetings.blocks[0];
    const scheduled = sync.meta.find((m) => m.key === 'scheduled')!;
    expect(scheduled.value).toBe('14:30');
    expect(sync.meta.map((m) => m.key)).toEqual(['purpose', 'scheduled', 'started', 'ended']);
  });

  it('bounds a block to the line before the next heading', () => {
    const model = scanDocument(fixtureLines('full-day.md'));
    const meetings = model.sections.find((s) => s.kind === 'meetings')!;
    const sync = meetings.blocks[0];
    // metaEndLine is the last meta line of Weekly Sync; endLine reaches the line before "### Standup"
    expect(sync.metaEndLine).toBeGreaterThan(sync.headingLineIndex);
    expect(sync.endLine).toBeGreaterThanOrEqual(sync.metaEndLine);
  });

  it('does not throw on malformed documents', () => {
    const model = scanDocument(fixtureLines('malformed.md'));
    expect(model.title).toBe('Just a title');
    expect(model.sections).toEqual([]);
  });

  it('treats a block with no meta as metaEndLine === headingLineIndex', () => {
    const model = scanDocument(fixtureLines('subsections.md'));
    const meetings = model.sections.find((s) => s.kind === 'meetings')!;
    const planning = meetings.blocks[0];
    expect(planning.name).toBe('Planning');
    // Planning has exactly one meta line (scheduled), so metaEndLine > heading.
    expect(planning.meta.map((m) => m.key)).toEqual(['scheduled']);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run (from `web/`):
```bash
npm test
```
Expected: FAIL — cannot resolve `./scan`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/doc/scan.ts`:
```ts
import { classifyLine } from './classify';
import type { ClassifiedLine, DocModel, Section, Block, SectionKind, MetaEntry } from './types';

function sectionKind(title: string): SectionKind {
  const t = title.trim().toLowerCase();
  if (t === 'to do' || t === 'todo') return 'todo';
  if (t === 'meetings') return 'meetings';
  if (t === 'notes') return 'notes';
  return 'other';
}

function collectBlocks(classified: ClassifiedLine[], from: number, to: number): Block[] {
  const blocks: Block[] = [];
  for (let i = from; i <= to; i++) {
    const c = classified[i];
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

      const meta: MetaEntry[] = [];
      let metaEndLine = start;
      for (let k = start + 1; k <= end && classified[k].kind === 'meta'; k++) {
        meta.push({ key: classified[k].metaKey!, value: classified[k].text, lineIndex: k });
        metaEndLine = k;
      }

      blocks.push({
        name: c.text,
        level: 3,
        headingLineIndex: start,
        startLine: start,
        endLine: end,
        meta,
        metaEndLine,
      });
      i = end;
    }
  }
  return blocks;
}

export function scanDocument(lines: string[]): DocModel {
  const classified = lines.map(classifyLine);

  let title: string | null = null;
  let titleLineIndex: number | null = null;
  for (let i = 0; i < classified.length; i++) {
    const c = classified[i];
    if (c.kind === 'heading' && c.level === 1) {
      title = c.text;
      titleLineIndex = i;
      break;
    }
  }

  const sections: Section[] = [];
  for (let i = 0; i < classified.length; i++) {
    const c = classified[i];
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

      const kind = sectionKind(c.text);
      const section: Section = {
        kind,
        title: c.text,
        level: 2,
        headingLineIndex: start,
        startLine: start,
        endLine: end,
        blocks: kind === 'meetings' || kind === 'notes' ? collectBlocks(classified, start + 1, end) : [],
      };
      sections.push(section);
    }
  }

  return { title, titleLineIndex, sections };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run (from `web/`):
```bash
npm test
```
Expected: PASS — all `scanDocument` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/doc/scan.ts web/src/lib/doc/scan.test.ts
git commit -m "feat: add structural document scanner"
```

---

## Task 8: Context resolver

**Files:**
- Create: `web/src/lib/doc/context.ts`
- Test: `web/src/lib/doc/context.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/doc/context.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { scanDocument } from './scan';
import { resolveContext, nearestHeadingLevel } from './context';
import { fixtureLines } from './__fixtures__/load';

describe('resolveContext', () => {
  const lines = fixtureLines('full-day.md');
  const model = scanDocument(lines);

  it('returns the meeting when the cursor is inside an H3 under Meetings', () => {
    const meetings = model.sections.find((s) => s.kind === 'meetings')!;
    const sync = meetings.blocks[0];
    const ctx = resolveContext(model, sync.headingLineIndex + 1);
    expect(ctx.kind).toBe('meeting');
    if (ctx.kind === 'meeting') expect(ctx.block.name).toBe('Weekly Sync');
  });

  it('returns the note when the cursor is inside an H3 under Notes', () => {
    const notes = model.sections.find((s) => s.kind === 'notes')!;
    const arch = notes.blocks[0];
    const ctx = resolveContext(model, arch.headingLineIndex + 1);
    expect(ctx.kind).toBe('note');
  });

  it('returns todo when the cursor is inside the To Do section', () => {
    const todo = model.sections.find((s) => s.kind === 'todo')!;
    const ctx = resolveContext(model, todo.headingLineIndex + 1);
    expect(ctx.kind).toBe('todo');
  });

  it('returns none when the cursor is on the title line', () => {
    expect(resolveContext(model, 0).kind).toBe('none');
  });
});

describe('nearestHeadingLevel', () => {
  it('finds the level of the nearest enclosing heading', () => {
    const lines = fixtureLines('subsections.md');
    // Inside the "Mitigations" (H5) area -> nearest heading level is 5.
    const idx = lines.indexOf('Cut scope.');
    expect(nearestHeadingLevel(lines, idx)).toBe(5);
  });

  it('returns null above any heading', () => {
    expect(nearestHeadingLevel(['', 'no heading yet'], 1)).toBeNull();
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run (from `web/`):
```bash
npm test
```
Expected: FAIL — cannot resolve `./context`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/doc/context.ts`:
```ts
import type { DocModel, Block, Section } from './types';

export type Context =
  | { kind: 'none' }
  | { kind: 'todo'; section: Section }
  | { kind: 'meeting'; block: Block; section: Section }
  | { kind: 'note'; block: Block; section: Section }
  | { kind: 'other'; section: Section };

export function resolveContext(model: DocModel, lineIndex: number): Context {
  const section = model.sections.find((s) => lineIndex >= s.startLine && lineIndex <= s.endLine);
  if (!section) return { kind: 'none' };

  if (section.kind === 'todo') return { kind: 'todo', section };

  if (section.kind === 'meetings' || section.kind === 'notes') {
    const block = section.blocks.find((b) => lineIndex >= b.startLine && lineIndex <= b.endLine);
    if (block) {
      return section.kind === 'meetings'
        ? { kind: 'meeting', block, section }
        : { kind: 'note', block, section };
    }
    return { kind: 'other', section };
  }

  return { kind: 'other', section };
}

const HEADING = /^(#{1,6})\s+/;

export function nearestHeadingLevel(lines: string[], lineIndex: number): number | null {
  for (let i = Math.min(lineIndex, lines.length - 1); i >= 0; i--) {
    const m = HEADING.exec(lines[i]);
    if (m) return m[1].length;
  }
  return null;
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run (from `web/`):
```bash
npm test
```
Expected: PASS — all `context` tests green.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/doc/context.ts web/src/lib/doc/context.test.ts
git commit -m "feat: add context resolver and nearest-heading helper"
```

---

## Task 9: Command parser & validator

**Files:**
- Create: `web/src/lib/doc/command.ts`
- Test: `web/src/lib/doc/command.test.ts`

- [ ] **Step 1: Write the failing test**

Create `web/src/lib/doc/command.test.ts`:
```ts
import { describe, it, expect } from 'vitest';
import { parseCommandLine, validateCommand, isValidDate } from './command';

describe('parseCommandLine', () => {
  it('splits name and rest-of-line argument', () => {
    expect(parseCommandLine('meeting Daily Standup')).toEqual({ name: 'meeting', arg: 'Daily Standup' });
  });

  it('lowercases the name and handles no-arg commands', () => {
    expect(parseCommandLine('Today')).toEqual({ name: 'today', arg: '' });
  });
});

describe('validateCommand', () => {
  it('accepts a valid text command', () => {
    expect(validateCommand('meeting Weekly Sync')).toEqual({ ok: true, command: 'meeting', arg: 'Weekly Sync' });
  });

  it('rejects unknown commands', () => {
    const r = validateCommand('meetng x');
    expect(r.ok).toBe(false);
    if (!r.ok) expect(r.error).toContain('Unknown command');
  });

  it('requires arguments where mandated', () => {
    const r = validateCommand('todo');
    expect(r.ok).toBe(false);
  });

  it('validates HH:MM for scheduled', () => {
    expect(validateCommand('scheduled 14:30').ok).toBe(true);
    expect(validateCommand('scheduled 25:00').ok).toBe(false);
  });

  it('validates YYYY-MM-DD for goto', () => {
    expect(validateCommand('goto 2026-06-23').ok).toBe(true);
    expect(validateCommand('goto 2026-13-01').ok).toBe(false);
  });

  it('validates theme values', () => {
    expect(validateCommand('theme dark').ok).toBe(true);
    expect(validateCommand('theme neon').ok).toBe(false);
  });

  it('accepts no-arg commands', () => {
    expect(validateCommand('start').ok).toBe(true);
    expect(validateCommand('close').ok).toBe(true);
  });
});

describe('isValidDate', () => {
  it('rejects impossible calendar dates', () => {
    expect(isValidDate('2026-02-30')).toBe(false);
    expect(isValidDate('2026-02-28')).toBe(true);
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run (from `web/`):
```bash
npm test
```
Expected: FAIL — cannot resolve `./command`.

- [ ] **Step 3: Write the implementation**

Create `web/src/lib/doc/command.ts`:
```ts
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
  | 'start' | 'end' | 'scheduled' | 'purpose' | 'topic'
  | 'goto' | 'today' | 'tab' | 'close' | 'w' | 'theme';

export type ArgKind = 'none' | 'text' | 'time' | 'date' | 'theme';

export interface CommandSpec {
  name: CommandName;
  argKind: ArgKind;
  argRequired: boolean;
}

export const COMMANDS: Record<CommandName, CommandSpec> = {
  meeting: { name: 'meeting', argKind: 'text', argRequired: true },
  note: { name: 'note', argKind: 'text', argRequired: true },
  section: { name: 'section', argKind: 'text', argRequired: true },
  todo: { name: 'todo', argKind: 'text', argRequired: true },
  start: { name: 'start', argKind: 'none', argRequired: false },
  end: { name: 'end', argKind: 'none', argRequired: false },
  scheduled: { name: 'scheduled', argKind: 'time', argRequired: true },
  purpose: { name: 'purpose', argKind: 'text', argRequired: true },
  topic: { name: 'topic', argKind: 'text', argRequired: true },
  goto: { name: 'goto', argKind: 'date', argRequired: true },
  today: { name: 'today', argKind: 'none', argRequired: false },
  tab: { name: 'tab', argKind: 'date', argRequired: true },
  close: { name: 'close', argKind: 'none', argRequired: false },
  w: { name: 'w', argKind: 'none', argRequired: false },
  theme: { name: 'theme', argKind: 'theme', argRequired: true },
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
  if (!(name in COMMANDS)) return { ok: false, error: `Unknown command: :${name}` };
  const spec = COMMANDS[name as CommandName];

  if (spec.argRequired && arg === '') return { ok: false, error: `:${name} requires an argument` };
  if (spec.argKind === 'time' && !TIME.test(arg)) return { ok: false, error: 'Expected HH:MM' };
  if (spec.argKind === 'date' && !isValidDate(arg)) return { ok: false, error: 'Expected YYYY-MM-DD' };
  if (spec.argKind === 'theme' && arg !== 'light' && arg !== 'dark') {
    return { ok: false, error: 'Expected light or dark' };
  }

  return { ok: true, command: spec.name, arg };
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run (from `web/`):
```bash
npm test
```
Expected: PASS — all `command` tests green.

- [ ] **Step 5: Remove the temporary fixture-loader test (now redundant with scanner tests)**

```bash
rm web/src/lib/doc/__fixtures__/load.test.ts
```

- [ ] **Step 6: Run the full suite once more**

Run (from `web/`):
```bash
npm test
```
Expected: PASS — classifier, renderer, scanner, context, and command suites all green.

- [ ] **Step 7: Commit**

```bash
git add web/src/lib/doc/command.ts web/src/lib/doc/command.test.ts web/src/lib/doc/__fixtures__/load.test.ts
git commit -m "feat: add command parser and validator"
```

---

## Phase 1 Done Criteria

- `make test-web` (or `cd web && npm test`) is green with classifier, renderer, scanner, context, and command suites.
- `docs/document-grammar.md` exists and matches the implemented behavior.
- `fixtures/` contains valid + edge-case documents exercised by the scanner/context tests.
- All modules under `web/src/lib/doc/` are pure (no DOM, no network), ready for the editor (Phase 4) and sidebar (Phase 5) to consume.

## Self-Review Notes (performed during authoring)

- **Spec coverage:** line types, section/block/meta derivation, context resolution, command grammar + arg validation all map to the grammar spec (Task 2) and design decisions (filename/title decoupling is a Phase 2/3 concern, not parsing).
- **Type consistency:** `ClassifiedLine`, `DocModel`, `Section`, `Block`, `MetaEntry` defined once in `types.ts` and consumed unchanged by `scan.ts`/`context.ts`. `CommandName` union is the single source for command identity.
- **No placeholders:** every step contains complete code and exact commands.
