<script lang="ts">
  import { app } from '../appState.svelte';
</script>

<main class="editor">
  <pre>{app.noteContent}</pre>
</main>

<style>
  .editor { flex: 1; min-width: 0; overflow-y: auto; padding: 1rem 1.5rem; }
  pre {
    margin: 0; white-space: pre-wrap; word-break: break-word;
    font-family: var(--font), system-ui, sans-serif; font-size: 0.95rem; line-height: 1.5;
  }
</style>
