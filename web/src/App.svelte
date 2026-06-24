<script lang="ts">
  import { onMount } from 'svelte';
  import Header from './lib/components/Header.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import EditorPane from './lib/components/EditorPane.svelte';
  import { app } from './lib/appState.svelte';

  onMount(() => {
    app.init();
  });

  function onKeydown(e: KeyboardEvent) {
    const target = e.target;
    const typing = target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;

    if ((e.ctrlKey || e.metaKey) && (e.key === 't' || e.key === 'T')) {
      e.preventDefault();
      app.goToday();
      return;
    }
    if (typing) return;
    if (e.key === '[') {
      e.preventDefault();
      app.prevDay();
    } else if (e.key === ']') {
      e.preventDefault();
      app.nextDay();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="app">
  <Header />
  <div class="body">
    <EditorPane />
    <Sidebar />
  </div>
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
  .body { display: flex; flex: 1; min-height: 0; }
</style>
