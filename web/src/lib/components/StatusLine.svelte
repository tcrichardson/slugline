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
    border-top: 1px solid var(--rule);
  }
  .mode { font-weight: 700; color: var(--fg); }
  .cmd { color: var(--fg); }
  .msg { margin-left: auto; color: var(--accent); }
</style>
