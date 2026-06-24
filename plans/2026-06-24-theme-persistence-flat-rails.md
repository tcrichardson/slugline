# Theme Persistence & Flat Twin-Rails UI — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Prerequisites:** MVP complete. Theming groundwork exists: `theme.ts` (light/dark token maps + `resolveTokens` partial overrides + `applyTheme`), `GET /api/config`, `:theme light|dark` live (session-only) switching, and `[ui.colors.<theme>]` config overrides.

**Goal:** Make `:theme` persist the chosen theme to `config.toml`, add a no-argument `:theme` toggle, and restyle the header, active edit line, and status line as a coherent flat "twin-rails" chrome system.

**Architecture:** The frontend keeps applying themes live, then fires `PUT /api/config/theme`; the Rust server surgically rewrites only `ui.theme` in `config.toml` using `toml_edit` (comment/format-preserving) and `GET /api/config` re-reads the file each request so a browser refresh reflects the persisted value. The three horizontal bands (header, active edit line, status line) share a new theme-aware `--rule` hairline; the active line gets a dedicated full-width `--edit-bar-bg` band.

**Tech Stack:** Rust (axum, serde, `toml`, new `toml_edit`), Svelte 5 + TypeScript + Vite, vitest.

---

## File Structure

| File | Change | Responsibility |
|---|---|---|
| `Cargo.toml` | modify | Add `toml_edit` dependency |
| `src/config.rs` | modify | `update_theme(path, theme)` — surgical, format-preserving write; `read_ui(path)` helper |
| `src/app.rs` | modify | `AppState` holds `config_path`; `GET /api/config` re-reads file; new `PUT /api/config/theme` |
| `src/main.rs` | modify | Pass `config_path` into `AppState` instead of `ui` |
| `web/src/lib/doc/command.ts` | modify | Allow no-arg `:theme` (toggle), still reject unknown values |
| `web/src/lib/editor/commands.ts` | modify | `theme` effect carries `''` for toggle |
| `web/src/lib/theme.ts` | modify | `nextTheme()` helper; add `--rule` + `--edit-bar-bg` to LIGHT/DARK |
| `web/src/lib/api.ts` | modify | `putTheme(theme)` client call |
| `web/src/lib/appState.svelte.ts` | modify | Resolve toggle, persist theme, set status message |
| `web/src/app.css` | modify | `:root` defaults for `--rule` + `--edit-bar-bg` |
| `web/src/lib/components/Header.svelte` | modify | Flat band + `--rule` bottom hairline; clock stacked on the right |
| `web/src/lib/components/EditorPane.svelte` | modify | Full-width `--edit-bar-bg` band on the active line + `--rule` hairlines |
| `web/src/lib/components/StatusLine.svelte` | modify | `--rule` top hairline |
| `README.md` | modify | Document persistence + new tokens |
| Tests | modify | `src/config.rs`, `src/app.rs`, `web/src/lib/doc/command.test.ts`, `web/src/lib/theme.test.ts` |

**Commands:** `cargo`/`make` from repo root; `npm`/`npx` from `web/`.

---

