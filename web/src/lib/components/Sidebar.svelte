<script lang="ts">
  import Calendar from './Calendar.svelte';
  import Agenda from './Agenda.svelte';
  import TodoList from './TodoList.svelte';
</script>

<aside class="sidebar">
  <Calendar />
  <Agenda />
  <TodoList />
</aside>

<style>
  .sidebar {
    width: 280px; flex-shrink: 0; border-left: 1px solid var(--status-bar);
    overflow-y: auto; background: var(--bg);
  }
</style>
