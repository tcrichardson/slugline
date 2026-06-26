# Markdown Highlighting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add blockquote (block-level), strikethrough, and highlight (inline) markdown rendering to inactive lines in the editor.

**Architecture:** Extend the existing `classify → renderInline → EditorPane` pipeline. `classify.ts` gains a `blockquote` line kind; `renderInline.ts` gains two new inline passes; `EditorPane.svelte` handles the new line kind and adds CSS; two new CSS tokens are added to `theme.ts` and `app.css`.

**Tech Stack:** TypeScript, Svelte 5, Vitest, Vite

---

## File Map

| File | Action | Purpose |
|---|---|---|
| `web/src/lib/doc/types.ts` | Modify | Add `'blockquote'` to `LineKind` union |
| `web/src/lib/doc/classify.ts` | Modify | Recognize `> text` and emit `blockquote` kind |
| `web/src/lib/doc/classify.test.ts` | Modify | Tests for blockquote classification |
| `web/src/lib/doc/renderInline.ts` | Modify | Add `~~s~~` → `<del>`, `==s==` → `<mark>` |
| `web/src/lib/doc/renderInline.test.ts` | Modify | Tests for strikethrough and highlight |
| `web/src/lib/theme.ts` | Modify | Add `--blockquote-border` and `--highlight-bg` tokens |
| `web/src/app.css` | Modify | Seed `:root` with the two new token values |
| `web/src/lib/components/EditorPane.svelte` | Modify | Handle `blockquote` in `prettyHtml()`; add CSS |

---

### Task 1: Classify blockquote lines

**Files:**
- Modify: `web/src/lib/doc/types.ts`
- Modify: `web/src/lib/doc/classify.ts`
- Test: `web/src/lib/doc/classify.test.ts`

- [ ] **Step 1: Write the failing tests**

Open `web/src/lib/doc/classify.test.ts` and append this block before the closing `});` of the `describe`:

```ts
  it('classifies blockquote lines with space after >', () => {
    const b = classifyLine('> some quoted text');
    expect(b.kind).toBe('blockquote');
    expect(b.text).toBe('some quoted text');
  });

  it('classifies blockquote lines without space after >', () => {
    const b = classifyLine('>no space');
    expect(b.kind).toBe('blockquote');
    expect(b.text).toBe('no space');
  });

  it('classifies bare > as blockquote with empty text', () => {
    const b = classifyLine('>');
    expect(b.kind).toBe('blockquote');
    expect(b.text).toBe('');
  });
```

- [ ] **Step 2: Run the tests to confirm they fail**

```bash
cd web && npm test -- --reporter=verbose 2>&1 | grep -E 'blockquote|FAIL|PASS'
```

Expected: three failures mentioning `blockquote`.

- [ ] **Step 3: Add `'blockquote'` to `LineKind` in `types.ts`**

In `web/src/lib/doc/types.ts`, change line 1 from:

```ts
export type LineKind = 'heading' | 'task' | 'list' | 'meta' | 'blank' | 'paragraph';
```

to:

```ts
export type LineKind = 'heading' | 'task' | 'list' | 'meta' | 'blank' | 'paragraph' | 'blockquote';
```

- [ ] **Step 4: Add the blockquote regex and case to `classify.ts`**

In `web/src/lib/doc/classify.ts`, add the regex constant after the existing `OL` line:

```ts
const BLOCKQUOTE = /^>\s?(.*)$/;
```

Then, in the `classifyLine` function, add the blockquote check after the `META` check and before the `UL` check (insert between `if (m) ...` and `const ul = UL.exec`):

```ts
  const bq = BLOCKQUOTE.exec(raw);
  if (bq) return { kind: 'blockquote', raw, text: bq[1] };
```

- [ ] **Step 5: Run the tests to confirm they pass**

```bash
cd web && npm test -- --reporter=verbose 2>&1 | grep -E 'blockquote|FAIL|PASS'
```

Expected: three new tests pass; all existing tests still pass.

- [ ] **Step 6: Commit**

```bash
cd web && git add src/lib/doc/types.ts src/lib/doc/classify.ts src/lib/doc/classify.test.ts && git commit -m "feat(classify): add blockquote line kind"
```

---

### Task 2: Render strikethrough and highlight inline

**Files:**
- Modify: `web/src/lib/doc/renderInline.ts`
- Test: `web/src/lib/doc/renderInline.test.ts`

- [ ] **Step 1: Write the failing tests**

Open `web/src/lib/doc/renderInline.test.ts` and append these cases inside the `describe` block:

