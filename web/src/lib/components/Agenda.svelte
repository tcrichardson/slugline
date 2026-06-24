<script lang="ts">
  import { app } from '../appState.svelte';
  import { deriveAgenda } from '../agenda';

  const items = $derived(deriveAgenda(app.editor.lines));
</script>

<section class="panel">
  <h2>Agenda</h2>
  {#if items.length === 0}
    <p class="empty">No scheduled meetings</p>
  {:else}
    <ul>
      {#each items as item (item.headingLineIndex)}
        <li class:done={!!item.ended}>
          <button onclick={() => app.jumpToLine(item.headingLineIndex)}>
            <span class="time">{item.time}</span>
            <span class="name">{item.name}</span>
            {#if item.ended}<span class="badge" title="Ended {item.ended}">✓</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .panel { padding: 0.75rem; border-top: 1px solid var(--status-bar); }
  .panel h2 { margin: 0 0 0.5rem; font-size: 0.9rem; color: var(--heading-2); }
  .empty { color: var(--muted); font-size: 0.8rem; margin: 0; }
  ul { list-style: none; margin: 0; padding: 0; }
  li button {
    display: flex; align-items: baseline; gap: 0.5rem; width: 100%;
    border: none; background: transparent; cursor: pointer; text-align: left;
    padding: 0.2rem 0.25rem; border-radius: 4px; color: var(--fg); font: inherit; font-size: 0.85rem;
  }
  li button:hover { background: var(--edit-line-bg); }
  .time { font-variant-numeric: tabular-nums; color: var(--accent); flex-shrink: 0; }
  li.done .name { color: var(--todo-done); text-decoration: line-through; }
  .badge { margin-left: auto; color: var(--todo-done); }
</style>
