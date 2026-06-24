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
  <div class="name">Slugline</div>
  <Tabs />
  <div class="clock">
    <span>{dateStr}</span>
    <span class="time">{timeStr}</span>
  </div>
</header>

<style>
  .header {
    display: flex; align-items: center; gap: 1.25rem;
    padding: 0.5rem 1rem; background: var(--status-bar);
    border-bottom: 1px solid var(--rule);
  }
  .name { font-weight: 700; font-size: 1.1rem; color: var(--heading-1); letter-spacing: 0.02em; }
  .clock {
    margin-left: auto; display: flex; flex-direction: column;
    text-align: right; font-size: 0.78rem; color: var(--muted); line-height: 1.15;
  }
  .time { font-variant-numeric: tabular-nums; }
</style>
