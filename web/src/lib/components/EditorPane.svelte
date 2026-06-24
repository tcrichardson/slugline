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
