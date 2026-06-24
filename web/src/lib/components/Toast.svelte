<script lang="ts">
  import { app } from '../appState.svelte';
</script>

{#if app.error}
  <div class="toast" role="alert">
    <span class="msg">{app.error}</span>
    <button class="dismiss" aria-label="Dismiss" onclick={() => app.clearError()}>×</button>
  </div>
{/if}

<style>
  .toast {
    position: fixed; bottom: 2.5rem; left: 50%; transform: translateX(-50%);
    display: flex; align-items: center; gap: 0.75rem; z-index: 50;
    max-width: 90vw; padding: 0.6rem 0.9rem; border-radius: 8px;
    background: #b3261e; color: #fff; font-size: 0.85rem;
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.25);
  }
  .dismiss { border: none; background: transparent; color: #fff; cursor: pointer; font-size: 1.1rem; line-height: 1; }
</style>
