# Phase 6: Themes & Packaging Polish — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Prerequisites:** Phases 1–5 complete. Theming groundwork already exists: `theme.ts` (light/dark token maps + blue heading ramp + `resolveTokens` partial overrides + `applyTheme`), `/api/config` serving the UI subset, and `:theme` live switching (Phase 4).

**Goal:** Finish the product: bundle **Roboto for fully-offline use**, add **non-blocking error surfacing** (load/save/config failures), and harden the **single-binary release build** (optimized profile, `--version`, `make dist`). Verify the agreed theming (config defaults, partial color overrides, `:theme` session switching, configurable edit-line position) end-to-end.

**Architecture:** Roboto is vendored via the `@fontsource/roboto` package and imported in `main.ts`, so Vite bundles the `woff2` files into `web/dist` and `rust-embed` ships them inside the binary — no network at runtime. Error surfacing adds a small reactive `error` field + auto-dismissing `Toast` to the store/UI; failing API paths set it. The release profile is tuned in `Cargo.toml`.

> **Scope note:** `:theme` is session-scoped (default comes from config; no live config writes) and custom colors come from `config.toml` — both already implemented; this phase verifies them rather than re-building them.

**Tech Stack:** Svelte 5, Vite, TypeScript, `@fontsource/roboto`, Rust release profile.

---

## File Structure

| File | Responsibility |
|---|---|
| `web/package.json` (modify) | Add `@fontsource/roboto` |
| `web/src/main.ts` (modify) | Import Roboto weights so Vite bundles them |
| `web/src/app.css` (modify) | Ensure `--font` uses the bundled family |
| `web/src/lib/appState.svelte.ts` (modify) | `error` state + `setError`/`clearError`; wire failures |
| `web/src/lib/components/Toast.svelte` (create) | Non-blocking error banner |
| `web/src/App.svelte` (modify) | Mount `Toast` |
| `Cargo.toml` (modify) | `[profile.release]` tuning |
| `src/cli.rs` (modify) | Enable `--version` |
| `Makefile` (modify) | Add `dist` target |

All `npm` commands run from `web/`; `cargo`/`make` from the repo root.

---

## Task 1: Bundle Roboto for offline use

**Files:**
- Modify: `web/package.json` (via npm), `web/src/main.ts`, `web/src/app.css`

- [ ] **Step 1: Install the vendored font package**

Run (from `web/`):
```bash
npm install @fontsource/roboto
```
Expected: `@fontsource/roboto` added to `dependencies`; it ships self-hosted `woff2` files (no CDN).

- [ ] **Step 2: Import the weights in `web/src/main.ts`**

