# Phase 0 — Workspace Scaffold & Iced Hello-Note — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restructure the repo into a Cargo workspace (`slugline-core` + `slugline`), move the reusable Rust (`date`/`store`/`config`) into the core crate with its tests intact, and stand up a minimal Iced window that reads today's note via the store and displays it as raw monospace text.

**Architecture:** A virtual Cargo workspace. `slugline-core` is a headless library (no Iced). `slugline` is the Iced binary depending on core. The axum server source is removed (the web build is preserved via a git tag); the `web/` directory is left untouched until the Phase 7 cutover.

**Tech Stack:** Rust (edition 2024), Iced `0.13.x`, `clap` (derive), `chrono` (today's date), `serde`/`toml`/`toml_edit`/`dirs` (config), `tempfile` (dev).

---

## File Structure (end state of Phase 0)

```
Cargo.toml                         # [workspace] virtual manifest (replaces the old package manifest)
crates/
  slugline-core/
    Cargo.toml
    src/
      lib.rs                       # pub mod date; store; config; dates;
      date.rs                      # MOVED from src/date.rs (unchanged)
      store.rs                     # MOVED from src/store.rs (unchanged)
      config.rs                    # MOVED from src/config.rs (unchanged)
      dates.rs                     # NEW: today_iso() (chrono); grows in Phase 2
  slugline/
    Cargo.toml                     # depends on slugline-core + iced + clap
    src/
      main.rs                      # Iced entry + CLI wiring
      cli.rs                       # trimmed CLI (notes_dir, config) + resolve
      app.rs                       # minimal Iced App (Model/Message/update/view/title)
```

**Removed in this phase:** `src/main.rs`, `src/lib.rs`, `src/app.rs`, `src/assets.rs`, `src/cli.rs` (replaced), and the old root `Cargo.toml` package section. **Left for Phase 7:** `web/`, and the axum/`rust-embed`/`mime_guess`/`tower`/`open`/`tokio`/`serde_json` dependency lines (they simply stop being referenced now).

---

### Task 1: Create the branch and preserve the web build

**Files:** none (git only)

- [ ] **Step 1: Create the feature branch**

Run:
```bash
git checkout -b iced-port
```
Expected: `Switched to a new branch 'iced-port'`

- [ ] **Step 2: Tag the last working web+axum build for recovery**

Run:
```bash
git tag -a web-final -m "Last Svelte+axum build before the Rust/Iced port"
git tag --list | grep web-final
```
Expected: prints `web-final`. (This is the recovery point the design promised; the axum source is about to be removed.)

---

### Task 2: Create the workspace and `slugline-core` by moving the reusable Rust

**Files:**
- Create: `Cargo.toml` (workspace), `crates/slugline-core/Cargo.toml`, `crates/slugline-core/src/lib.rs`
- Move: `src/date.rs` → `crates/slugline-core/src/date.rs`; `src/store.rs` → `crates/slugline-core/src/store.rs`; `src/config.rs` → `crates/slugline-core/src/config.rs`

- [ ] **Step 1: Move the three reusable modules with git (preserves history)**

Run:
```bash
mkdir -p crates/slugline-core/src crates/slugline/src
git mv src/date.rs crates/slugline-core/src/date.rs
git mv src/store.rs crates/slugline-core/src/store.rs
git mv src/config.rs crates/slugline-core/src/config.rs
```
Expected: no output (success). `git status` shows the three renames.

- [ ] **Step 2: Delete the now-orphaned axum/server source**

Run:
```bash
git rm src/main.rs src/lib.rs src/app.rs src/assets.rs src/cli.rs
```
Expected: `rm 'src/...'` lines for all five. The `src/` directory is now empty.

- [ ] **Step 3: Write the workspace manifest** (`Cargo.toml`)

Replace the entire root `Cargo.toml` with:
```toml
[workspace]
resolver = "2"
members = ["crates/slugline-core", "crates/slugline"]

[workspace.package]
version = "0.1.0"
edition = "2024"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

- [ ] **Step 4: Write `crates/slugline-core/Cargo.toml`**

```toml
[package]
name = "slugline-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { version = "1.0.228", features = ["derive"] }
toml = "1.1.2"
toml_edit = "0.25"
dirs = "6.0.0"
chrono = { version = "0.4", default-features = false, features = ["clock"] }

[dev-dependencies]
tempfile = "3.27.0"
```

- [ ] **Step 5: Write `crates/slugline-core/src/lib.rs`**

```rust
//! Slugline core: headless domain logic (no UI framework dependency).
pub mod config;
pub mod date;
pub mod dates;
pub mod store;
```

- [ ] **Step 6: Create a stub `crates/slugline-core/src/dates.rs`** (filled in Task 3)

```rust
// today_iso() is implemented in Task 3.
```

- [ ] **Step 7: Create a placeholder binary so the workspace resolves** (`crates/slugline/Cargo.toml`)

```toml
[package]
name = "slugline"
version.workspace = true
edition.workspace = true

[dependencies]
slugline-core = { path = "../slugline-core" }
```

- [ ] **Step 8: Create a placeholder `crates/slugline/src/main.rs`** (replaced in Task 5)

```rust
fn main() {}
```

- [ ] **Step 9: Verify the moved core tests still pass**

Run: `cargo test -p slugline-core`
Expected: PASS — the `date`, `store`, and `config` test modules (14 tests) all pass. `dates` has no tests yet.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "refactor: split into cargo workspace; move date/store/config into slugline-core"
```

---

### Task 3: Add `today_iso()` to core

**Files:**
- Modify: `crates/slugline-core/src/dates.rs`

- [ ] **Step 1: Write the failing test** (append to `crates/slugline-core/src/dates.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::is_valid_date;

    #[test]
    fn today_iso_is_a_valid_yyyy_mm_dd() {
        let t = today_iso();
        assert_eq!(t.len(), 10, "expected YYYY-MM-DD, got {t:?}");
        assert!(is_valid_date(&t), "today_iso() produced an invalid date: {t:?}");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slugline-core dates::`
Expected: FAIL to compile with "cannot find function `today_iso`".

- [ ] **Step 3: Implement `today_iso`** (prepend above the `#[cfg(test)]` block in `dates.rs`)

```rust
use chrono::Local;

/// Today's date in the local timezone, formatted `YYYY-MM-DD`.
pub fn today_iso() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slugline-core dates::`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/slugline-core/src/dates.rs
git commit -m "feat(core): add today_iso()"
```

---

### Task 4: Trimmed CLI in the `slugline` binary

**Files:**
- Create: `crates/slugline/src/cli.rs`
- Modify: `crates/slugline/Cargo.toml`

- [ ] **Step 1: Add binary dependencies** (`crates/slugline/Cargo.toml`)

Replace the file with:
```toml
[package]
name = "slugline"
version.workspace = true
edition.workspace = true

[dependencies]
slugline-core = { path = "../slugline-core" }
iced = "0.13"
clap = { version = "4.6.1", features = ["derive"] }
```

- [ ] **Step 2: Write the failing test** (`crates/slugline/src/cli.rs`)

```rust
use std::path::PathBuf;

use clap::Parser;
use slugline_core::config::{expand_tilde, Config};

#[derive(Parser, Debug, Default)]
#[command(name = "slugline", version, about = "Keyboard-driven daily notes")]
pub struct Cli {
    /// Override the notes directory.
    #[arg(long)]
    pub notes_dir: Option<PathBuf>,
    /// Use a specific config file instead of the default.
    #[arg(long)]
    pub config: Option<PathBuf>,
}

/// The effective notes directory after applying CLI > file > defaults precedence.
pub struct Resolved {
    pub notes_dir: PathBuf,
}

pub fn resolve(cli: &Cli, config: &Config) -> Resolved {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_notes_dir_overrides_config() {
        let cli = Cli { notes_dir: Some(PathBuf::from("/tmp/notes")), config: None };
        let r = resolve(&cli, &Config::default());
        assert_eq!(r.notes_dir, PathBuf::from("/tmp/notes"));
    }

    #[test]
    fn falls_back_to_config_notes_dir() {
        let cli = Cli { notes_dir: None, config: None };
        let r = resolve(&cli, &Config::default());
        let home = dirs::home_dir().unwrap();
        assert_eq!(r.notes_dir, home.join("Documents/Slugline"));
    }
}
```

Note: `dirs` is a transitive dep via `slugline-core`, but the test uses it directly — add `dirs = "6.0.0"` to `[dev-dependencies]` of `crates/slugline/Cargo.toml`:
```toml
[dev-dependencies]
dirs = "6.0.0"
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p slugline cli::`
Expected: FAIL — `todo!()` panics (`not yet implemented`).

- [ ] **Step 4: Implement `resolve`** (replace the `todo!()` body)

```rust
pub fn resolve(cli: &Cli, config: &Config) -> Resolved {
    Resolved {
        notes_dir: cli
            .notes_dir
            .clone()
            .unwrap_or_else(|| expand_tilde(&config.server.notes_dir)),
    }
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p slugline cli::`
Expected: PASS (2 tests).

- [ ] **Step 6: Commit**

```bash
git add crates/slugline/Cargo.toml crates/slugline/src/cli.rs
git commit -m "feat(app): trimmed CLI (notes_dir, config) with resolve"
```

---

### Task 5: Minimal Iced app — render today's note read-only

**Files:**
- Create: `crates/slugline/src/app.rs`
- Modify: `crates/slugline/src/main.rs`

- [ ] **Step 1: Write the Iced `App`** (`crates/slugline/src/app.rs`)

```rust
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Font, Length, Task};

use slugline_core::store::NotesStore;

pub struct App {
    date: String,
    lines: Vec<String>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl App {
    /// Build the app by reading (or materializing) the note for `date`.
    pub fn new(store: &NotesStore, date: String) -> Self {
        match store.read_or_create(&date) {
            Ok(content) => Self {
                date,
                lines: content.lines().map(str::to_string).collect(),
                error: None,
            },
            Err(e) => Self {
                date,
                lines: Vec::new(),
                error: Some(format!("Failed to load note: {e}")),
            },
        }
    }

    pub fn title(&self) -> String {
        format!("Slugline — {}", self.date)
    }

    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut col = column![].spacing(2).padding(16);
        if let Some(err) = &self.error {
            col = col.push(text(err.clone()));
        }
        for line in &self.lines {
            // Render each raw line in monospace. Empty lines get a space so they keep height.
            let display = if line.is_empty() { " ".to_string() } else { line.clone() };
            col = col.push(text(display).font(Font::MONOSPACE));
        }
        scrollable(container(col).width(Length::Fill)).into()
    }
}
```

- [ ] **Step 2: Write `main.rs` that wires CLI → store → Iced** (`crates/slugline/src/main.rs`)

```rust
mod app;
mod cli;

use clap::Parser;
use iced::Task;

use slugline_core::config::{default_config_path, load_or_create};
use slugline_core::dates::today_iso;
use slugline_core::store::{ensure_writable_dir, NotesStore};

use crate::app::App;
use crate::cli::{resolve, Cli};

pub fn main() -> iced::Result {
    let cli = Cli::parse();

    let config_path = cli.config.clone().unwrap_or_else(default_config_path);
    let config = load_or_create(&config_path).unwrap_or_else(|e| {
        eprintln!("Failed to load config {}: {e}", config_path.display());
        std::process::exit(1);
    });

    let resolved = resolve(&cli, &config);
    if let Err(e) = ensure_writable_dir(&resolved.notes_dir) {
        eprintln!(
            "Notes directory {} is not usable: {e}",
            resolved.notes_dir.display()
        );
        std::process::exit(1);
    }

    let store = NotesStore::new(resolved.notes_dir);
    let date = today_iso();

    iced::application(App::title, App::update, App::view)
        .run_with(move || (App::new(&store, date.clone()), Task::none()))
}
```

- [ ] **Step 3: Verify the workspace builds**

Run: `cargo build`
Expected: compiles both crates with no errors (first Iced build downloads wgpu/winit — may take a few minutes).

- [ ] **Step 4: Manual smoke test**

Run: `cargo run -p slugline -- --notes-dir ./dev-notes`
Expected: a native window opens titled `Slugline — <today's date>`, showing today's note as raw monospace lines (the materialized template `# <date>-<WD>`, `## To Do`, `## Meetings`, `## Notes`). Closing the window exits the process cleanly. Confirm `./dev-notes/<today>.md` was created on disk.

- [ ] **Step 5: Commit**

```bash
git add crates/slugline/src/app.rs crates/slugline/src/main.rs
git commit -m "feat(app): minimal Iced window rendering today's note read-only"
```

---

### Task 6: Workspace hygiene gate

**Files:** none (verification only)

- [ ] **Step 1: Format check**

Run: `cargo fmt --all -- --check`
Expected: no diff. If it fails, run `cargo fmt --all` and re-run.

- [ ] **Step 2: Clippy**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: no warnings. (The empty `Message {}` enum in `app.rs` is fine; `update` is a no-op this phase.)

- [ ] **Step 3: Full test run**

Run: `cargo test --workspace`
Expected: all tests green (core: date/store/config/dates; app: cli).

- [ ] **Step 4: Commit any formatting fixups**

```bash
git add -A
git commit -m "chore: cargo fmt + clippy clean for phase 0" || echo "nothing to commit"
```

---

## Self-Review (performed while writing this plan)

- **Spec coverage:** Covers Section 1 (workspace split; core gets date/store/config; deps trimmed; CLI drops `--port`/`--no-open`) and the roadmap's Phase 0 "done" definition (window titled `Slugline — <today>` renders today's note; clean close).
- **Deferred deliberately:** pretty per-line rendering, editing, autosave, config-driven scroll position, and Roboto font embedding — all owned by Phase 1 (editing) or Phase 6 (fonts/theme). Phase 0 uses Iced's built-in `Font::MONOSPACE`.
- **Type consistency:** `Cli`/`Resolved`/`resolve` signatures match between `cli.rs` and `main.rs`; `App::new(&NotesStore, String)`, `App::title/update/view` match the `iced::application(...).run_with(...)` call. `today_iso() -> String` matches its use in `main.rs`.
- **Placeholder scan:** the only `todo!()` is an intentional red-phase stub in Task 4, replaced in the same task.
