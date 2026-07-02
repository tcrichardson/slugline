# Phase 7 — Cutover — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.
>
> **This phase has no TypeScript counterpart to port and no new domain logic.** It is a deletion +
> cleanup + documentation pass: remove the now-fully-superseded web frontend and HTTP-server era
> artifacts, finalize the CLI/config surface the Iced app actually uses, and bring `README.md` in
> line with reality. The "TDD" convention still applies in spirit — every step ends with
> `cargo test --workspace` green — but most steps are deletions or doc edits, not new tests.
>
> **Baseline (verified on the real `master` tip before writing this plan):** `1206a2f "chore: fmt
> + clippy clean for phase 6"`. `cargo test --workspace`: 164 `slugline-core` tests + 51 `slugline`
> tests, all green. `cargo fmt --check` clean. `cargo clippy --workspace --all-targets -- -D
> warnings` clean. `cargo run -p slugline` opens the window, reads/writes `config.toml`, shows
> today's note, no panic.
>
> **The `web-final` tag already exists** (`git tag -l` shows it, created back in Phase 0 per
> `docs/superpowers/plans/2026-07-01-iced-port-phase-0-scaffold.md`'s Task 1: `git tag -a web-final
> -m "Last Svelte+axum build before the Rust/Iced port"`, pointing at `593f205` — before any port
> work happened). `git diff web-final HEAD -- web/` is empty: nothing under `web/` has changed
> since that tag, so it already fully satisfies the roadmap's "tag the web version" requirement.
> **This plan does not re-tag anything.**
>
> **The old axum backend is already gone.** There is no top-level `src/` directory, no
> `src/app.rs`, no `src/assets.rs` — Phase 0 already moved `date.rs`/`store.rs`/`config.rs` into
> `slugline-core` and deleted the rest. `Cargo.lock`/`Cargo.toml` already have zero references to
> `axum`, `rust-embed`, `mime_guess`, or `tower` (verified: `grep -rn "axum\|rust-embed\|mime_guess\|tower"
> crates/*/Cargo.toml` and `Cargo.lock` both return nothing). **This plan does not delete any of
> those** — there is nothing left to delete. What *is* still true from `src/config.rs`'s HTTP-server
> era: `ServerConfig`'s `port`/`auto_open` fields (read by the old `/api/config` handler, which no
> longer exists) and the `read_ui()` function (read by the same dead handler) are dead code that
> Phase 0 carried over unchanged ("MOVED from `src/config.rs` (unchanged)"). This phase removes
> them — the one piece of actual cutover cleanup left to do.

**Goal:** Delete `web/` (the Svelte frontend, fully superseded since Phase 1c's walking skeleton),
remove the last vestiges of the deleted HTTP-server config surface (`ServerConfig.port`/`auto_open`,
`read_ui()`), and rewrite `README.md` (and the `Makefile`) to describe the shipped Iced desktop app
instead of the axum+Svelte app it used to be.

**Architecture:** No new modules. `slugline_core::config::ServerConfig` is renamed
`NotesConfig` (it now holds exactly one field: `notes_dir`) and `Config::server` is renamed
`Config::notes`, matching the `[notes]` TOML table name (was `[server]`) — there is no server, so
keeping that name would keep documenting a lie. `cli.rs`'s `resolve()` follows the rename. The
Makefile drops every `web/`-touching target. The README is restructured around the actual binary:
one Cargo workspace, a native window, no browser, no HTTP API.

**Tech Stack:** No new dependencies; this phase only removes dead code and stale docs.

---

## Prerequisites

- **Phases 0–6 are complete and committed on `master`**, `cargo test --workspace` green (164
  `slugline-core` + 51 `slugline` tests, as of `1206a2f "chore: fmt + clippy clean for phase 6"`).
- The `web-final` tag exists and matches the current `web/` tree exactly (verify with `git diff
  web-final HEAD -- web/` before starting — it must print nothing).

## Scope

**In this phase:**
- Delete `web/` (Svelte 5 + Vite frontend: `~1,591` LOC of pure logic + `~465` LOC of components +
  `~951` LOC of tests, all ported to `slugline-core`/`slugline` by Phases 1–6) and its
  `.gitignore` entries (`/web/dist`, `/web/node_modules`).
- `slugline-core::config`: rename `ServerConfig` → `NotesConfig` (drop `port`/`auto_open` and their
  `default_port()`/serde defaults — dead since the axum server that read them no longer exists);
  rename `Config::server` → `Config::notes`; TOML table `[server]` → `[notes]`. Delete `read_ui()`
  (dead: only the deleted `/api/config` handler ever called it — `App::new` has read `config.ui`
  directly since Phase 6). Update every test in `config.rs` accordingly.
- `crates/slugline/src/cli.rs`: update `resolve()`'s one reference (`config.server.notes_dir` →
  `config.notes.notes_dir`). No CLI flag changes — `--notes-dir`/`--config` already match the
  design's target surface (`--port`/`--no-open` were already dropped in Phase 0).
- `Makefile`: delete `test-web`, `fmt-web`, `dev-web` targets and the `web/` build step inside
  `build`; `run`/`dev`/`test`/`fmt`/`build`/`dist` become plain `cargo` invocations.
- `README.md`: rewrite to describe the shipped app — no Node/Svelte/browser/HTTP-API references;
  updated Project Structure (the two-crate Cargo workspace), Development, Quick Start, and
  Configuration (`[notes]`, no `port`/`auto_open`) sections; add the missing `:people`/`:p` command
  row and a mention of the `⌘K`/`Ctrl+K` command palette and the resizable/collapsible sidebar
  (both shipped in Phases 3/5, never documented in the README).
- Final full-workspace verification: `cargo test --workspace`, `cargo fmt --check`, `cargo clippy
  --workspace --all-targets -- -D warnings`, a `cargo run -p slugline` smoke test.

**Deferred / explicitly not done:**
- **Re-tagging `web-final`.** It already exists and already matches `HEAD`'s `web/` tree exactly
  (verified above). Nothing to do.
- **Renaming the `[notes]` table's `notes_dir` key itself**, or moving it out of a table entirely
  (e.g. a bare top-level `notes_dir = "..."`). `notes_dir` living under a named table is unchanged
  behavior from today; only the table's name (`server` → `notes`) changes, since that's the part
  that was actively misleading (there is no server). Renaming the key too would be gratuitous
  extra config-format churn for zero behavioral benefit.
- **A config-migration path for old `[server]`-shaped `config.toml` files.** Slugline is unreleased
  personal software with no external users yet (confirmed: no `config.toml` fixture or checked-in
  example anywhere in the repo references the old shape outside of `config.rs`'s own tests and this
  README). `load_or_create` already tolerates a missing `[notes]` table via serde defaults, so an
  old file just silently reverts `notes_dir` to the default rather than erroring — acceptable for a
  pre-1.0 tool.
- **Full keybinding/command audit of the README beyond the two gaps above.** Cross-checked the
  README's Motions/Editing/Insert/Tab-navigation tables against
  `crates/slugline-core/src/editor/keymap.rs` line by line: they already match. Only the missing
  `:people`/`:p` command and the undocumented `⌘K` palette / sidebar resize are gaps; nothing else
  drifted.

---

## File Structure (files added/changed/deleted in Phase 7)

```
web/                              # DELETED (entire directory)

crates/slugline-core/
  src/
    config.rs                     # REWRITE: ServerConfig -> NotesConfig (drop port/auto_open),
                                   #          Config::server -> Config::notes, [server] -> [notes]
                                   #          in serialized output; delete read_ui(); update tests

crates/slugline/
  src/
    cli.rs                        # MODIFY: config.server.notes_dir -> config.notes.notes_dir

Makefile                          # REWRITE: drop test-web/fmt-web/dev-web + web build step
README.md                         # REWRITE: drop Svelte/Node/API content; update to match the app
.gitignore                        # MODIFY: drop /web/dist, /web/node_modules entries
```

---

### Task 1: Delete `web/`

**Files:**
- Delete: `web/` (entire directory)

- [ ] **Step 1: Confirm the safety net** — verify the tag matches before deleting anything:

```bash
git diff web-final HEAD -- web/
```

Expected: no output (empty diff).

- [ ] **Step 2: Delete the directory**

```bash
git rm -r web/
```

- [ ] **Step 3: Commit**

```bash
git commit -m "chore: delete web/ (fully superseded by the Iced port)"
```

---

### Task 2: Retire the dead HTTP-server config surface

**Files:**
- Modify: `crates/slugline-core/src/config.rs`
- Modify: `crates/slugline/src/cli.rs`

- [ ] **Step 1: Rewrite `config.rs`'s config structs and helpers** — replace the whole file's
  non-test content (everything above `#[cfg(test)]`):

```rust
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, Item, Value};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub notes: NotesConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

/// Where notes live on disk. Was `ServerConfig` (with `port`/`auto_open`) back when Slugline
/// had an axum HTTP server; the port that remains is a desktop app with no server, so this
/// struct now holds exactly the one field that's still meaningful.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotesConfig {
    #[serde(default = "default_notes_dir")]
    pub notes_dir: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font")]
    pub font: String,
    #[serde(default = "default_edit_line_position")]
    pub edit_line_position: f32,
    #[serde(default)]
    pub colors: BTreeMap<String, BTreeMap<String, String>>,
}

fn default_notes_dir() -> String {
    "~/Documents/Slugline".to_string()
}
fn default_theme() -> String {
    "light".to_string()
}
fn default_font() -> String {
    "Roboto".to_string()
}
fn default_edit_line_position() -> f32 {
    0.5
}

impl Default for NotesConfig {
    fn default() -> Self {
        Self {
            notes_dir: default_notes_dir(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            font: default_font(),
            edit_line_position: default_edit_line_position(),
            colors: BTreeMap::new(),
        }
    }
}

impl Config {
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

/// Expand a leading `~/` to the user's home directory.
pub fn expand_tilde(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(p)
}

/// Default config file path: `~/.config/slugline/config.toml` (XDG-style, also on macOS).
pub fn default_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("slugline").join("config.toml")
}

/// Load config from `path`, creating it with defaults if missing.
pub fn load_or_create(path: &Path) -> io::Result<Config> {
    match fs::read_to_string(path) {
        Ok(s) => Config::from_toml(&s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let cfg = Config::default();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml = toml::to_string_pretty(&cfg).map_err(io::Error::other)?;
            fs::write(path, toml)?;
            Ok(cfg)
        }
        Err(e) => Err(e),
    }
}

/// Surgically set `ui.theme` in the TOML at `path`, preserving comments and
/// formatting. Creates the file with defaults first if it does not exist.
pub fn update_theme(path: &Path, theme: &str) -> io::Result<()> {
    let existing = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            toml::to_string_pretty(&Config::default()).map_err(io::Error::other)?
        }
        Err(e) => return Err(e),
    };

    let mut doc = existing
        .parse::<DocumentMut>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Preserve the inline comment suffix (e.g. `  # current theme`) on the value,
    // but only if `[ui]` and `theme` already exist — avoids a panic on Item::None.
    let existing_suffix = doc
        .get("ui")
        .and_then(|ui| ui.get("theme"))
        .and_then(|item| item.as_value())
        .and_then(|v| v.decor().suffix())
        .and_then(|s| s.as_str())
        .map(|s| s.to_owned());

    let mut new_val = Value::from(theme);
    if let Some(suffix) = existing_suffix {
        new_val.decor_mut().set_suffix(suffix);
    }
    doc["ui"]["theme"] = Item::Value(new_val);

    fs::write(path, doc.to_string())
}
```

Note what left: `read_ui()` is gone (dead — only the deleted `/api/config` handler called it);
`ServerConfig`/`default_port`/`default_true` are gone; `NotesConfig` replaces them with just
`notes_dir`.

- [ ] **Step 2: Update the tests** — replace the `#[cfg(test)] mod tests { ... }` block at the
  bottom of the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn defaults_apply_when_fields_missing() {
        let cfg = Config::from_toml("").unwrap();
        assert_eq!(cfg.notes.notes_dir, "~/Documents/Slugline");
        assert_eq!(cfg.ui.theme, "light");
        assert_eq!(cfg.ui.font, "Roboto");
        assert!((cfg.ui.edit_line_position - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_overrides() {
        let toml = r##"
            [notes]
            notes_dir = "/tmp/my-notes"

            [ui]
            theme = "dark"

            [ui.colors.dark]
            "--bg" = "#101018"
        "##;
        let cfg = Config::from_toml(toml).unwrap();
        assert_eq!(cfg.notes.notes_dir, "/tmp/my-notes");
        assert_eq!(cfg.ui.theme, "dark");
        assert_eq!(cfg.ui.colors["dark"]["--bg"], "#101018");
    }

    #[test]
    fn expands_leading_tilde() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(
            expand_tilde("~/Documents/Slugline"),
            home.join("Documents/Slugline")
        );
        assert_eq!(
            expand_tilde("/abs/path"),
            std::path::PathBuf::from("/abs/path")
        );
    }

    #[test]
    fn load_or_create_writes_default_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");
        let cfg = load_or_create(&path).unwrap();
        assert_eq!(cfg.notes.notes_dir, "~/Documents/Slugline");
        assert!(path.exists());
        // Second load reads the file back.
        let again = load_or_create(&path).unwrap();
        assert_eq!(again.notes.notes_dir, "~/Documents/Slugline");
    }

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
    fn update_theme_no_panic_when_ui_section_absent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        // File exists but has no [ui] section — previously would panic.
        fs::write(&path, "[notes]\nnotes_dir = \"/tmp/x\"\n").unwrap();
        update_theme(&path, "dark").unwrap();
        let cfg = load_or_create(&path).unwrap();
        assert_eq!(cfg.ui.theme, "dark");
    }
}
```

(`read_ui_defaults_on_missing_file` is deleted along with `read_ui()` itself — there's no longer
anything to test.)

- [ ] **Step 3: Update `cli.rs`'s one reference** — in `crates/slugline/src/cli.rs`, in
  `resolve()`, replace:

```rust
            .unwrap_or_else(|| expand_tilde(&config.server.notes_dir)),
```

with:

```rust
            .unwrap_or_else(|| expand_tilde(&config.notes.notes_dir)),
```

- [ ] **Step 4: Run the tests**

```bash
cargo test --workspace
```

Expected: PASS — 163 `slugline-core` tests (164 minus the deleted `read_ui_defaults_on_missing_file`)
+ 51 `slugline` tests.

- [ ] **Step 5: Format and lint**

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: both clean.

- [ ] **Step 6: Commit**

```bash
git add crates/slugline-core/src/config.rs crates/slugline/src/cli.rs
git commit -m "refactor(core): retire the dead HTTP-server config surface (ServerConfig -> NotesConfig)"
```

---

### Task 3: Trim the `Makefile`

**Files:**
- Modify: `Makefile`

- [ ] **Step 1: Replace the whole file**

```makefile
.PHONY: run dev test fmt build dist

# Run the app with the default notes directory (~/Documents/Slugline).
run:
	cargo run -p slugline

# Run with a throwaway notes dir, for local development.
dev:
	cargo run -p slugline -- --notes-dir ./dev-notes

test:
	cargo test --workspace

fmt:
	cargo fmt

# Production build: a single self-contained release binary.
build:
	cargo build --release -p slugline

dist: build
	@echo "Built single binary:"
	@ls -lh target/release/slugline
```

- [ ] **Step 2: Smoke-test the trimmed targets**

```bash
make test
make fmt
```

Expected: both succeed (no diff from `cargo fmt` — the workspace is already formatted).

- [ ] **Step 3: Commit**

```bash
git add Makefile
git commit -m "chore: trim Makefile to cargo-only targets"
```

---

### Task 4: Update `.gitignore`

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: Drop the now-meaningless `web/` entries** — replace:

```
/target
/web/dist
/web/node_modules
/dev-notes
/.superpowers/
/.worktrees/
/.complexity
```

with:

```
/target
/dev-notes
/.superpowers/
/.worktrees/
/.complexity
```

- [ ] **Step 2: Commit**

```bash
git add .gitignore
git commit -m "chore: drop web/-related .gitignore entries"
```

---

### Task 5: Rewrite `README.md`

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Replace the whole file**

```markdown
# Slugline

A single-user, local-first, keyboard-driven (vim-modal) daily notes app. Notes are stored as plain
`YYYY-MM-DD.md` files on disk. Slugline is a native desktop app (Rust + [Iced](https://iced.rs))
that ships as a single self-contained binary — no cloud, no browser, no runtime dependencies.

## Features

- **Vim-modal editor** — NORMAL/INSERT modes, motions (`h j k l w b e 0 $ gg G`), edits (`x dd yy p P o O i a A`), undo/redo (`u` / `Ctrl-R`)
- **Per-line rendering** — one raw edit line; all other lines render pretty (headings, tasks, lists, blockquotes, bold/italic/strikethrough/highlight/code/links)
- **Tabs** — open multiple dates side-by-side (`gt` / `gT`, `:tab`, `:close`)
- **Resizable, collapsible sidebar** — calendar, agenda, and 7-day to-do view, dragged wider/narrower or collapsed to a slim rail
- **Calendar** — dots on dates that have notes; click to open or create; month navigation
- **Agenda** — today's scheduled meetings, click to jump to the meeting
- **To Do** — a 7-day rolling view of open/done items across notes, click to jump to the item
- **Command palette** — press `:` (or `⌘K` / `Ctrl+K`) for a fuzzy-searchable list of every command
- **Themes** — built-in `light` (default) and `dark`; switch with `:theme dark` / `:theme light`, or just `:theme` to toggle. The choice is **saved to `config.toml`** (comment-preserving). Partial color overrides via `[ui.colors.<theme>]`.
- **Offline fonts** — Roboto is bundled inside the binary; no network required

## Usage

Slugline is a modal editor in the style of Vim. Keyboard input is interpreted differently depending on the current mode.

### Modes

| Mode | How to enter | How to exit |
|------|-------------|-------------|
| **Normal** | Default on open; `Escape` from Insert | — |
| **Insert** | `i`, `a`, `A`, `o`, `O` | `Escape` |
| **Command** | `:` from Normal, or `⌘K` / `Ctrl+K` from anywhere | `Escape` to cancel, `Enter` to run |

The cursor changes shape: a block in Normal mode, an I-beam in Insert mode.

### Normal Mode

Normal mode is for navigation and editing commands. Keystrokes are not inserted as text.

#### Motions

| Key | Action |
|-----|--------|
| `h` / `←` | Move left |
| `l` / `→` | Move right |
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `w` | Next word start |
| `b` | Previous word start |
| `e` | Word end |
| `0` | Line start |
| `$` | Line end |
| `gg` | First line |
| `G` | Last line |

#### Editing

| Key | Action |
|-----|--------|
| `x` | Delete character under cursor |
| `dd` | Delete current line (saved to register) |
| `yy` | Yank (copy) current line to register |
| `p` | Paste register below current line |
| `P` | Paste register above current line |
| `t` | Toggle task checkbox (`[ ]` ↔ `[x]`) |
| `u` | Undo |
| `Ctrl-r` | Redo |

#### Entering Insert Mode

| Key | Enters insert at… |
|-----|-------------------|
| `i` | Cursor position |
| `a` | After cursor |
| `A` | End of line |
| `o` | New line below |
| `O` | New line above |

#### Tab & Day Navigation

| Key | Action |
|-----|--------|
| `gt` | Next tab |
| `gT` | Previous tab |
| `[` | Previous day |
| `]` | Next day |
| `Ctrl-t` | Today |

### Insert Mode

Insert mode behaves like a standard text editor.

| Key | Action |
|-----|--------|
| `Escape` | Return to Normal mode |
| `Enter` | Insert newline (splits line at cursor) |
| `Backspace` | Delete character before cursor; at column 0, merges with line above |
| `Tab` | Insert 2 spaces |
| `Ctrl-w` | Delete word before cursor |
| `←` `→` `↑` `↓` | Move cursor |
| any text character | Insert at cursor position |

### Command Mode

Press `:` in Normal mode (or `⌘K` / `Ctrl+K` from either mode) to open the command palette: a
fuzzy-searchable overlay of every command below. Type to filter, press `Enter` to run the top
match (or a fully-typed command line), or press `Escape` to cancel.

#### Navigation

| Command | Description |
|---------|-------------|
| `:goto YYYY-MM-DD` | Open a specific date |
| `:today` | Open today's note |
| `:tab YYYY-MM-DD` | Open a date in a new tab |
| `:close` | Close the active tab |

#### File

| Command | Description |
|---------|-------------|
| `:w` | Save the current note to disk |

#### Content

| Command | Description |
|---------|-------------|
| `:meeting <name>` | Append a `### <name>` block under `## Meetings` (creates section if absent) |
| `:note <name>` | Append a `### <name>` block under `## Notes` (creates section if absent) |
| `:todo <text>` | Append `- [ ] <text>` to `## To Do`; if inside a meeting block, tags it with the meeting name |
| `:section <name>` | Insert a sub-heading one level deeper than the heading at the cursor |
| `:people <names>` / `:p <names>` | Add people to the meeting or note block at the cursor |

#### Meeting Metadata

These commands must be run with the cursor inside a `### meeting` block.

| Command | Description |
|---------|-------------|
| `:scheduled HH:MM` | Set the scheduled time for the meeting |
| `:start` | Record the actual start time (uses the current clock time) |
| `:end` | Record the actual end time (uses the current clock time) |
| `:purpose <text>` | Set the meeting purpose |

#### Note Metadata

These commands must be run with the cursor inside a `### note` block.

| Command | Description |
|---------|-------------|
| `:topic <text>` | Set the topic for the note block |

#### UI

| Command | Description |
|---------|-------------|
| `:theme` | Toggle between light and dark |
| `:theme light` | Switch to the light theme |
| `:theme dark` | Switch to the dark theme |

---

## Quick Start

```sh
# Run from source (opens a native window)
make dev

# Production build → single binary
make dist
./target/release/slugline
```

### Command-line options

```
slugline [OPTIONS]

Options:
      --notes-dir <PATH>   Notes directory (default: ~/Documents/Slugline)
      --config <PATH>      Config file path
  -V, --version            Print version
  -h, --help               Print help
```

## Configuration

Slugline writes a default config on first launch:

- **macOS/Linux:** `~/.config/slugline/config.toml`

Example `config.toml`:

```toml
[notes]
notes_dir = "~/Documents/Slugline"

[ui]
theme = "dark"
edit_line_position = 0.35   # 0.0–1.0, fraction from top

[ui.colors.dark]
"--accent" = "#e0af68"
"--rule" = "#2d3650"        # hairline under header / around the edit bar
"--edit-bar-bg" = "#2a344c" # active-line band
```

Most config changes take effect on restart; the active theme is also written back to `config.toml` whenever you switch it in-app.

## Development

**Prerequisites:** Rust (stable)

```sh
# Run with a throwaway notes dir
make dev

# Tests (whole workspace)
make test

# Format
make fmt

# Lint
cargo clippy --workspace --all-targets -- -D warnings
```

## Project Structure

```
slugline/
├── crates/
│   ├── slugline-core/       # headless domain logic, no UI dependency
│   │   └── src/
│   │       ├── agenda.rs    # scheduled-meeting derivation
│   │       ├── config.rs    # TOML config loading + comment-preserving theme writes
│   │       ├── date.rs      # date validation + weekday helper
│   │       ├── dates.rs     # calendar month-grid math
│   │       ├── doc/         # line classifier, inline-span renderer, scanner, commands
│   │       ├── editor/      # vim-modal state machine (motions, edits, insert, keymap)
│   │       ├── store.rs     # filesystem note store (atomic writes, materialize-on-open)
│   │       ├── tabs.rs      # open-tabs state
│   │       ├── theme.rs     # light/dark color token resolution
│   │       └── todos.rs     # 7-day to-do aggregation
│   └── slugline/            # the Iced desktop app
│       └── src/
│           ├── main.rs      # entry point, CLI wiring
│           ├── app.rs       # Model/Message/update/view/subscription
│           ├── cli.rs       # CLI argument parsing (clap)
│           ├── keys.rs      # Iced key event -> keymap-string mapping
│           ├── theme_iced.rs # theme tokens -> iced::Color
│           └── ui/          # editor_pane, sidebar, calendar, agenda, todo_list,
│                             # tab_strip, command_palette, status_line, toast
├── docs/superpowers/         # design docs and implementation plans
├── fixtures/                 # sample note files used by tests
├── Cargo.toml                # workspace manifest
└── Makefile
```

## Notes Format

Notes are plain Markdown files named `YYYY-MM-DD.md`. The supported subset:

```markdown
# 2026-06-24-TUE

## Morning

- [ ] Task item
- [x] Done task
- Regular list item
  - Indented list item
  1. Ordered list item

> Blockquote text

meta: value

Paragraph with **bold**, *italic*, ~~strikethrough~~, ==highlight==, `code`, and [link text](https://example.com).

## Meetings

::: meeting 09:00–10:00 | Topic | Purpose
scheduled: 09:00
start: 09:00
end: 10:00
topic: Standup
purpose: Sync
:::
```

### Inline markup summary

| Syntax | Renders as |
|--------|-----------|
| `**text**` | **Bold** |
| `*text*` or `_text_` | *Italic* |
| `~~text~~` | ~~Strikethrough~~ |
| `==text==` | Highlighted background |
| `` `code` `` | Inline code |
| `[label](url)` | Hyperlink (http/https/mailto only) |

### Block elements

| Syntax | Renders as |
|--------|-----------|
| `# Heading` … `###### Heading` | H1–H6 headings |
| `- [ ] text` / `- [x] text` | Open / done task checkbox |
| `- text` / `* text` / `+ text` | Unordered list item |
| `1. text` | Ordered list item |
| `  - text` (2-space indent) | Nested list item (depth increases per 2 spaces) |
| `> text` | Blockquote (left border, muted italic) |
| `meta:key value` | Metadata field (used by agenda and meeting blocks) |
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: rewrite README for the native Iced app (no web/HTTP-API content)"
```

---

### Task 6: Final workspace verification

**Files:** none (verification only)

- [ ] **Step 1: Full test suite**

```bash
cargo test --workspace
```

Expected: PASS — 163 `slugline-core` tests + 51 `slugline` tests (one test count lower than the
Phase 6 baseline: `read_ui_defaults_on_missing_file` was deleted in Task 2, nothing else changed
test-wise this phase).

- [ ] **Step 2: Format and lint**

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: both clean.

- [ ] **Step 3: Confirm no stray dependency or dead-code references survived**

```bash
grep -rn "axum\|rust-embed\|mime_guess\|tower\b" Cargo.lock crates/*/Cargo.toml
grep -rn "ServerConfig\|read_ui\|server\.notes_dir\|\[server\]" crates README.md Makefile
```

Expected: both empty.

- [ ] **Step 4: Manual smoke test**

```bash
cargo run -p slugline -- --notes-dir /tmp/slugline-smoke
```

Expected: a native window opens, titled `Slugline — <today>`; today's note materializes as text;
edit a line, wait ~1s, confirm `/tmp/slugline-smoke/<today>.md` was written; close the window
cleanly (no panic).

- [ ] **Step 5: Commit** (only if Step 3/4 required any fixes; otherwise nothing to commit — Tasks
  1–5 already committed their own changes)

## "Done" definition

Matches the roadmap's Phase 7 row: `web/`, the old HTTP-server config surface, and every stale
web-era doc reference are gone; `cargo test --workspace`, `cargo fmt --check`, and `cargo clippy
--workspace --all-targets -- -D warnings` are green; `cargo run -p slugline` (built from a
workspace containing no `web/` directory at all) opens, edits, and autosaves correctly; `README.md`
accurately describes the shipped single-binary desktop app with no reference to Svelte, Node, or an
HTTP API.