Add these imports at the **top** of `web/src/main.ts`, before the existing `import './app.css'`:
```ts
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';
```
(These CSS files declare `@font-face` rules whose `url()`s point at the package's `woff2`; Vite resolves and bundles them into `web/dist/assets/`.)

- [ ] **Step 3: Confirm `--font` references Roboto in `web/src/app.css`**

Ensure the `:root` block in `web/src/app.css` contains (it should already from Phase 3):
```css
  --font: 'Roboto';
```
And that `body` uses it with a system fallback (already present):
```css
body {
  font-family: var(--font), system-ui, -apple-system, sans-serif;
}
```

- [ ] **Step 4: Build and verify the fonts are bundled (not fetched remotely)**

Run (from `web/`):
```bash
npm run build
ls dist/assets | grep -i -E "roboto|woff2"
```
Expected: the build succeeds and one or more `*.woff2` files (Roboto) are present under `web/dist/assets/`. There must be **no** reference to `fonts.googleapis.com`/`fonts.gstatic.com` in the build output:
```bash
grep -rIl "fonts.g" dist || echo "no remote font references (good)"
```
Expected: `no remote font references (good)`.

- [ ] **Step 5: Commit**

```bash
git add web/package.json web/package-lock.json web/src/main.ts web/src/app.css
git commit -m "feat: bundle Roboto via @fontsource for fully-offline fonts"
```

---

## Task 2: Non-blocking error surfacing

**Files:**
- Modify: `web/src/lib/appState.svelte.ts`
- Create: `web/src/lib/components/Toast.svelte`
- Modify: `web/src/App.svelte`

Failures currently only `console.error`. This adds a reactive `error` field with auto-dismiss and a small banner, and wires the config/load/save paths to it.

- [ ] **Step 1: Add error state + helpers to the store**

In `appState.svelte.ts`, add a field alongside the other `$state` fields:
```ts
  error = $state<string | null>(null);
```
And a private timer field next to `saveTimer`:
```ts
  private errorTimer: ReturnType<typeof setTimeout> | null = null;
```
Add these methods to the `AppStore` class (e.g. just before `prevMonth`):
```ts
  setError(message: string): void {
    this.error = message;
    if (this.errorTimer) clearTimeout(this.errorTimer);
    this.errorTimer = setTimeout(() => {
      this.error = null;
    }, 5000);
  }

  clearError(): void {
    if (this.errorTimer) {
      clearTimeout(this.errorTimer);
      this.errorTimer = null;
    }
    this.error = null;
  }
```

- [ ] **Step 2: Wire failures to `setError`**

In `init`, the `catch` around config loading — replace its body with:
```ts
      console.error(e);
      this.setError('Failed to load settings; using defaults.');
```

In `loadActive`, the `catch` — replace its body with:
```ts
      console.error(e);
      this.setError(`Failed to load note ${date}.`);
```

In `flush`, the `catch` — replace its body with:
```ts
      console.error(e);
      this.editor = { ...this.editor, message: 'Save failed' };
      this.setError('Save failed — your edits are kept in memory and will retry.');
```

- [ ] **Step 3: Create `web/src/lib/components/Toast.svelte`**

```svelte
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
```

- [ ] **Step 4: Mount the toast in `web/src/App.svelte`**

Add the import alongside the others:
```ts
  import Toast from './lib/components/Toast.svelte';
```
And place `<Toast />` just before the closing `</div>` of `.app` (after `<StatusLine />`):
```svelte
  <StatusLine />
  <Toast />
</div>
```

- [ ] **Step 5: Type-check and build**

Run (from `web/`):
```bash
npm run check && npm run build && npm test
```
Expected: `svelte-check` clean; build succeeds; unit tests pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/appState.svelte.ts web/src/lib/components/Toast.svelte web/src/App.svelte
git commit -m "feat: surface load/save/config errors via a dismissable toast"
```

---

## Task 3: Release profile, `--version`, and `make dist`

**Files:**
- Modify: `Cargo.toml`, `src/cli.rs`, `Makefile`

- [ ] **Step 1: Tune the release profile in `Cargo.toml`**

Append to `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```
(LTO + a single codegen unit + symbol stripping yield a smaller, faster single binary. Unwinding is left enabled for safety with axum/tokio.)

- [ ] **Step 2: Enable `--version` in `src/cli.rs`**

In the `#[command(...)]` attribute on `Cli`, add `version` so clap wires `--version` to the crate version. Change:
```rust
#[command(name = "slugline", about = "Keyboard-driven daily notes")]
```
to:
```rust
#[command(name = "slugline", version, about = "Keyboard-driven daily notes")]
```

- [ ] **Step 3: Add a `dist` target to the `Makefile`**

Add to the `.PHONY` line and append the target (recipe lines **tab**-indented):
```makefile
dist: build
	@echo "Built single binary:"
	@ls -lh target/release/slugline
```

- [ ] **Step 4: Verify the optimized build, version flag, and binary**

From the repo root:
```bash
make dist
./target/release/slugline --version
./target/release/slugline --help
```
Expected:
- `make dist` runs the frontend build + release build and prints the binary size.
- `--version` prints `slugline 0.1.0` (the crate version).
- `--help` lists `--notes-dir`, `--port`, `--no-open`, `--config`.

- [ ] **Step 5: Smoke-run the optimized binary**

```bash
./target/release/slugline --notes-dir ./dev-notes --no-open --port 4747
```
Open `http://127.0.0.1:4747`, confirm the app loads, then stop the server.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/cli.rs Makefile
git commit -m "build: tune release profile, add --version and make dist"
```

---

## Task 4: End-to-end theming, fonts & error verification

**Files:** none (verification only)

- [ ] **Step 1: Run the production binary**

From the repo root:
```bash
make dist
./target/release/slugline --notes-dir ./dev-notes --no-open --port 4747
```
Open `http://127.0.0.1:4747` and focus the window.

- [ ] **Step 2: Verify offline Roboto**

- In DevTools → Network, reload and confirm the `*.woff2` requests are served from `127.0.0.1:4747` (same origin), **not** from any Google domain.
- Set the browser offline (DevTools → Network → Offline) and hard-reload: the app and its fonts still load (everything is embedded in the binary).
- Body text and headings render in **Roboto** (headings bold via the blue ramp).

- [ ] **Step 3: Verify themes & live switching**

- Default theme matches `config.toml` (`light` out of the box): calm blue heading ramp on a near-white canvas; the heading colors step `--heading-1` → `--heading-6`.
- Run `:theme dark` → switches live to the deep slate-indigo canvas; `:theme light` switches back. (Session-scoped — the launch default still comes from config.)

- [ ] **Step 4: Verify config-driven customization (restart-to-apply)**

- Stop the server. Edit `~/.config/slugline/config.toml`:
  ```toml
  [ui]
  theme = "dark"
  edit_line_position = 0.35

  [ui.colors.dark]
  "--accent" = "#e0af68"
  ```
- Restart the binary and reload. Expected: launches in **dark**, the accent color (calendar selection, agenda time, status message) is the custom gold, and the **edit line sits ~35%** down the editor instead of centered.
- Restore the config (or delete it to regenerate defaults) when done.

- [ ] **Step 5: Verify error surfacing**

- Stop the backend while the page stays open, then trigger a save (type a character and wait ~1s, or run `:w`). Expected: a red **toast** appears (`Save failed — your edits are kept in memory and will retry.`) and the buffer is **not** lost; restart the backend and the next edit/`:w` succeeds. The toast auto-dismisses after ~5s and has a × to dismiss early.

---

## Phase 6 Done Criteria

- `cd web && npm run build` bundles Roboto `woff2` into `web/dist/assets/` with no remote font references; `npm run check`/`npm test` are green.
- `make dist` produces an optimized single binary that serves the SPA + embedded fonts fully offline; `--version` and `--help` work.
- Themes: light default, `:theme` live switching, and `config.toml` partial color overrides + `edit_line_position` all take effect (overrides on restart).
- Load/save/config failures surface as a non-blocking, auto-dismissing toast without losing in-memory edits.

## Self-Review Notes (performed during authoring)

- **Spec coverage (roadmap Phase 6 row):** built-in light/dark token maps + blue ramp and partial per-theme overrides and `:theme` switching already shipped in Phases 3–4 (verified in Task 4, not rebuilt); **bundled-offline Roboto** (Task 1), **release build / single binary** hardened with optimized profile + `make dist` + `--version` (Task 3), and **error-surfacing polish** (Task 2) are delivered here.
- **Offline guarantee:** `@fontsource` self-hosts `woff2`; importing in `main.ts` makes Vite bundle them into `web/dist`, which `rust-embed` ships in the binary and `static_handler` serves with the correct `font/woff2` mime (via `mime_guess`). Task 1 Step 4 explicitly checks there are no remote font URLs.
- **Type/code consistency:** the store additions reuse existing fields/methods (`editor`, `saveTimer`, the `init`/`loadActive`/`flush` catch blocks defined in Phase 4) and `app.config.edit_line_position` (consumed by the Phase 4 `EditorPane` scroll effect). `Toast.svelte` uses `app.error`/`app.clearError` defined in Task 2.
- **No placeholders:** every code step contains complete code; runtime/visual behavior is covered by the explicit manual checklist (Task 4), consistent with prior phases for non-unit-testable UI/build concerns.

