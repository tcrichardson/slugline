# Markdown Highlighting — Design

**Date:** 2026-06-26

## Goal

Expand the set of markdown constructs rendered on inactive lines in the editor. Three new constructs are added: blockquotes (block-level), strikethrough (inline), and highlight (inline).

## Scope

- **In scope:** single-level blockquotes (`> text`), strikethrough (`~~text~~`), highlight (`==text==`)
- **Out of scope:** nested blockquotes, code fences, horizontal rules, images, any active-line syntax colouring

## Approach

Extend the existing `classify` → `renderInline` → `EditorPane` pipeline. No new files; no new dependencies.

## File Changes

### `web/src/lib/doc/types.ts`

Add `'blockquote'` to the `LineKind` union. No new fields are needed on `ClassifiedLine` because single-level blockquotes carry only `text`.

### `web/src/lib/doc/classify.ts`

Add regex:

```ts
const BLOCKQUOTE = /^>\s?(.*)$/;
```

Checked after the `meta` check, before the list checks. Matching lines emit:

```ts
{ kind: 'blockquote', raw, text: m[1] }
```

### `web/src/lib/doc/renderInline.ts`

Two new passes inserted after the bold/italic block (step 4) and before the code-span restore (step 5):

1. `~~([^~]+)~~` → `<del>$1</del>`
2. `==([^=]+)==` → `<mark>$1</mark>`

Code spans are immune because they have already been replaced with `\u0000N\u0000` placeholders at this point.

### `web/src/lib/components/EditorPane.svelte`

New case in `prettyHtml()`:

```ts
case 'blockquote':
  return `<span class="bq">${renderInline(c.text)}</span>`;
```

New CSS in the `<style>` block:

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

### `web/src/lib/theme.ts`

Two new tokens added to both `LIGHT` and `DARK` maps:

| Token | Light | Dark |
|---|---|---|
| `--blockquote-border` | `#93c5fd` | `#3b82f6` |
| `--highlight-bg` | `#fef08a` | `#713f12` |

### `web/src/app.css`

Same two tokens added to `:root` with the light values (seeds the page before JS theme application).

## Testing

- `classify.test.ts`: cases for `> text`, `>text` (no space), blank blockquote `>`
- `renderInline.test.ts`: cases for `~~strikethrough~~`, `==highlight==`, and combinations with existing bold/italic/code to verify ordering is correct

## Error handling

No new error paths. Unrecognised lines already fall through to `paragraph`; blockquote lines that fail to match (impossible given the regex) would also fall through safely.
