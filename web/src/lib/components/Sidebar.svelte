<script lang="ts">
  import Calendar from './Calendar.svelte';
</script>

<aside class="sidebar">
  <Calendar />
  <section class="panel">
    <h2>Agenda</h2>
    <p class="empty">Coming in Phase 5</p>
  </section>
  <section class="panel">
    <h2>To Do</h2>
    <p class="empty">Coming in Phase 5</p>
  </section>
</aside>

<style>
  .sidebar {
    width: 280px; flex-shrink: 0; border-left: 1px solid var(--status-bar);
    overflow-y: auto; background: var(--bg);
  }
  .panel { padding: 0.75rem; border-top: 1px solid var(--status-bar); }
  .panel h2 { margin: 0 0 0.5rem; font-size: 0.9rem; color: var(--heading-2); }
  .empty { color: var(--muted); font-size: 0.8rem; margin: 0; }
</style>
