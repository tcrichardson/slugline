# Phase 2: Rust File API + Server — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the runnable `axum` backend: a thin filesystem API over a notes directory (`GET /api/notes`, `GET/PUT /api/notes/{date}`, `GET /api/config`), TOML config with CLI overrides, atomic writes, materialize-on-open, strict date/path-traversal validation, startup validation, localhost-only bind with auto-open, and embedded-SPA serving via `rust-embed`.

**Architecture:** A library crate (`src/lib.rs`) holds pure, unit-testable modules (`date`, `store`, `config`, `cli`, `app`, `assets`); a thin binary (`src/main.rs`) wires them together. The server performs **no Markdown parsing** — it stores and serves raw `.md` files. Date strings are strictly validated (`YYYY-MM-DD` + real calendar date) and only ever joined with a fixed `{date}.md` filename, making path traversal impossible.

**Tech Stack:** Rust 2024, `axum` 0.8+ (note: 0.8 uses `{param}` path syntax), `tokio`, `serde`/`serde_json`, `toml`, `rust-embed`, `mime_guess`, `clap` (derive), `dirs`, `open`. Tests use `tempfile` + `tower` (`oneshot`).

---

## File Structure

| File | Responsibility |
|---|---|
| `src/lib.rs` | Library root; declares modules |
| `src/main.rs` | Binary: parse CLI → load config → validate dir → bind → auto-open → serve |
| `src/date.rs` | `is_valid_date`, `weekday_abbr` (pure, dep-free) |
| `src/store.rs` | `NotesStore` (path resolution, list, read-or-create, atomic write), `daily_template`, `ensure_writable_dir` |
| `src/config.rs` | `Config`/`ServerConfig`/`UiConfig`, defaults, TOML load-or-create, `expand_tilde`, `default_config_path` |
| `src/cli.rs` | `Cli` (clap), `resolve` (CLI > file > default precedence) |
| `src/assets.rs` | `rust-embed` of `web/dist`, SPA `static_handler` |
| `src/app.rs` | `AppState`, `build_router`, API handlers, router tests |
| `web/dist/index.html` | Committed placeholder so `rust-embed` compiles before the real frontend exists |
| `Makefile` | Adds `run`, `test`, `build`, `fmt` targets |

---

## Task 1: Convert to lib+bin, add dependencies, baseline build

**Files:**
- Create: `src/lib.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add runtime dependencies**

Run from the repo root:
```bash
cargo add tokio --features full
cargo add axum
cargo add serde --features derive
cargo add serde_json
cargo add toml
cargo add rust-embed
cargo add mime_guess
cargo add clap --features derive
cargo add dirs
cargo add open
```
Expected: `Cargo.toml` `[dependencies]` lists all of the above with current versions. (If `axum` resolves below `0.8`, bump it: `cargo add axum@^0.8` — the route syntax in this plan assumes `{param}`.)

- [ ] **Step 2: Add dev dependencies**

```bash
cargo add --dev tempfile
cargo add --dev tower --features util
```
Expected: `[dev-dependencies]` lists `tempfile` and `tower`.

- [ ] **Step 3: Create the library root `src/lib.rs`**

```rust
//! Slugline backend: a thin filesystem API over a notes directory.
//! Modules are declared as they are implemented.
```

- [ ] **Step 4: Replace `src/main.rs` with a minimal async entrypoint**

```rust
#[tokio::main]
async fn main() {
    println!("slugline (scaffold)");
}
```

- [ ] **Step 5: Verify the baseline builds**

Run:
```bash
cargo build
```
Expected: compiles successfully (unused-dependency warnings are acceptable at this stage).

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/main.rs
git commit -m "chore: add backend deps and split into lib+bin"
```

---

## Task 2: Date utilities

