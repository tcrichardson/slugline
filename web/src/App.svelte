<script lang="ts">
  import { onMount } from 'svelte';
  import Header from './lib/components/Header.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import EditorPane from './lib/components/EditorPane.svelte';
  import StatusLine from './lib/components/StatusLine.svelte';
  import Toast from './lib/components/Toast.svelte';
  import { app } from './lib/appState.svelte';

  onMount(() => {
    app.init();
  });

  function onKeydown(e: KeyboardEvent) {
    // Let browser-level Cmd/Meta shortcuts (reload, devtools) and function keys through.
    if (e.metaKey) return;
    if (/^F\d{1,2}$/.test(e.key)) return;
    if (e.key === 'Shift' || e.key === 'Control' || e.key === 'Alt' || e.key === 'Meta') return;
    e.preventDefault();
    app.onKey({ key: e.key, ctrl: e.ctrlKey, meta: e.metaKey, shift: e.shiftKey });
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="app">
  <Header />
  <div class="body">
    <EditorPane />
    <Sidebar />
  </div>
  <StatusLine />
  <Toast />
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
  .body { display: flex; flex: 1; min-height: 0; }
</style>
