<script lang="ts">
  import { app } from '../appState.svelte';
  import Tabs from './Tabs.svelte';

  const dateStr = $derived(
    `${app.now.getFullYear()}-${String(app.now.getMonth() + 1).padStart(2, '0')}-${String(
      app.now.getDate(),
    ).padStart(2, '0')}`,
  );
  const timeStr = $derived(
    `${String(app.now.getHours()).padStart(2, '0')}:${String(app.now.getMinutes()).padStart(2, '0')}`,
  );
</script>

<header class="header">
  <div class="brand">
    <div class="name">Slugline</div>
    <div class="clock">
      <span>{dateStr}</span>
      <span class="time">{timeStr}</span>
    </div>
  </div>
  <Tabs />
</header>

<style>
  .header {
    display: flex; align-items: flex-end; gap: 1.5rem;
    padding: 0.5rem 1rem 0; border-bottom: 1px solid var(--status-bar);
    background: var(--status-bar);
  }
  .name { font-weight: 700; font-size: 1.1rem; color: var(--heading-1); }
  .clock { display: flex; flex-direction: column; font-size: 0.8rem; color: var(--muted); line-height: 1.1; }
  .time { font-variant-numeric: tabular-nums; }
</style>
