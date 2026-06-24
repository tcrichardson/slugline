<script lang="ts">
  import { app } from '../appState.svelte';
</script>

<section class="panel">
  <h2>To Do</h2>
  {#if app.todoGroups.length === 0}
    <p class="empty">No to dos in the last 7 days</p>
  {:else}
    {#each app.todoGroups as group (group.date)}
      <div class="group">
        <h3>{group.date}</h3>
        <ul>
          {#each group.todos as todo (todo.lineIndex)}
            <li class:done={todo.done}>
              <button onclick={() => app.goToDateAndLine(group.date, todo.lineIndex)}>
                <span class="box">{todo.done ? '☑' : '☐'}</span>
                <span class="text">{todo.text}</span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  {/if}
</section>

<style>
  .panel { padding: 0.75rem; border-top: 1px solid var(--status-bar); }
  .panel h2 { margin: 0 0 0.5rem; font-size: 0.9rem; color: var(--heading-2); }
  .empty { color: var(--muted); font-size: 0.8rem; margin: 0; }
  .group h3 { margin: 0.5rem 0 0.25rem; font-size: 0.75rem; color: var(--muted); font-variant-numeric: tabular-nums; }
  ul { list-style: none; margin: 0; padding: 0; }
  li button {
    display: flex; align-items: baseline; gap: 0.4rem; width: 100%;
    border: none; background: transparent; cursor: pointer; text-align: left;
    padding: 0.15rem 0.25rem; border-radius: 4px; color: var(--fg); font: inherit; font-size: 0.85rem;
  }
  li button:hover { background: var(--edit-line-bg); }
  .box { flex-shrink: 0; }
  li.done .text { color: var(--todo-done); text-decoration: line-through; }
</style>