## Task 1: `update_theme` — surgical, format-preserving config write

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/config.rs` (add `update_theme`, `read_ui`, tests)

- [ ] **Step 1: Add the `toml_edit` dependency**

In `Cargo.toml`, under `[dependencies]` (keep alphabetical-ish ordering near `toml`):
```toml
toml_edit = "0.25"
```

- [ ] **Step 2: Write the failing test for comment-preserving theme write**

Append to the `mod tests` block in `src/config.rs` (before the closing `}`):
```rust
    #[test]
    fn update_theme_preserves_comments_and_changes_value() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(
            &path,
            "# my notes config\n[ui]\ntheme = \"light\"  # current theme\nfont = \"Roboto\"\n",
        )
        .unwrap();

        update_theme(&path, "dark").unwrap();

        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("# my notes config"), "leading comment kept");
        assert!(after.contains("# current theme"), "inline comment kept");
        assert!(after.contains("theme = \"dark\""), "theme updated");
        // Round-trips through the normal parser.
        let cfg = Config::from_toml(&after).unwrap();
        assert_eq!(cfg.ui.theme, "dark");
    }

    #[test]
    fn update_theme_creates_file_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");
        update_theme(&path, "dark").unwrap();
        let cfg = load_or_create(&path).unwrap();
        assert_eq!(cfg.ui.theme, "dark");
    }

    #[test]
    fn read_ui_defaults_on_missing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("absent.toml");
        let ui = read_ui(&path);
        assert_eq!(ui.theme, "light");
    }
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test --lib config::tests::update_theme_preserves_comments_and_changes_value`
Expected: FAIL to compile — `update_theme`/`read_ui` not found.

- [ ] **Step 4: Implement `update_theme` and `read_ui`**

In `src/config.rs`, add these public functions after `load_or_create` (before `#[cfg(test)]`):
```rust
/// Read just the UI config subset from `path`, falling back to defaults on any error.
pub fn read_ui(path: &Path) -> UiConfig {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| Config::from_toml(&s).ok())
        .map(|c| c.ui)
        .unwrap_or_default()
}

/// Surgically set `ui.theme` in the TOML at `path`, preserving comments and
/// formatting. Creates the file with defaults first if it does not exist.
pub fn update_theme(path: &Path, theme: &str) -> io::Result<()> {
    let existing = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let s = toml::to_string_pretty(&Config::default())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, &s)?;
            s
        }
        Err(e) => return Err(e),
    };

    let mut doc = existing
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    doc["ui"]["theme"] = toml_edit::value(theme);
    fs::write(path, doc.to_string())
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --lib config::tests`
Expected: PASS (all config tests, including the three new ones).

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/config.rs
git commit -m "feat(config): add format-preserving update_theme + read_ui"
```

---

## Task 2: Persist endpoint + file-backed `GET /api/config`

**Files:**
- Modify: `src/app.rs` (AppState, handlers, router, tests)
- Modify: `src/main.rs` (construct AppState with `config_path`)

- [ ] **Step 1: Update the test imports and `test_state` helper in `src/app.rs`**

In the `mod tests` block, replace the import lines so the now-unused `PathBuf` and `UiConfig` imports are dropped. The full import list for `mod tests` should be exactly:
```rust
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tempfile::tempdir;
    use tower::ServiceExt;

    use crate::store::NotesStore;
```

Replace the existing `test_state` helper with a version that takes the `tempdir` handle and writes a real config file (the new `get_config` reads from disk):
```rust
    fn test_state(tmp: &tempfile::TempDir) -> SharedState {
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, "[ui]\ntheme = \"light\"\nfont = \"Roboto\"\n").unwrap();
        Arc::new(AppState {
            store: NotesStore::new(tmp.path().join("notes")),
            config_path,
        })
    }