**Files:**
- Create: `src/date.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Declare the module in `src/lib.rs`**

Add this line under the doc comment in `src/lib.rs`:
```rust
pub mod date;
```

- [ ] **Step 2: Write the failing tests**

Create `src/date.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_real_dates_and_rejects_impossible_ones() {
        assert!(is_valid_date("2026-06-23"));
        assert!(is_valid_date("2024-02-29")); // leap year
        assert!(!is_valid_date("2026-02-30"));
        assert!(!is_valid_date("2026-13-01"));
        assert!(!is_valid_date("2026-00-10"));
        assert!(!is_valid_date("2026-6-23")); // not zero-padded
        assert!(!is_valid_date("not-a-date"));
        assert!(!is_valid_date("2026-06-23/../etc")); // traversal attempt
    }

    #[test]
    fn computes_weekday_abbreviation() {
        assert_eq!(weekday_abbr("2026-06-23"), "TUE");
        assert_eq!(weekday_abbr("2000-01-01"), "SAT");
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run:
```bash
cargo test --lib date
```
Expected: FAIL to compile — `is_valid_date`/`weekday_abbr` not found.

- [ ] **Step 4: Write the implementation (above the test module)**

Insert at the top of `src/date.rs`, before the `#[cfg(test)]` block:
```rust
/// Validate a strict `YYYY-MM-DD` string that is also a real calendar date.
pub fn is_valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let digits = b[0..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..10].iter().all(u8::is_ascii_digit);
    if !digits {
        return false;
    }
    let y: i32 = s[0..4].parse().unwrap();
    let m: u32 = s[5..7].parse().unwrap();
    let d: u32 = s[8..10].parse().unwrap();
    if !(1..=12).contains(&m) || d < 1 {
        return false;
    }
    d <= days_in_month(y, m)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Uppercase 3-letter weekday (e.g. "TUE") for a valid `YYYY-MM-DD` date.
/// Uses Sakamoto's algorithm. The caller must pass a date that `is_valid_date` accepts.
pub fn weekday_abbr(s: &str) -> &'static str {
    let y: i32 = s[0..4].parse().unwrap();
    let m: i32 = s[5..7].parse().unwrap();
    let d: i32 = s[8..10].parse().unwrap();
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yy = if m < 3 { y - 1 } else { y };
    let idx = (yy + yy / 4 - yy / 100 + yy / 400 + T[(m - 1) as usize] + d).rem_euclid(7);
    const NAMES: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
    NAMES[idx as usize]
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run:
```bash
cargo test --lib date
```
Expected: PASS — both date tests green.

- [ ] **Step 6: Commit**

```bash
git add src/date.rs src/lib.rs
git commit -m "feat: add date validation and weekday helper"
```

---

## Task 3: NotesStore (template, list, read-or-create, atomic write)

**Files:**
- Create: `src/store.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Declare the module in `src/lib.rs`**

Add:
```rust
pub mod store;
```

- [ ] **Step 2: Write the failing tests**

Create `src/store.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn template_has_title_with_weekday_and_sections() {
        let t = daily_template("2026-06-23");
        assert!(t.starts_with("# 2026-06-23-TUE\n"));
        assert!(t.contains("## To Do"));
        assert!(t.contains("## Meetings"));
        assert!(t.contains("## Notes"));
        assert!(t.ends_with('\n'));
    }

    #[test]
    fn path_for_rejects_invalid_dates() {
        let store = NotesStore::new(tempdir().unwrap().path().to_path_buf());
        assert!(store.path_for("2026-06-23").is_some());
        assert!(store.path_for("../secret").is_none());
        assert!(store.path_for("2026-13-01").is_none());
    }

    #[test]
    fn read_or_create_materializes_then_reuses() {
        let dir = tempdir().unwrap();
        let store = NotesStore::new(dir.path().to_path_buf());
        let first = store.read_or_create("2026-06-23").unwrap();
        assert!(first.starts_with("# 2026-06-23-TUE"));
        assert!(dir.path().join("2026-06-23.md").exists());

        // Mutate on disk, then ensure read returns the existing content (not a fresh template).
        store.write("2026-06-23", "# edited").unwrap();
        let second = store.read_or_create("2026-06-23").unwrap();
        assert_eq!(second, "# edited\n");
    }

    #[test]
    fn write_ensures_trailing_newline_atomically() {
        let dir = tempdir().unwrap();
        let store = NotesStore::new(dir.path().to_path_buf());
        store.write("2026-06-23", "no newline").unwrap();
        let content = std::fs::read_to_string(dir.path().join("2026-06-23.md")).unwrap();
        assert_eq!(content, "no newline\n");
        // No leftover temp file.
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn list_dates_filters_and_sorts() {
        let dir = tempdir().unwrap();
        let store = NotesStore::new(dir.path().to_path_buf());
        store.write("2026-06-23", "a").unwrap();
        store.write("2026-06-21", "b").unwrap();
        std::fs::write(dir.path().join("README.md"), "x").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "x").unwrap();
        assert_eq!(store.list_dates().unwrap(), vec!["2026-06-21", "2026-06-23"]);
    }

    #[test]
    fn ensure_writable_dir_creates_missing() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("notes");
        ensure_writable_dir(&nested).unwrap();
        assert!(nested.is_dir());
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run:
```bash
cargo test --lib store
```
Expected: FAIL to compile — `NotesStore`/`daily_template`/`ensure_writable_dir` not found.

- [ ] **Step 4: Write the implementation (above the test module)**

Insert at the top of `src/store.rs`:
```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::date::{is_valid_date, weekday_abbr};

/// The built-in daily note template (materialized on open).
pub fn daily_template(date: &str) -> String {
    format!(
        "# {date}-{wd}\n\n## To Do\n\n## Meetings\n\n## Notes\n",
        date = date,
        wd = weekday_abbr(date),
    )
}

/// Create `dir` if missing and verify it is writable (used for startup validation).
pub fn ensure_writable_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let probe = dir.join(".slugline-write-probe");
    fs::write(&probe, b"")?;
    fs::remove_file(&probe)?;
    Ok(())
}