```ts
  it('renders strikethrough', () => {
    expect(renderInline('~~deleted~~')).toBe('<del>deleted</del>');
  });

  it('renders highlight', () => {
    expect(renderInline('==marked==')).toBe('<mark>marked</mark>');
  });

  it('renders strikethrough alongside bold without conflict', () => {
    expect(renderInline('**bold** and ~~strike~~')).toBe(
      '<strong>bold</strong> and <del>strike</del>',
    );
  });

  it('does not process strikethrough or highlight inside code spans', () => {
    expect(renderInline('`~~raw~~`')).toBe('<code>~~raw~~</code>');
    expect(renderInline('`==raw==`')).toBe('<code>==raw==</code>');
  });
```

- [ ] **Step 2: Run the tests to confirm they fail**

```bash
cd web && npm test -- --reporter=verbose 2>&1 | grep -E 'strikethrough|highlight|conflict|code spans|FAIL|PASS'
```

Expected: four failures.

- [ ] **Step 3: Add the two new passes to `renderInline.ts`**

In `web/src/lib/doc/renderInline.ts`, after the `_([^_]+)_` italic line (step 4) and before the `// 5. Restore code spans` comment, insert:

```ts
  // 5a. Strikethrough.
  s = s.replace(/~~([^~]+)~~/g, '<del>$1</del>');

  // 5b. Highlight.
  s = s.replace(/==([^=]+)==/g, '<mark>$1</mark>');
```

Renumber the existing `// 5. Restore code spans.` comment to `// 6. Restore code spans.`.

- [ ] **Step 4: Run the tests to confirm they pass**

```bash
cd web && npm test -- --reporter=verbose 2>&1 | grep -E 'strikethrough|highlight|conflict|code spans|FAIL|PASS'
```

Expected: all four new tests pass; all existing tests still pass.

- [ ] **Step 5: Commit**

```bash
cd web && git add src/lib/doc/renderInline.ts src/lib/doc/renderInline.test.ts && git commit -m "feat(renderInline): add strikethrough and highlight inline markup"
```

---

### Task 3: Add theme tokens for blockquote border and highlight background

**Files:**
- Modify: `web/src/lib/theme.ts`
- Modify: `web/src/app.css`

No unit tests — these are CSS variable values applied to the DOM.

- [ ] **Step 1: Add tokens to `theme.ts`**

In `web/src/lib/theme.ts`, in the `LIGHT` object, add after `'--cursor': '#1b2330',`:

```ts
  '--blockquote-border': '#93c5fd',
  '--highlight-bg': '#fef08a',
```

In the `DARK` object, add after `'--cursor': '#e7ecf5',`:

```ts
  '--blockquote-border': '#3b82f6',
  '--highlight-bg': '#713f12',
```

- [ ] **Step 2: Seed `:root` in `app.css`**

In `web/src/app.css`, inside the `:root` block, add after `--cursor: #1b2330;`:

```css
  --blockquote-border: #93c5fd;
  --highlight-bg: #fef08a;
```

- [ ] **Step 3: Run the full test suite to confirm nothing broke**

```bash
cd web && npm test
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
cd web && git add src/lib/theme.ts src/app.css && git commit -m "feat(theme): add blockquote-border and highlight-bg tokens"
```

---

### Task 4: Wire blockquote into EditorPane

**Files:**
- Modify: `web/src/lib/components/EditorPane.svelte`

No unit test — this is a visual Svelte component; correctness is verified by building and inspecting.

- [ ] **Step 1: Add the `blockquote` case to `prettyHtml()`**

In `web/src/lib/components/EditorPane.svelte`, in the `prettyHtml` function, add a new case after the `'list'` case and before the `default`:

```ts
      case 'blockquote':
        return `<span class="bq">${renderInline(c.text)}</span>`;
```

- [ ] **Step 2: Add CSS for `.bq`, `del`, and `mark`**

In the `<style>` block of `EditorPane.svelte`, append before the closing `</style>`:

```css
  :global(.bq) {
    display: block;
    border-left: 3px solid var(--blockquote-border);
    padding-left: 0.75rem;
    color: var(--muted);
    font-style: italic;
  }
  :global(del) { color: var(--muted); text-decoration: line-through; }
  :global(mark) {
    background: var(--highlight-bg);
    color: inherit;
    border-radius: 2px;
    padding: 0 2px;
  }
```

- [ ] **Step 3: Build to verify no TypeScript/Svelte errors**

```bash
cd web && npm run check
```

Expected: no errors.

- [ ] **Step 4: Run the full test suite**

```bash
cd web && npm test
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
cd web && git add src/lib/components/EditorPane.svelte && git commit -m "feat(editor): render blockquote lines; add del and mark styles"
```