```

Then update every existing call site in this test module from `test_state(dir.path().to_path_buf())` to `test_state(&dir)` (in `get_note_materializes_and_returns_markdown`, `get_note_rejects_bad_date`, `put_then_list_round_trips`, and `serves_spa_index_at_root`). Replace the `config_endpoint_returns_ui_json` test and add two new theme tests:
```rust
    #[tokio::test]
    async fn config_endpoint_returns_ui_json() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(&dir));
        let resp = app
            .oneshot(Request::get("/api/config").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp).await;
        assert!(body.contains("\"theme\":\"light\""));
        assert!(body.contains("\"font\":\"Roboto\""));
    }

    #[tokio::test]
    async fn put_theme_persists_and_get_reflects_it() {
        let dir = tempdir().unwrap();
        let state = test_state(&dir);
        let path = state.config_path.clone();
        let app = build_router(state);

        let put = app
            .clone()
            .oneshot(
                Request::put("/api/config/theme")
                    .header("content-type", "application/json")
                    .body(Body::from("{\"theme\":\"dark\"}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(put.status(), StatusCode::NO_CONTENT);

        assert!(std::fs::read_to_string(&path).unwrap().contains("theme = \"dark\""));

        let get = app
            .oneshot(Request::get("/api/config").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(body_string(get).await.contains("\"theme\":\"dark\""));
    }

    #[tokio::test]
    async fn put_theme_rejects_unknown_value() {
        let dir = tempdir().unwrap();
        let state = test_state(&dir);
        let path = state.config_path.clone();
        let app = build_router(state);

        let put = app
            .oneshot(
                Request::put("/api/config/theme")
                    .header("content-type", "application/json")
                    .body(Body::from("{\"theme\":\"neon\"}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(put.status(), StatusCode::BAD_REQUEST);
        // File untouched.
        assert!(std::fs::read_to_string(&path).unwrap().contains("theme = \"light\""));
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --lib app::tests`
Expected: FAIL to compile — `AppState` has no `config_path`; route `/api/config/theme` missing.

- [ ] **Step 3: Update `AppState`, handlers, and router**

In `src/app.rs`, change the imports and `AppState`, and add the handler + route. Replace the top of the file (imports + struct + router) with:
```rust
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Json, Router};

use serde::Deserialize;

use crate::config::{read_ui, update_theme};
use crate::store::NotesStore;

pub struct AppState {
    pub store: NotesStore,
    pub config_path: PathBuf,
}

pub type SharedState = Arc<AppState>;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/api/notes", get(list_notes))
        .route("/api/notes/{date}", get(get_note).put(put_note))
        .route("/api/config", get(get_config))
        .route("/api/config/theme", put(put_config_theme))
        .fallback(crate::assets::static_handler)
        .with_state(state)
}
```

Replace the existing `get_config` function with the file-backed version, and add the new handler + request type below it:
```rust
async fn get_config(State(state): State<SharedState>) -> impl IntoResponse {
    Json(read_ui(&state.config_path))
}

#[derive(Deserialize)]
struct ThemeRequest {
    theme: String,
}

async fn put_config_theme(
    State(state): State<SharedState>,
    Json(req): Json<ThemeRequest>,
) -> impl IntoResponse {
    if req.theme != "light" && req.theme != "dark" {
        return (StatusCode::BAD_REQUEST, "unknown theme").into_response();
    }
    match update_theme(&state.config_path, &req.theme) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
```

Note: the old `use crate::config::UiConfig;` import at the top of the file is removed (the struct no longer stores `UiConfig`). Also remove the now-unused `use crate::config::UiConfig;` line inside `mod tests` if present.

- [ ] **Step 4: Update `src/main.rs` to pass the config path**

In `src/main.rs`, change the `AppState` construction (the `let state = Arc::new(...)` block) to:
```rust
    let state = Arc::new(AppState {
        store: NotesStore::new(resolved.notes_dir.clone()),
        config_path: config_path.clone(),
    });
```
The `config.ui` clone is no longer needed there (server settings still come from `resolved`); `config` is still used by `resolve(&cli, &config)` above, so keep the earlier `load_or_create` call.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --lib app::tests`
Expected: PASS (all app tests including the two new theme tests).

- [ ] **Step 6: Build the whole crate to catch unused imports**

Run: `cargo build`
Expected: builds clean (no `unused import` warnings for `UiConfig`).

- [ ] **Step 7: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat(api): persist theme via PUT /api/config/theme; serve config from disk"
```

---

## Task 3: No-argument `:theme` toggle (pure logic)

**Files:**
- Modify: `web/src/lib/doc/command.ts`
- Modify: `web/src/lib/theme.ts` (add `nextTheme`)
- Test: `web/src/lib/doc/command.test.ts`, `web/src/lib/theme.test.ts`

- [ ] **Step 1: Write the failing tests**

In `web/src/lib/doc/command.test.ts`, inside the `describe('validateCommand', ...)` block, add:
```ts
  it('allows :theme with no argument (toggle)', () => {
    const r = validateCommand('theme');
    expect(r.ok).toBe(true);
    if (r.ok) expect(r.arg).toBe('');
  });
```

In `web/src/lib/theme.test.ts`, add a new describe block:
```ts
import { nextTheme } from './theme';

describe('nextTheme', () => {
  it('flips dark to light and anything else to dark', () => {
    expect(nextTheme('dark')).toBe('light');
    expect(nextTheme('light')).toBe('dark');
    expect(nextTheme('whatever')).toBe('dark');
  });
});
```
(Add `nextTheme` to the existing `import { resolveTokens, LIGHT, DARK } from './theme';` line, or use the separate import above — both work.)

- [ ] **Step 2: Run the tests to verify they fail**

Run (from `web/`): `npx vitest run src/lib/doc/command.test.ts src/lib/theme.test.ts`
Expected: FAIL — `theme` requires an argument; `nextTheme` is not exported.

- [ ] **Step 3: Allow no-arg theme in `command.ts`**

In `web/src/lib/doc/command.ts`, change the `theme` spec (line 42) to not require an argument:
```ts
  theme: { name: 'theme', argKind: 'theme', argRequired: false },
```
And change the theme validation (lines 67–69) to accept an empty arg (toggle) while still rejecting unknown values:
```ts
  if (spec.argKind === 'theme' && arg !== '' && arg !== 'light' && arg !== 'dark') {
    return { ok: false, error: 'Expected light or dark' };
  }
```

- [ ] **Step 4: Add `nextTheme` to `theme.ts`**

In `web/src/lib/theme.ts`, add after `builtinTokens`:
```ts
/** The opposite of the given theme (anything not 'dark' flips to 'dark'). */
export function nextTheme(theme: string): string {
  return theme === 'dark' ? 'light' : 'dark';
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run (from `web/`): `npx vitest run src/lib/doc/command.test.ts src/lib/theme.test.ts`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/doc/command.ts web/src/lib/doc/command.test.ts web/src/lib/theme.ts web/src/lib/theme.test.ts
git commit -m "feat(editor): allow no-arg :theme toggle; add nextTheme helper"
```

---

## Task 4: Wire toggle resolution + persistence into the app store

**Files:**
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/editor/commands.ts`
- Modify: `web/src/lib/appState.svelte.ts`

- [ ] **Step 1: Add the `putTheme` API client**

Append to `web/src/lib/api.ts`:
```ts
export async function putTheme(theme: string): Promise<void> {
  const res = await fetch('/api/config/theme', {
    method: 'PUT',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ theme }),
  });
  if (!res.ok) throw new Error(`putTheme failed: ${res.status}`);
}
```

- [ ] **Step 2: Stop hard-coding the theme message in `commands.ts`**

In `web/src/lib/editor/commands.ts`, change the `theme` case (line 184–185) so the status message is set by the store after the real target is resolved (the arg may be `''` for a toggle):
```ts
    case 'theme':
      return { state: { ...base, message: '' }, effect: { type: 'theme', theme: arg } };
```

- [ ] **Step 3: Resolve the toggle and persist in `appState.svelte.ts`**

In `web/src/lib/appState.svelte.ts`, extend the imports:
```ts
import { applyTheme, nextTheme } from './theme';
import { getConfig, listNotes, getNote, putNote, putTheme } from './api';
```

Replace the `case 'theme':` block inside `runEffect` (lines 148–153) with:
```ts
      case 'theme': {
        if (!this.config) return;
        const target = effect.theme === '' ? nextTheme(this.config.theme) : effect.theme;
        this.config = { ...this.config, theme: target };
        applyTheme(target, this.config.font, this.config.colors);
        this.editor = { ...this.editor, message: `Theme: ${target}` };
        await this.persistTheme(target);
        return;
      }
```

Add this private method (e.g. just after `flush()`):
```ts
  private async persistTheme(theme: string): Promise<void> {
    try {
      await putTheme(theme);
    } catch (e) {
      console.error(e);
      this.setError('Failed to save theme preference.');
    }
  }
```

- [ ] **Step 4: Type-check and run the web test suite**

Run (from `web/`):
```bash
npm run check
npx vitest run
```
Expected: type-check passes; all existing + new tests pass.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/api.ts web/src/lib/editor/commands.ts web/src/lib/appState.svelte.ts
git commit -m "feat(theme): resolve :theme toggle and persist choice to config"
```

---

## Task 5: Add the `--rule` and `--edit-bar-bg` theme tokens

**Files:**
- Modify: `web/src/lib/theme.ts` (LIGHT + DARK maps)
- Modify: `web/src/app.css` (`:root` defaults)
- Test: `web/src/lib/theme.test.ts`

- [ ] **Step 1: Write the failing test for token presence**

In `web/src/lib/theme.test.ts`, add inside `describe('theme', ...)`:
```ts
  it('defines the rule and edit-bar tokens for both themes', () => {
    for (const t of [LIGHT, DARK]) {
      expect(t['--rule']).toMatch(/^#/);
      expect(t['--edit-bar-bg']).toMatch(/^#/);
    }
  });
```

- [ ] **Step 2: Run the test to verify it fails**

Run (from `web/`): `npx vitest run src/lib/theme.test.ts`
Expected: FAIL — tokens undefined.

- [ ] **Step 3: Add the tokens to both maps**

In `web/src/lib/theme.ts`, add to the `LIGHT` object (after `'--edit-line-bg'`):
```ts
  '--edit-bar-bg': '#e2ebff',
  '--rule': '#d9e0ec',
```
And to the `DARK` object (after `'--edit-line-bg'`):
```ts
  '--edit-bar-bg': '#2a344c',
  '--rule': '#2d3650',
```

- [ ] **Step 4: Add matching `:root` defaults**

In `web/src/app.css`, inside `:root` (after the `--edit-line-bg` line):
```css
  --edit-bar-bg: #e2ebff;
  --rule: #d9e0ec;
```

- [ ] **Step 5: Run the test to verify it passes**

Run (from `web/`): `npx vitest run src/lib/theme.test.ts`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/theme.ts web/src/lib/theme.test.ts web/src/app.css
git commit -m "feat(theme): add --rule and --edit-bar-bg tokens (light + dark)"
```

---

## Task 6: Header — flat band + bottom hairline, clock on the right

**Files:**
- Modify: `web/src/lib/components/Header.svelte`

- [ ] **Step 1: Restructure the markup (clock to the right of the tabs)**

In `web/src/lib/components/Header.svelte`, replace the `<header>` block (lines 15–24) with:
```svelte
<header class="header">
  <div class="name">Slugline</div>
  <Tabs />
  <div class="clock">
    <span>{dateStr}</span>
    <span class="time">{timeStr}</span>
  </div>
</header>
```

- [ ] **Step 2: Replace the styles**

Replace the `<style>` block (lines 26–35) with:
```svelte
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
```

- [ ] **Step 3: Type-check**

Run (from `web/`): `npm run check`
Expected: passes (no template/type errors).

- [ ] **Step 4: Commit**

```bash
git add web/src/lib/components/Header.svelte
git commit -m "feat(ui): flat header band with hairline rule and right-aligned clock"
```

---

## Task 7: Edit bar — full-width band on the active line

**Files:**
- Modify: `web/src/lib/components/EditorPane.svelte`

- [ ] **Step 1: Move horizontal padding from the container to the lines**

In `web/src/lib/components/EditorPane.svelte`, replace the `.editor` and `.line` rules (lines 62–63) with:
```css
  .editor { flex: 1; min-width: 0; overflow-y: auto; padding: 1rem 0; line-height: 1.6; }
  .line {
    white-space: pre-wrap; word-break: break-word; min-height: 1.6em;
    padding: 0 1.5rem; border-block: 1px solid transparent;
  }
```
(The `border-block: 1px solid transparent` reserves space so the active line's hairlines do not shift layout when the cursor moves.)

- [ ] **Step 2: Style the active line as the full-width edit bar**

Immediately after the `.line { ... }` rule, replace the existing `.line.active .raw` rule (line 64) with:
```css
  .line.active {
    background: var(--edit-bar-bg);
    border-block-color: var(--rule);
  }
  .line.active .raw { font-family: ui-monospace, 'SF Mono', monospace; }
```

- [ ] **Step 3: Type-check and run the web suite**

Run (from `web/`):
```bash
npm run check
npx vitest run
```
Expected: passes.

- [ ] **Step 4: Commit**

```bash
git add web/src/lib/components/EditorPane.svelte
git commit -m "feat(ui): full-width edit-bar band on the active line"
```

---

## Task 8: Status line — top hairline rule

**Files:**
- Modify: `web/src/lib/components/StatusLine.svelte`

- [ ] **Step 1: Replace the invisible border with the rule token**

In `web/src/lib/components/StatusLine.svelte`, change the `.status` rule (line 40) so the top border uses `--rule`:
```css
    border-top: 1px solid var(--rule);
```
(Leave the rest of `.status` — `background: var(--status-bar)`, padding, font — unchanged.)

- [ ] **Step 2: Type-check**

Run (from `web/`): `npm run check`
Expected: passes.

- [ ] **Step 3: Commit**

```bash
git add web/src/lib/components/StatusLine.svelte
git commit -m "feat(ui): status line adopts the shared hairline rule"
```

---

## Task 9: Documentation

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Update the Themes feature bullet and Configuration section**

In `README.md`, update the Themes bullet (line 13) to note persistence and the toggle:
```markdown
- **Themes** — built-in `light` (default) and `dark`; switch with `:theme dark` / `:theme light`, or just `:theme` to toggle. The choice is **saved to `config.toml`** (comment-preserving). Partial color overrides via `[ui.colors.<theme>]`.
```

In the Configuration section's example `config.toml` (around lines 196–198), extend the dark overrides block to mention the new tokens:
```toml
[ui.colors.dark]
"--accent" = "#e0af68"
"--rule" = "#2d3650"        # hairline under header / around the edit bar
"--edit-bar-bg" = "#2a344c" # active-line band
```

And replace the "Config changes take effect on restart." line (line 200) with:
```markdown
Most config changes take effect on restart; the active theme is also written back to `config.toml` whenever you switch it in-app.
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: document theme persistence, toggle, and new color tokens"
```

---

## Task 10: Full verification

- [ ] **Step 1: Run the complete Rust + web suites and type-check**

Run from repo root:
```bash
make test
make test-web
cd web && npm run check && cd ..
```
Expected: all green.

- [ ] **Step 2: Format**

Run: `make fmt && make fmt-web`
Expected: no diffs that break the build (commit any formatting changes if produced).

- [ ] **Step 3: Manual smoke test**

Run: `make dev`, open the app, then verify:
- Header is a flat band with a visible hairline underneath and the date/time stacked on the right.
- The active editing line is a full-width tinted band with hairlines top and bottom; moving the cursor up/down does not cause vertical jitter.
- The status line has a matching hairline on top.
- `:theme` toggles light↔dark; `:theme dark` / `:theme light` still work; the status line shows `Theme: <name>`.
- After toggling, the chosen theme appears in `~/.config/slugline/config.toml` (or the dev config), comments preserved; reloading the browser keeps the new theme.
- A `[ui.colors.dark] "--edit-bar-bg" = "#..."` override changes the dark edit-bar color after restart.

- [ ] **Step 4: Final commit (if formatting or docs changed)**

```bash
git add -A
git commit -m "chore: formatting and verification for theme persistence + flat rails"
```

---

## Notes for the implementer

- **`toml_edit` auto-vivification:** `doc["ui"]["theme"] = toml_edit::value(theme)` creates the `[ui]` table if it is missing; since the default config always serializes `[ui]`, this matters only for hand-trimmed files.
- **Why `GET` re-reads the file:** there is no in-memory `UiConfig` to keep in sync, so a browser refresh always reflects the persisted theme without locks. The read happens only on page load/refresh.
- **Edit-bar prominence is tunable:** `--edit-bar-bg` is independent of the sidebar-hover `--edit-line-bg`; bump it (and `--rule`) per theme if the band reads too faint.
- **Mockup reference:** `/var/folders/qt/5ymmsbds7yjflxkd95h91kth0000gn/T/kilo/slugline-theme-mockup.html` (Direction 3, "Twin rails (flat)") is the visual target.