#[derive(Clone)]
pub struct NotesStore {
    notes_dir: PathBuf,
}

impl NotesStore {
    pub fn new(notes_dir: PathBuf) -> Self {
        Self { notes_dir }
    }

    pub fn notes_dir(&self) -> &Path {
        &self.notes_dir
    }

    /// Resolve the on-disk path for a date, rejecting anything that is not a valid `YYYY-MM-DD`.
    pub fn path_for(&self, date: &str) -> Option<PathBuf> {
        if !is_valid_date(date) {
            return None;
        }
        Some(self.notes_dir.join(format!("{date}.md")))
    }

    /// List dates (`YYYY-MM-DD`) that have note files, sorted ascending.
    pub fn list_dates(&self) -> io::Result<Vec<String>> {
        let mut out = Vec::new();
        let rd = match fs::read_dir(&self.notes_dir) {
            Ok(rd) => rd,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(out),
            Err(e) => return Err(e),
        };
        for entry in rd {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if let Some(stem) = name.strip_suffix(".md") {
                if is_valid_date(stem) {
                    out.push(stem.to_string());
                }
            }
        }
        out.sort();
        Ok(out)
    }

    /// Read a note, materializing it from the template if it does not yet exist.
    pub fn read_or_create(&self, date: &str) -> io::Result<String> {
        let path = self
            .path_for(date)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid date"))?;
        match fs::read_to_string(&path) {
            Ok(content) => Ok(content),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let content = daily_template(date);
                self.write(date, &content)?;
                Ok(content)
            }
            Err(e) => Err(e),
        }
    }

    /// Atomically write a note (temp file + rename), ensuring a trailing newline.
    pub fn write(&self, date: &str, content: &str) -> io::Result<()> {
        let path = self
            .path_for(date)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid date"))?;
        fs::create_dir_all(&self.notes_dir)?;

        let mut body = content.to_string();
        if !body.ends_with('\n') {
            body.push('\n');
        }

        let tmp = self.notes_dir.join(format!(".{date}.md.tmp"));
        fs::write(&tmp, body.as_bytes())?;
        fs::rename(&tmp, &path)?;
        Ok(())
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run:
```bash
cargo test --lib store
```
Expected: PASS — all store tests green.

- [ ] **Step 6: Commit**

```bash
git add src/store.rs src/lib.rs
git commit -m "feat: add NotesStore with atomic writes and materialize-on-open"
```

---

## Task 4: Configuration

**Files:**
- Create: `src/config.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Declare the module in `src/lib.rs`**

Add:
```rust
pub mod config;
```

- [ ] **Step 2: Write the failing tests**

Create `src/config.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn defaults_apply_when_fields_missing() {
        let cfg = Config::from_toml("").unwrap();
        assert_eq!(cfg.server.port, 4747);
        assert_eq!(cfg.server.auto_open, true);
        assert_eq!(cfg.server.notes_dir, "~/Documents/Slugline");
        assert_eq!(cfg.ui.theme, "light");
        assert_eq!(cfg.ui.font, "Roboto");
        assert!((cfg.ui.edit_line_position - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_overrides() {
        let toml = r#"
            [server]
            port = 9000
            auto_open = false

            [ui]
            theme = "dark"

            [ui.colors.dark]
            "--bg" = "#101018"
        "#;
        let cfg = Config::from_toml(toml).unwrap();
        assert_eq!(cfg.server.port, 9000);
        assert_eq!(cfg.server.auto_open, false);
        assert_eq!(cfg.ui.theme, "dark");
        assert_eq!(cfg.ui.colors["dark"]["--bg"], "#101018");
    }

    #[test]
    fn expands_leading_tilde() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_tilde("~/Documents/Slugline"), home.join("Documents/Slugline"));
        assert_eq!(expand_tilde("/abs/path"), std::path::PathBuf::from("/abs/path"));
    }

    #[test]
    fn load_or_create_writes_default_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");
        let cfg = load_or_create(&path).unwrap();
        assert_eq!(cfg.server.port, 4747);
        assert!(path.exists());
        // Second load reads the file back.
        let again = load_or_create(&path).unwrap();
        assert_eq!(again.server.port, 4747);
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run:
```bash
cargo test --lib config
```
Expected: FAIL to compile — `Config` etc. not found.

- [ ] **Step 4: Write the implementation (above the test module)**

Insert at the top of `src/config.rs`:
```rust
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_notes_dir")]
    pub notes_dir: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub auto_open: bool,
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
fn default_port() -> u16 {
    4747
}
fn default_true() -> bool {
    true
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

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            notes_dir: default_notes_dir(),
            port: default_port(),
            auto_open: default_true(),
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

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            ui: UiConfig::default(),
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
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
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
            let toml = toml::to_string_pretty(&cfg)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            fs::write(path, toml)?;
            Ok(cfg)
        }
        Err(e) => Err(e),
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run:
```bash
cargo test --lib config
```
Expected: PASS — all config tests green.

- [ ] **Step 6: Commit**

```bash
git add src/config.rs src/lib.rs
git commit -m "feat: add TOML config with defaults, tilde expansion, load-or-create"
```

---

## Task 5: CLI args & precedence resolution

**Files:**
- Create: `src/cli.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Declare the module in `src/lib.rs`**

Add:
```rust
pub mod cli;
```

- [ ] **Step 2: Write the failing tests**

Create `src/cli.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[test]
    fn cli_overrides_take_precedence_over_config() {
        let cli = Cli {
            notes_dir: Some(PathBuf::from("/tmp/notes")),
            port: Some(9999),
            no_open: true,
            config: None,
        };
        let cfg = Config::default();
        let r = resolve(&cli, &cfg);
        assert_eq!(r.notes_dir, PathBuf::from("/tmp/notes"));
        assert_eq!(r.port, 9999);
        assert_eq!(r.auto_open, false);
    }

    #[test]
    fn falls_back_to_config_then_defaults() {
        let cli = Cli {
            notes_dir: None,
            port: None,
            no_open: false,
            config: None,
        };
        let cfg = Config::default();
        let r = resolve(&cli, &cfg);
        let home = dirs::home_dir().unwrap();
        assert_eq!(r.notes_dir, home.join("Documents/Slugline"));
        assert_eq!(r.port, 4747);
        assert_eq!(r.auto_open, true);
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run:
```bash
cargo test --lib cli
```
Expected: FAIL to compile — `Cli`/`resolve` not found.

- [ ] **Step 4: Write the implementation (above the test module)**

Insert at the top of `src/cli.rs`:
```rust
use std::path::PathBuf;

use clap::Parser;

use crate::config::{expand_tilde, Config};

#[derive(Parser, Debug, Default)]
#[command(name = "slugline", about = "Keyboard-driven daily notes")]
pub struct Cli {
    /// Override the notes directory.
    #[arg(long)]
    pub notes_dir: Option<PathBuf>,
    /// Override the listen port.
    #[arg(long)]
    pub port: Option<u16>,
    /// Do not auto-open the browser on launch.
    #[arg(long)]
    pub no_open: bool,
    /// Use a specific config file instead of the default.
    #[arg(long)]
    pub config: Option<PathBuf>,
}

/// The effective runtime settings after applying CLI > file > defaults precedence.
pub struct Resolved {
    pub notes_dir: PathBuf,
    pub port: u16,
    pub auto_open: bool,
}

pub fn resolve(cli: &Cli, config: &Config) -> Resolved {
    Resolved {
        notes_dir: cli
            .notes_dir
            .clone()
            .unwrap_or_else(|| expand_tilde(&config.server.notes_dir)),
        port: cli.port.unwrap_or(config.server.port),
        auto_open: if cli.no_open {
            false
        } else {
            config.server.auto_open
        },
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run:
```bash
cargo test --lib cli
```
Expected: PASS — both CLI tests green.

- [ ] **Step 6: Commit**

```bash
git add src/cli.rs src/lib.rs
git commit -m "feat: add CLI args and precedence resolution"
```

---

## Task 6: Embedded assets + SPA fallback + placeholder

**Files:**
- Create: `src/assets.rs`, `web/dist/index.html`
- Modify: `src/lib.rs`, `.gitignore`

- [ ] **Step 1: Create the placeholder SPA so `rust-embed` has a folder to embed**

Create `web/dist/index.html`:
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Slugline</title>
  </head>
  <body>
    <h1>Slugline</h1>
    <p>Backend is running. The frontend build will replace this placeholder.</p>
  </body>
</html>
```

- [ ] **Step 2: Ignore built frontend output but keep the placeholder tracked**

Append to `.gitignore`:
```
/web/dist
/web/node_modules
```

- [ ] **Step 3: Declare the module in `src/lib.rs`**

Add:
```rust
pub mod assets;
```

- [ ] **Step 4: Write the failing test**

Create `src/assets.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_index_is_embedded() {
        let asset = Assets::get("index.html").expect("index.html should be embedded");
        let html = String::from_utf8_lossy(&asset.data);
        assert!(html.contains("Slugline"));
    }
}
```

- [ ] **Step 5: Run the test to verify it fails**

Run:
```bash
cargo test --lib assets
```
Expected: FAIL to compile — `Assets` not found.

- [ ] **Step 6: Write the implementation (above the test module)**

Insert at the top of `src/assets.rs`:
```rust
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "web/dist"]
pub struct Assets;

/// Serve an embedded asset, falling back to `index.html` for SPA routes.
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            [(header::CONTENT_TYPE, mime.as_ref().to_owned())],
            content.data.into_owned(),
        )
            .into_response();
    }

    match Assets::get("index.html") {
        Some(content) => (
            [(header::CONTENT_TYPE, "text/html; charset=utf-8".to_owned())],
            content.data.into_owned(),
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}
```

- [ ] **Step 7: Run the test to verify it passes**

Run:
```bash
cargo test --lib assets
```
Expected: PASS — embedded placeholder found.

- [ ] **Step 8: Commit**

```bash
git add -f web/dist/index.html
git add src/assets.rs src/lib.rs .gitignore
git commit -m "feat: embed SPA assets with placeholder and SPA fallback handler"
```
Note: `-f` force-adds the placeholder because `/web/dist` is now git-ignored.

---

## Task 7: App state, router & API handlers

**Files:**
- Create: `src/app.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Declare the module in `src/lib.rs`**

Add:
```rust
pub mod app;
```

- [ ] **Step 2: Write the failing tests**

Create `src/app.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tower::ServiceExt;

    use crate::config::UiConfig;
    use crate::store::NotesStore;

    fn test_state(dir: PathBuf) -> SharedState {
        Arc::new(AppState {
            store: NotesStore::new(dir),
            ui: UiConfig::default(),
        })
    }

    async fn body_string(resp: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn get_note_materializes_and_returns_markdown() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(dir.path().to_path_buf()));
        let resp = app
            .oneshot(Request::get("/api/notes/2026-06-23").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp).await;
        assert!(body.starts_with("# 2026-06-23-TUE"));
        assert!(dir.path().join("2026-06-23.md").exists());
    }

    #[tokio::test]
    async fn get_note_rejects_bad_date() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(dir.path().to_path_buf()));
        let resp = app
            .oneshot(Request::get("/api/notes/not-a-date").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn put_then_list_round_trips() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(dir.path().to_path_buf()));

        let put = app
            .clone()
            .oneshot(
                Request::put("/api/notes/2026-06-23")
                    .body(Body::from("# hi\n"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(put.status(), StatusCode::NO_CONTENT);

        let list = app
            .oneshot(Request::get("/api/notes").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(body_string(list).await, "[\"2026-06-23\"]");
    }

    #[tokio::test]
    async fn config_endpoint_returns_ui_json() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(dir.path().to_path_buf()));
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
    async fn serves_spa_index_at_root() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(dir.path().to_path_buf()));
        let resp = app
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(body_string(resp).await.contains("Slugline"));
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run:
```bash
cargo test --lib app
```
Expected: FAIL to compile — `AppState`/`build_router`/`SharedState` not found.

- [ ] **Step 4: Write the implementation (above the test module)**

Insert at the top of `src/app.rs`:
```rust
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

use crate::config::UiConfig;
use crate::store::NotesStore;

pub struct AppState {
    pub store: NotesStore,
    pub ui: UiConfig,
}

pub type SharedState = Arc<AppState>;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/api/notes", get(list_notes))
        .route("/api/notes/{date}", get(get_note).put(put_note))
        .route("/api/config", get(get_config))
        .fallback(crate::assets::static_handler)
        .with_state(state)
}

