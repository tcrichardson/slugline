<script lang="ts">
  import { app } from '../appState.svelte';
</script>

<nav class="tabs">
  {#each app.tabsState.tabs as date, i (date)}
    <button
      class="tab"
      class:active={i === app.tabsState.activeIndex}
      onclick={() => app.switchTab(i)}
    >
      <span class="label">{date}</span>
      <span
        class="close"
        role="button"
        tabindex="0"
        aria-label="Close tab"
        onclick={(e) => {
          e.stopPropagation();
          app.closeAt(i);
        }}
        onkeydown={(e) => {
          if (e.key === 'Enter') {
            e.stopPropagation();
            app.closeAt(i);
          }
        }}>×</span
      >
    </button>
  {/each}
</nav>

<style>
  .tabs { display: flex; gap: 0.25rem; align-items: flex-end; overflow-x: auto; }
  .tab {
    display: inline-flex; align-items: center; gap: 0.4rem;
    border: none; cursor: pointer; padding: 0.3rem 0.6rem;
    border-radius: 6px 6px 0 0; background: transparent; color: var(--muted);
    font: inherit; font-size: 0.85rem; white-space: nowrap;
  }
  .tab.active { background: var(--bg); color: var(--fg); box-shadow: inset 0 -2px 0 var(--accent); }
  .close { opacity: 0.6; }
  .close:hover { opacity: 1; }
</style>