async fn list_notes(State(state): State<SharedState>) -> impl IntoResponse {
    match state.store.list_dates() {
        Ok(dates) => Json(dates).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_note(State(state): State<SharedState>, Path(date): Path<String>) -> impl IntoResponse {
    if state.store.path_for(&date).is_none() {
        return (StatusCode::BAD_REQUEST, "invalid date").into_response();
    }
    match state.store.read_or_create(&date) {
        Ok(content) => (
            [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
            content,
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn put_note(
    State(state): State<SharedState>,
    Path(date): Path<String>,
    body: String,
) -> impl IntoResponse {
    if state.store.path_for(&date).is_none() {
        return (StatusCode::BAD_REQUEST, "invalid date").into_response();
    }
    match state.store.write(&date, &body) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_config(State(state): State<SharedState>) -> impl IntoResponse {
    Json(state.ui.clone())
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run:
```bash
cargo test --lib app
```
Expected: PASS — all five router tests green.

- [ ] **Step 6: Run the full test suite**

Run:
```bash
cargo test
```
Expected: PASS — date, store, config, cli, assets, and app suites all green.

- [ ] **Step 7: Commit**

```bash
git add src/app.rs src/lib.rs
git commit -m "feat: add app state, router, and file API handlers"
```

---

## Task 8: Wire up `main`, startup validation, auto-open + Makefile

**Files:**
- Modify: `src/main.rs`, `Makefile`

- [ ] **Step 1: Replace `src/main.rs` with the full wiring**

```rust
use std::net::Ipv4Addr;
use std::process;
use std::sync::Arc;

use clap::Parser;

use slugline::app::{build_router, AppState};
use slugline::cli::{resolve, Cli};
use slugline::config::{default_config_path, load_or_create};
use slugline::store::{ensure_writable_dir, NotesStore};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config_path = cli.config.clone().unwrap_or_else(default_config_path);
    let config = match load_or_create(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config {}: {e}", config_path.display());
            process::exit(1);
        }
    };

    let resolved = resolve(&cli, &config);

    if let Err(e) = ensure_writable_dir(&resolved.notes_dir) {
        eprintln!(
            "Notes directory {} is not usable: {e}",
            resolved.notes_dir.display()
        );
        process::exit(1);
    }

    let state = Arc::new(AppState {
        store: NotesStore::new(resolved.notes_dir.clone()),
        ui: config.ui.clone(),
    });
    let app = build_router(state);

    let listener = match tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, resolved.port)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind 127.0.0.1:{}: {e}", resolved.port);
            process::exit(1);
        }
    };

    let url = format!("http://127.0.0.1:{}", resolved.port);
    println!(
        "Slugline serving at {url}  (notes: {})",
        resolved.notes_dir.display()
    );

    if resolved.auto_open {
        let _ = open::that(&url);
    }

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {e}");
        process::exit(1);
    }
}
```

- [ ] **Step 2: Verify it builds**

Run:
```bash
cargo build
```
Expected: compiles successfully.

- [ ] **Step 3: Extend the `Makefile` with backend targets**

Replace the contents of `Makefile` with:
```makefile
.PHONY: dev run test test-web fmt fmt-web build

# Run the backend (serves the embedded SPA + API)
run:
	cargo run

# Backend dev with a throwaway notes dir and no browser auto-open
dev:
	cargo run -- --notes-dir ./dev-notes --no-open

test:
	cargo test

test-web:
	cd web && npm test

fmt:
	cargo fmt

fmt-web:
	cd web && npx prettier --write "src/**/*.{ts,svelte}"

# Production build: frontend bundle (Vite default outDir is web/dist) then release binary
build:
	cd web && npm run build
	cargo build --release
```
(Recipe lines must be **tab**-indented — a Make requirement.)

- [ ] **Step 4: Manually verify the running server end-to-end**

Run the server against a throwaway notes directory (no browser):
```bash
cargo run -- --notes-dir /tmp/slugline-test --no-open --port 4747
```
In a second terminal, exercise the API:
```bash
curl -s http://127.0.0.1:4747/api/notes
curl -s http://127.0.0.1:4747/api/notes/2026-06-23 | head -1
curl -s http://127.0.0.1:4747/api/notes
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:4747/api/notes/not-a-date
curl -s http://127.0.0.1:4747/api/config
curl -s http://127.0.0.1:4747/ | grep -o Slugline
```
Expected:
- First `/api/notes` → `[]`
- `/api/notes/2026-06-23` first line → `# 2026-06-23-TUE`
- Second `/api/notes` → `["2026-06-23"]` (now materialized)
- bad-date request → `400`
- `/api/config` → JSON containing `"theme":"light"`
- `/` → `Slugline`

Then stop the server (Ctrl-C) and confirm the file exists:
```bash
ls /tmp/slugline-test
```
Expected: `2026-06-23.md` present.

- [ ] **Step 5: Commit**

```bash
git add src/main.rs Makefile
git commit -m "feat: wire up main with startup validation, localhost bind, and auto-open"
```

---

## Phase 2 Done Criteria

- `cargo test` is green across `date`, `store`, `config`, `cli`, `assets`, `app`.
- `cargo run -- --notes-dir <dir> --no-open` serves the API; the manual `curl` round-trip in Task 8 behaves as specified.
- Invalid dates return `400`; traversal attempts never reach the filesystem (rejected by `is_valid_date`).
- A missing config file is created with defaults at `~/.config/slugline/config.toml`; CLI flags override it.
- The binary serves the placeholder SPA at `/` and is ready for the real Vite bundle (`web/dist`) in later phases.

## Self-Review Notes (performed during authoring)

- **Spec coverage (roadmap Phase 2 row):** `GET /api/notes` (list_notes), `GET/PUT /api/notes/{date}` (get_note/put_note, materialize-on-open, atomic write), `GET /api/config` (get_config, UI subset), strict date + traversal rejection (`is_valid_date` + `path_for`), TOML config (config.rs), startup dir validation (ensure_writable_dir + main), `rust-embed` SPA serving (assets.rs), `--notes-dir`/`--port`/`--no-open` (cli.rs), localhost-only bind + default 4747 + auto-open (main.rs), Rust unit tests over temp dir — all present.
- **Type consistency:** `NotesStore` API (`path_for`, `list_dates`, `read_or_create`, `write`) is defined once in Task 3 and consumed unchanged by handlers in Task 7. `UiConfig` defined in Task 4 is reused by `AppState` and the `/api/config` handler. `Cli`/`resolve`/`Resolved` defined in Task 5 are consumed by `main` in Task 8. `Assets`/`static_handler` defined in Task 6 are referenced by `build_router` in Task 7 (assets task ordered first to satisfy the dependency).
- **Version caveat:** `axum` 0.8 path syntax `{date}` is used; if `cargo add` resolves an older axum, pin `axum@^0.8`.
- **No placeholders:** every step contains complete code and exact commands with expected output.
```
