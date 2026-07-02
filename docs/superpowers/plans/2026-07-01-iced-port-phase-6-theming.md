# Phase 6 — Theming & Polish — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **This is a port, plus one shell-level refactor with no web counterpart.** Behavioral truth for
> the ported pieces is `web/src/lib/theme.ts` (+ `theme.test.ts`),
> `web/src/lib/components/StatusLine.svelte`, and `web/src/lib/components/Toast.svelte`. The
> **dynamic per-widget theming plumbing** itself (replacing the `ui::palette` module's
> hardcoded-dark-only `Color` constants with a `Palette` value threaded through every
> `ui::*::view` function) has no web counterpart — CSS custom properties gave the web app dynamic
> theming for free, so this port has to build the equivalent machinery from scratch. Same
> rationale as Phase 3's `shift_month` and Phase 5's `fuzzy_score`: fresh implementation, fresh
> tests.
>
> **Iced API caution:** method/type names target iced `0.13.x` (`container::Style`,
> `button::Style`/`Status`, `stack!`, `Padding`'s array-literal conversions). Every non-trivial
> API call in this plan was implemented in a disposable scratch worktree off the real `master` tip
> (`d920d43 "fix(keys): apply shift modifier to character keys"`), compiled, tested (`cargo test
> --workspace`: 164 `slugline-core` tests + 51 `slugline` tests, all green, up from 158 + 37),
> formatted, linted (`cargo clippy --workspace --all-targets -- -D warnings`: clean), and
> smoke-run (`cargo run -p slugline`, confirmed the window opens, a default `config.toml` with
> `theme = "light"` is written, today's note materializes, and there is no panic) before being
> copied into this document — but if a signature has drifted in your checkout, confirm it and
> adjust; the *intent* is the contract.

**Goal:** Make the theme switchable at runtime: port `theme.ts`'s light/dark token resolution into
`core`, replace the UI's hardcoded-dark-only color constants with a `Palette` computed from the
active theme, wire `:theme`/`:theme dark`/`:theme light` to actually swap it and persist the choice
via the existing comment-preserving `toml_edit` writer (with rollback on a failed write), and add
the two still-missing shell pieces — a status line (mode/context/message) and an error toast with
5s auto-dismiss.

**Architecture:** One new `slugline-core` module, `theme` (pure token resolution: `light()`/
`dark()`/`resolve_tokens()`/`next_theme()`, string-keyed exactly like the web's CSS-custom-property
map so config's existing `ui.colors: BTreeMap<String, BTreeMap<String, String>>` overrides merge
in without `core` needing to know every key in advance). One new `slugline` module,
`theme_iced::Palette` (parses those hex-string tokens into `iced::Color`s once per theme change,
not per frame). Every `ui::*::view` function is refactored to take `&Palette` as a parameter
instead of reaching for the old `ui::palette` module's constants (which is deleted). `App` gains
`theme`/`color_overrides`/`palette`/`config_path` fields, a `switch_theme` handler for
`AppEffect::Theme` (optimistic apply + a persistence `Task` that rolls back on failure), and
`error_expires_at` for the toast's 5s auto-dismiss (driven by the existing 250ms `Tick`
subscription). Two new UI-only files, `ui/status_line.rs` and `ui/toast.rs`, complete the main
pane and the `view()` overlay stack.

**Tech Stack:** Rust, existing `slugline-core::config` (`UiConfig`, `update_theme` — both already
ported in Phase 0, untouched by this phase), Iced `0.13.x` `container::Style`/`button::Style`
closures for per-widget coloring, `stack!` for the toast overlay (same mechanism Phase 5 used for
the command palette). No new crate dependencies.

---

## Prerequisites

- **Phases 0, 1a, 1b, 1c, 2, 3, 4, 5 are complete and committed on `master`, and `cargo test
  --workspace` is green** (158 `slugline-core` tests + 37 tests in `slugline`, as of `d920d43 "fix(keys):
  apply shift modifier to character keys"`). Phase 6 builds directly on
  `crates/slugline-core/src/config.rs`'s `UiConfig { theme, font, edit_line_position, colors }` and
  `update_theme(path, theme)` (both already ported, untouched here),
  `crates/slugline-core/src/editor/keymap.rs`'s `AppEffect::Theme(String)` (emitted by
  `run_command`'s `CommandName::Theme` branch since Phase 5 — `crates/slugline/src/app.rs`'s
  `run_effect` currently has `AppEffect::Theme(_) => Task::none(), // wired in Phase 6`, which this
  phase replaces), and every `ui/*.rs` file's existing hardcoded `ui::palette` constants (dark-only
  values copied from `web/src/lib/theme.ts`'s `DARK`), which this phase deletes in favor of a
  runtime-switchable `Palette`.

## Scope

**In this phase:**
- `core::theme`: `Tokens` (a `BTreeMap<String, String>` type alias — deliberately not a fixed
  struct, so config's free-form `ui.colors` overrides merge by string key without `core` needing a
  field for every possible override), `light()`, `dark()`, `builtin_tokens(theme)`,
  `next_theme(theme)`, `resolve_tokens(theme, overrides)` — a port of `web/src/lib/theme.ts`
  (`applyTheme` has no port: it's a DOM side effect with no Iced equivalent, same "UI-adapter
  boundary is untested/unported" reasoning the design applies to `EditorPane.svelte`).
- `theme_iced::Palette` (new file, `slugline` crate): one `iced::Color` field per rendering concern
  the old `ui::palette` module had, plus `parse_hex` (total, never panics on malformed
  config-supplied strings) and `Palette::for_theme(theme, overrides)` — the one entry point that
  resolves tokens and converts them.
- Deleting `ui/palette.rs` and threading `&Palette` through every `ui::*::view` function
  (`editor_pane`, `tab_strip`, `calendar`, `agenda`, `todo_list`, `sidebar`, `command_palette`).
  `tab_strip` additionally gains real styling for the first time (previously "intentionally
  default" per its Phase 2 comment): active tab gets `--edit-bar-bg` background + `--fg` text,
  matching `web/src/lib/components/Tabs.svelte`.
- `App` gains `config_path: PathBuf`, `theme: String`, `color_overrides: BTreeMap<String, Tokens>`,
  `palette: Palette`, and `error_expires_at: Option<Instant>`; `App::new` gains `ui_config:
  UiConfig` and `config_path: PathBuf` parameters, wired from `main.rs`, which now actually reads
  `config.ui` instead of discarding it.
- `App::run_effect`'s `AppEffect::Theme(arg)` branch: a new `switch_theme` method applies the
  target theme (`next_theme` when `arg` is empty, i.e. bare `:theme`; otherwise `arg` verbatim —
  already validated to be `""`/`"light"`/`"dark"` by Phase 5's `validate_command`) optimistically,
  then persists it via `update_theme` in a `Task`; a new `Message::ThemePersisted` handles the
  result, rolling back `theme`/`palette` to the previous value and surfacing a toast on failure
  (design Section 6), but only if the user hasn't since switched again.
- `ui/status_line.rs` (new): mode label (`-- NORMAL --`/`-- INSERT --`) + a cursor-context
  breadcrumb (via the already-ported `core::doc::context::resolve_context`) on the left, the
  editor's `message` right-aligned — port of `web/src/lib/components/StatusLine.svelte`. Wired into
  `App::main_pane`, below the editor pane.
- `ui/toast.rs` (new) + `Message::DismissError`: a fixed bottom-center error toast with a dismiss
  button — port of `web/src/lib/components/Toast.svelte` (including its hardcoded, non-themed red).
  `App::error`/`error_expires_at` get a `set_error` helper (message + 5s expiry) and the existing
  250ms `Tick` subscription clears an expired error. Rendered via `stack!` on top of everything
  else, including the command palette, whenever `error.is_some()`.

**Deferred on purpose:**
- **OS light/dark following** (a `:theme system` mode). Explicitly deferred in design Section 4
  ("needs a helper like `dark-light`; ... Explicit light/dark ships first") — this phase implements
  exactly that explicit light/dark switching, nothing more.
- **`iced::Theme::Custom` integration.** The design sketches "The UI builds an `iced::Theme::Custom`
  palette plus a styling module that reads tokens for per-widget colors/borders." Every `ui/*.rs`
  widget already ignores the `_theme: &iced::Theme` argument Iced's `button`/`container` style
  closures receive, sourcing colors from the old hardcoded `ui::palette` constants instead — that
  pattern predates this phase (Phases 2-5) and this phase's `Palette` is a mechanical,
  minimal-diff generalization of it (a value instead of a constant), not a rewrite onto Iced's own
  theme system. Revisit only if a real need for Iced-native theme propagation (e.g. third-party
  widgets that *do* read `_theme`) comes up.
- **Live-reload of `ui.colors`/`theme` when `config.toml` is edited externally** (outside the app,
  while it's running). The app only ever changes its own theme via `:theme`, which already updates
  `self.palette` in memory *and* writes the file — there is no scenario where the file the app just
  wrote diverges from `self.theme`, so a file-watcher would only matter for a human hand-editing
  `config.toml` mid-session, which is out of scope (same "don't add untested secondary
  interactions" reasoning prior phases used).
- **A dedicated light/dark toggle button or keybinding** beyond `:theme`/`:theme light`/`:theme
  dark` (already reachable via Phase 5's command palette, including ⌘K). The roadmap's "`:theme`
  switch" is satisfied by the command; a redundant button is unrequested surface area.
- **Config override keys with no `Palette` field** (e.g. a typo'd `--bgg`, or a real web token this
  port never added a field for, like `--edit-line-bg`, which nothing in the Iced UI currently
  renders a background for). `Palette::from_tokens` only reads the fixed set of keys it has fields
  for; unknown keys in `resolve_tokens`'s output are silently unused. This mirrors the fixed,
  compile-time-known set of rendering concerns the Iced UI actually has — `core::theme` stays
  fully string-keyed/config-compatible either way.

---

## File Structure (files added/changed in Phase 6)

```
crates/slugline-core/
  src/
    lib.rs                          # + pub mod theme;
    theme.rs                        # NEW: Tokens, light(), dark(), builtin_tokens(), next_theme(),
                                     #      resolve_tokens() (tested)

crates/slugline/
  src/
    main.rs                         # + mod theme_iced; pass config.ui + config_path into App::new
    theme_iced.rs                   # NEW: Palette, parse_hex(), Palette::for_theme() (tested)
    app.rs                          # REWRITE: theme/color_overrides/palette/config_path/
                                     #          error_expires_at fields, switch_theme(), set_error(),
                                     #          Message::{ThemePersisted,DismissError}, status_line +
                                     #          toast wiring in view()/main_pane()
    ui/mod.rs                       # + pub mod status_line; pub mod toast; - pub mod palette;
    ui/palette.rs                   # DELETED: superseded by theme_iced::Palette
    ui/editor_pane.rs               # REWRITE: view()/active_line()/pretty_line()/inline() take &Palette
    ui/tab_strip.rs                 # REWRITE: view() takes &Palette; active/inactive styling added
    ui/calendar.rs                  # REWRITE: view()/day_cell() take &Palette
    ui/agenda.rs                    # REWRITE: view()/agenda_row() take &Palette
    ui/todo_list.rs                 # REWRITE: view()/group_view()/todo_row() take &Palette
    ui/sidebar.rs                   # REWRITE: view() takes &Palette, threads it to calendar/agenda/todo_list
    ui/command_palette.rs           # REWRITE: view()/suggestion_row() take &Palette
    ui/status_line.rs               # NEW: mode + context breadcrumb + message (tested)
    ui/toast.rs                     # NEW: fixed bottom-center error toast + dismiss button
```

---

### Task 1: Port `theme.ts` into `core::theme`

**Files:**
- Create: `crates/slugline-core/src/theme.rs`
- Modify: `crates/slugline-core/src/lib.rs`

- [ ] **Step 1: Write the failing tests** — create `crates/slugline-core/src/theme.rs`:

```rust
use std::collections::BTreeMap;

/// A resolved set of color tokens, keyed by the same `--token-name` strings the web app
/// used as CSS custom properties (e.g. `--bg`, `--heading-1`). Kept as a string-keyed map
/// (not a fixed struct) so config's free-form per-theme overrides
/// (`UiConfig::colors: BTreeMap<String, BTreeMap<String, String>>`) can merge over it by
/// name without `core` needing to know about every override key in advance. Port of
/// `web/src/lib/theme.ts`'s `Tokens`.
pub type Tokens = BTreeMap<String, String>;

fn tokens(pairs: &[(&str, &str)]) -> Tokens {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// The built-in light palette. Port of `web/src/lib/theme.ts`'s `LIGHT`.
pub fn light() -> Tokens {
    tokens(&[
        ("--bg", "#fbfcfe"),
        ("--fg", "#1b2330"),
        ("--muted", "#5b6675"),
        ("--accent", "#2f6df6"),
        ("--heading-1", "#1d4ed8"),
        ("--heading-2", "#2563eb"),
        ("--heading-3", "#3b82f6"),
        ("--heading-4", "#60a5fa"),
        ("--heading-5", "#7dabfb"),
        ("--heading-6", "#9cc2fc"),
        ("--todo-done", "#8a93a3"),
        ("--meta", "#6b7686"),
        ("--status-bar", "#eef2f9"),
        ("--edit-line-bg", "#eaf1ff"),
        ("--edit-bar-bg", "#e2ebff"),
        ("--rule", "#d9e0ec"),
        ("--cursor", "#1b2330"),
        ("--blockquote-border", "#93c5fd"),
        ("--highlight-bg", "#fef08a"),
    ])
}

/// The built-in dark palette. Port of `web/src/lib/theme.ts`'s `DARK`. `web/`'s in-code
/// `DARK` object omits `--todo-done` (only `LIGHT` and `web/src/app.css`'s `:root` default
/// define it, both `#8a93a3`) — the web app never overrides it for dark mode, so the
/// rendered color is always that same muted gray regardless of theme. This port makes
/// that explicit rather than relying on a CSS-cascade accident: `--todo-done` is included
/// here with the identical value.
pub fn dark() -> Tokens {
    tokens(&[
        ("--bg", "#161a26"),
        ("--fg", "#e7ecf5"),
        ("--muted", "#97a1b3"),
        ("--accent", "#2f6df6"),
        ("--heading-1", "#1d4ed8"),
        ("--heading-2", "#3b82f6"),
        ("--heading-3", "#60a5fa"),
        ("--heading-4", "#7dabfb"),
        ("--heading-5", "#9cc2fc"),
        ("--heading-6", "#9cc2fc"),
        ("--todo-done", "#8a93a3"),
        ("--meta", "#97a1b3"),
        ("--status-bar", "#1f2535"),
        ("--edit-line-bg", "#222a3d"),
        ("--edit-bar-bg", "#2a344c"),
        ("--rule", "#2d3650"),
        ("--cursor", "#e7ecf5"),
        ("--blockquote-border", "#3b82f6"),
        ("--highlight-bg", "#713f12"),
    ])
}

/// The built-in tokens for `theme` (anything other than `"dark"` is treated as light).
/// Port of `web/src/lib/theme.ts`'s `builtinTokens`.
pub fn builtin_tokens(theme: &str) -> Tokens {
    if theme == "dark" { dark() } else { light() }
}

/// The opposite of `theme` (anything not `"dark"` flips to `"dark"`). Port of
/// `web/src/lib/theme.ts`'s `nextTheme`.
pub fn next_theme(theme: &str) -> String {
    if theme == "dark" {
        "light".to_string()
    } else {
        "dark".to_string()
    }
}

/// Merge the built-in tokens for `theme` with `overrides[theme]` (config's per-theme
/// color overrides), overrides winning. Port of `web/src/lib/theme.ts`'s
/// `resolveTokens`.
pub fn resolve_tokens(theme: &str, overrides: &BTreeMap<String, Tokens>) -> Tokens {
    let mut result = builtin_tokens(theme);
    if let Some(over) = overrides.get(theme) {
        for (k, v) in over {
            result.insert(k.clone(), v.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_built_in_light_tokens_by_default() {
        assert_eq!(
            resolve_tokens("light", &BTreeMap::new())["--bg"],
            light()["--bg"]
        );
    }

    #[test]
    fn returns_dark_tokens_for_the_dark_theme() {
        assert_eq!(
            resolve_tokens("dark", &BTreeMap::new())["--bg"],
            dark()["--bg"]
        );
    }

    #[test]
    fn falls_back_to_light_for_unknown_themes() {
        assert_eq!(
            resolve_tokens("neon", &BTreeMap::new())["--bg"],
            light()["--bg"]
        );
    }

    #[test]
    fn applies_per_theme_config_overrides_over_the_base() {
        let mut overrides = BTreeMap::new();
        overrides.insert("dark".to_string(), tokens(&[("--bg", "#000000")]));
        let t = resolve_tokens("dark", &overrides);
        assert_eq!(t["--bg"], "#000000");
        assert_eq!(t["--fg"], dark()["--fg"]);
    }

    #[test]
    fn defines_the_rule_and_edit_bar_tokens_for_both_themes() {
        for t in [light(), dark()] {
            assert!(t["--rule"].starts_with('#'));
            assert!(t["--edit-bar-bg"].starts_with('#'));
        }
    }

    #[test]
    fn next_theme_flips_dark_to_light_and_anything_else_to_dark() {
        assert_eq!(next_theme("dark"), "light");
        assert_eq!(next_theme("light"), "dark");
        assert_eq!(next_theme("whatever"), "dark");
    }
}
```

- [ ] **Step 2: Declare the module** — in `crates/slugline-core/src/lib.rs`, replace:

```rust
//! Slugline core: headless domain logic (no UI framework dependency).
pub mod agenda;
pub mod config;
pub mod date;
pub mod dates;
pub mod doc;
pub mod editor;
pub mod store;
pub mod tabs;
pub mod todos;
```

with:

```rust
//! Slugline core: headless domain logic (no UI framework dependency).
pub mod agenda;
pub mod config;
pub mod date;
pub mod dates;
pub mod doc;
pub mod editor;
pub mod store;
pub mod tabs;
pub mod theme;
pub mod todos;
```

- [ ] **Step 3: Run the tests** — `cargo test -p slugline-core theme::`
Expected: PASS (6 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/slugline-core/src/theme.rs crates/slugline-core/src/lib.rs
git commit -m "feat(core): port resolveTokens/nextTheme"
```

---

### Task 2: Add `theme_iced::Palette` (hex-string tokens -> Iced colors)

**Files:**
- Create: `crates/slugline/src/theme_iced.rs`
- Modify: `crates/slugline/src/main.rs` (module declaration only — the `App::new` wiring is Task 4)

- [ ] **Step 1: Write the failing tests** — create `crates/slugline/src/theme_iced.rs`:

```rust
//! Adapts `slugline_core::theme` tokens (hex strings) into ready-to-render Iced colors.
//! Replaces the old `ui::palette` module (a fixed set of dark-only constants) now that
//! the active theme can change at runtime via `:theme`.

use std::collections::BTreeMap;

use iced::Color;

use slugline_core::theme::{Tokens, resolve_tokens};

/// One `Color` per rendering concern used across `ui/*`. Computed once per theme change
/// (not per-frame) and stored on `App`; every `ui::*::view` function takes `&Palette`
/// instead of reaching for a global constant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub bg: Color,
    pub fg: Color,
    pub muted: Color,
    pub accent: Color,
    pub heading: [Color; 6],
    pub todo_done: Color,
    pub meta: Color,
    pub status_bar: Color,
    pub edit_bar_bg: Color,
    pub rule: Color,
    pub cursor: Color,
    pub blockquote_border: Color,
    pub highlight_bg: Color,
}

/// Parse a `#rrggbb` hex string into a Color. Falls back to opaque black on any
/// malformed input (missing `#`, wrong length, non-hex digits) — config-supplied
/// override strings are untrusted, so this must never panic.
fn parse_hex(s: &str) -> Color {
    let s = s.strip_prefix('#').unwrap_or(s);
    let byte = |i: usize| s.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok());
    match (s.len() == 6, byte(0), byte(2), byte(4)) {
        (true, Some(r), Some(g), Some(b)) => Color::from_rgb8(r, g, b),
        _ => Color::BLACK,
    }
}

fn token(tokens: &Tokens, key: &str) -> Color {
    tokens
        .get(key)
        .map(|v| parse_hex(v))
        .unwrap_or(Color::BLACK)
}

impl Palette {
    fn from_tokens(tokens: &Tokens) -> Self {
        Self {
            bg: token(tokens, "--bg"),
            fg: token(tokens, "--fg"),
            muted: token(tokens, "--muted"),
            accent: token(tokens, "--accent"),
            heading: [
                token(tokens, "--heading-1"),
                token(tokens, "--heading-2"),
                token(tokens, "--heading-3"),
                token(tokens, "--heading-4"),
                token(tokens, "--heading-5"),
                token(tokens, "--heading-6"),
            ],
            todo_done: token(tokens, "--todo-done"),
            meta: token(tokens, "--meta"),
            status_bar: token(tokens, "--status-bar"),
            edit_bar_bg: token(tokens, "--edit-bar-bg"),
            rule: token(tokens, "--rule"),
            cursor: token(tokens, "--cursor"),
            blockquote_border: token(tokens, "--blockquote-border"),
            highlight_bg: token(tokens, "--highlight-bg"),
        }
    }

    /// Resolve `theme`'s tokens (built-ins + `overrides[theme]`) into a ready-to-render
    /// `Palette`. The one entry point `App` calls on boot and after every theme switch.
    pub fn for_theme(theme: &str, overrides: &BTreeMap<String, Tokens>) -> Self {
        Self::from_tokens(&resolve_tokens(theme, overrides))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_hex_color() {
        assert_eq!(parse_hex("#1b2330"), Color::from_rgb8(0x1b, 0x23, 0x30));
        assert_eq!(parse_hex("1b2330"), Color::from_rgb8(0x1b, 0x23, 0x30));
    }

    #[test]
    fn falls_back_to_black_on_malformed_input() {
        assert_eq!(parse_hex("not-a-color"), Color::BLACK);
        assert_eq!(parse_hex("#fff"), Color::BLACK); // 3-digit shorthand unsupported
        assert_eq!(parse_hex(""), Color::BLACK);
    }

    #[test]
    fn for_theme_resolves_light_and_dark_distinctly() {
        let light = Palette::for_theme("light", &BTreeMap::new());
        let dark = Palette::for_theme("dark", &BTreeMap::new());
        assert_ne!(light.bg, dark.bg);
    }

    #[test]
    fn for_theme_applies_a_config_override() {
        let mut overrides = BTreeMap::new();
        let mut dark_overrides = Tokens::new();
        dark_overrides.insert("--bg".to_string(), "#000000".to_string());
        overrides.insert("dark".to_string(), dark_overrides);
        let p = Palette::for_theme("dark", &overrides);
        assert_eq!(p.bg, Color::BLACK);
    }
}
```

- [ ] **Step 2: Declare the module** — in `crates/slugline/src/main.rs`, replace:

```rust
mod app;
mod cli;
mod keys;
mod ui;
```

with:

```rust
mod app;
mod cli;
mod keys;
mod theme_iced;
mod ui;
```

- [ ] **Step 3: Run the tests** — `cargo test -p slugline theme_iced::`
Expected: PASS (4 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/slugline/src/theme_iced.rs crates/slugline/src/main.rs
git commit -m "feat(app): add theme_iced::Palette"
```

---

### Task 3: Delete `ui::palette`; thread `&Palette` through every `ui::*::view`

**Files:**
- Delete: `crates/slugline/src/ui/palette.rs`
- Modify: `crates/slugline/src/ui/mod.rs`, `ui/editor_pane.rs`, `ui/tab_strip.rs`,
  `ui/calendar.rs`, `ui/agenda.rs`, `ui/todo_list.rs`, `ui/sidebar.rs`, `ui/command_palette.rs`

This task's seven files only compile together (each takes a `&Palette` its caller must also pass),
so — like Phase 3's Task 2/4 split — expect `cargo build -p slugline` to fail until every file in
this task is done. It does **not** yet touch `app.rs`'s call sites (`sidebar::view(...)`,
`editor_pane::view(...)`, etc.) or `App`'s fields — that's Task 4. Do this whole task, then Task 4,
then build.

- [ ] **Step 1: Delete the old palette module**

```bash
rm crates/slugline/src/ui/palette.rs
```

- [ ] **Step 2: Update `ui/mod.rs`** — replace:

```rust
pub mod agenda;
pub mod calendar;
pub mod command_palette;
pub mod editor_pane;
pub mod palette;
pub mod sidebar;
pub mod tab_strip;
pub mod todo_list;
```

with:

```rust
pub mod agenda;
pub mod calendar;
pub mod command_palette;
pub mod editor_pane;
pub mod sidebar;
pub mod status_line;
pub mod tab_strip;
pub mod todo_list;
pub mod toast;
```

(`status_line`/`toast` don't exist until Tasks 6/7 — this line is added now so it only needs
touching once; `cargo build` won't reach them until those files exist, since this task's own
seven-file build failure comes first.)

- [ ] **Step 3: Rewrite `ui/editor_pane.rs`** — replace the whole file:

```rust
use iced::font::{Style as FontStyle, Weight};
use iced::widget::{column, container, rich_text, row, scrollable, span, text};
use iced::{Element, Font, Length};

use slugline_core::doc::{Line, Span, classify_line, render_inline};
use slugline_core::editor::{EditorState, Mode};

use crate::theme_iced::Palette;

const MONO: Font = Font::MONOSPACE;

pub fn view<'a, Message: Clone + 'static>(
    editor: &'a EditorState,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let mut col = column![].padding([16, 24]).spacing(2).width(Length::Fill);
    for (i, line) in editor.lines.iter().enumerate() {
        if i == editor.cursor.line {
            col = col.push(active_line(line, editor.cursor.col, editor.mode, palette));
        } else {
            col = col.push(pretty_line(line, palette));
        }
    }
    scrollable(col).height(Length::Fill).into()
}

fn active_line<'a, Message: Clone + 'static>(
    line: &str,
    col: usize,
    mode: Mode,
    palette: &Palette,
) -> Element<'a, Message> {
    let chars: Vec<char> = line.chars().collect();
    let col = col.min(chars.len());
    let before: String = chars[..col].iter().collect();
    let cursor_char: String = chars
        .get(col)
        .map(|c| c.to_string())
        .unwrap_or_else(|| " ".into());
    let after: String = if col < chars.len() {
        chars[col + 1..].iter().collect()
    } else {
        String::new()
    };

    let cursor_color = palette.cursor;
    let bg_color = palette.bg;
    let cursor: Element<'a, Message> = match mode {
        Mode::Normal => container(text(cursor_char).font(MONO).color(bg_color))
            .style(move |_| container::Style {
                background: Some(cursor_color.into()),
                ..container::Style::default()
            })
            .into(),
        Mode::Insert => row![
            container(text(""))
                .width(2)
                .height(Length::Fixed(18.0))
                .style(move |_| container::Style {
                    background: Some(cursor_color.into()),
                    ..container::Style::default()
                }),
            text(cursor_char).font(MONO),
        ]
        .into(),
    };

    let edit_bar_bg = palette.edit_bar_bg;
    let rule = palette.rule;
    let line_row = row![text(before).font(MONO), cursor, text(after).font(MONO)];
    container(line_row)
        .width(Length::Fill)
        .padding([2, 0])
        .style(move |_| container::Style {
            background: Some(edit_bar_bg.into()),
            border: iced::Border {
                color: rule,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn pretty_line<'a, Message: Clone + 'static>(
    line: &str,
    palette: &Palette,
) -> Element<'a, Message> {
    match classify_line(line) {
        Line::Blank => text(" ").into(),
        Line::Heading { level, text: t } => {
            let color = palette.heading[(level as usize).clamp(1, 6) - 1];
            let size = 24.0 - (level as f32 - 1.0) * 2.0;
            inline(
                &render_inline(&t),
                Some(color),
                Some(size),
                Weight::Bold,
                false,
                palette,
            )
        }
        Line::Task { done, text: t } => {
            let box_glyph = if done { "\u{2611}" } else { "\u{2610}" }; // ☑ / ☐
            let content = inline(
                &render_inline(&t),
                if done { Some(palette.todo_done) } else { None },
                None,
                Weight::Normal,
                done, // strikethrough when done
                palette,
            );
            row![text(box_glyph), text(" "), content].into()
        }
        Line::List {
            ordered,
            number,
            depth,
            text: t,
        } => {
            let prefix = if ordered {
                format!("{}. ", number.unwrap_or(1))
            } else {
                "\u{2022} ".to_string() // •
            };
            row![
                container(text("")).width(Length::Fixed(depth as f32 * 20.0)),
                text(prefix),
                inline(
                    &render_inline(&t),
                    None,
                    None,
                    Weight::Normal,
                    false,
                    palette
                ),
            ]
            .into()
        }
        Line::Blockquote { text: t } => {
            let border_color = palette.blockquote_border;
            container(inline(
                &render_inline(&t),
                Some(palette.muted),
                None,
                Weight::Normal,
                false,
                palette,
            ))
            .padding([0, 12])
            .style(move |_| container::Style {
                border: iced::Border {
                    color: border_color,
                    width: 3.0,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            })
            .into()
        }
        Line::Meta { key, text: t } => row![
            text(key.to_uppercase()).size(11).color(palette.muted),
            text(" "),
            inline(
                &render_inline(&t),
                Some(palette.muted),
                Some(12.0),
                Weight::Normal,
                false,
                palette,
            ),
        ]
        .into(),
        Line::Paragraph { text: t } => inline(
            &render_inline(&t),
            None,
            None,
            Weight::Normal,
            false,
            palette,
        ),
    }
}

/// Build a `rich_text` from spans. `base_*` apply to every span; per-span flags layer on top.
fn inline<'a, Message: Clone + 'static>(
    spans: &[Span],
    base_color: Option<iced::Color>,
    base_size: Option<f32>,
    base_weight: Weight,
    base_strike: bool,
    palette: &Palette,
) -> Element<'a, Message> {
    let built: Vec<_> = spans
        .iter()
        .map(|s| {
            let mut sp = span(s.text.clone());
            // Font: bold/italic/code.
            let mut font = Font {
                weight: base_weight,
                ..Font::DEFAULT
            };
            if s.bold {
                font.weight = Weight::Bold;
            }
            if s.italic {
                font.style = FontStyle::Italic;
            }
            if s.code {
                font = MONO;
            }
            sp = sp.font(font);
            if let Some(c) = base_color {
                sp = sp.color(c);
            }
            if s.code {
                sp = sp.color(palette.muted);
            }
            if s.link.is_some() {
                // No dedicated `--link` token exists in `web/`'s theme (links had no
                // custom CSS color there); reuse `--accent` rather than add a token
                // with no built-in-palette counterpart to diverge on.
                sp = sp.color(palette.accent).underline(true);
            }
            if let Some(sz) = base_size {
                sp = sp.size(sz);
            }
            if base_strike || s.strike {
                sp = sp.strikethrough(true);
            }
            // Highlight (==text==): span background if supported by the pinned version;
            // otherwise fall back to the highlight color as foreground.
            if s.highlight {
                sp = sp.color(palette.highlight_bg);
            }
            sp
        })
        .collect();
    rich_text(built).into()
}
```

- [ ] **Step 4: Rewrite `ui/tab_strip.rs`** — replace the whole file:

```rust
use iced::widget::{button, container, row, text};
use iced::{Element, Length};

use slugline_core::tabs::TabsState;

use crate::app::Message;
use crate::theme_iced::Palette;

/// A simple horizontal strip of tab buttons reflecting the open dates and the active
/// one. Port of `web/src/lib/components/Tabs.svelte`'s styling: the active tab gets an
/// `--edit-bar-bg` background + `--fg` text; inactive tabs are transparent + `--muted`.
pub fn view<'a>(tabs: &TabsState, palette: &Palette) -> Element<'a, Message> {
    let mut strip = row![].spacing(6).padding([6, 8]).width(Length::Fill);
    for (i, date) in tabs.tabs.iter().enumerate() {
        let active = i == tabs.active_index;
        let marker = if active { "\u{25b8} " } else { "" };
        let fg = palette.fg;
        let muted = palette.muted;
        let edit_bar_bg = palette.edit_bar_bg;
        let label = button(text(format!("{marker}{date}")).size(13))
            .on_press(Message::SwitchTab(i))
            .padding([4, 10])
            .style(move |_theme, _status| button::Style {
                background: if active {
                    Some(edit_bar_bg.into())
                } else {
                    None
                },
                text_color: if active { fg } else { muted },
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            });
        let close = button(text("\u{00d7}").size(13))
            .on_press(Message::CloseTab(i))
            .padding([4, 8])
            .style(move |_theme, _status| button::Style {
                background: None,
                text_color: muted,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            });
        strip = strip.push(label).push(close);
    }
    container(strip).width(Length::Fill).into()
}
```

- [ ] **Step 5: Rewrite `ui/calendar.rs`** — replace the whole file:

```rust
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use slugline_core::dates::{MonthCell, YearMonth, month_grid};

use crate::app::Message;
use crate::theme_iced::Palette;

const DOW: [&str; 7] = ["S", "M", "T", "W", "T", "F", "S"];
const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const CELL: f32 = 28.0;

/// The calendar section of the sidebar: month header with prev/next, a
/// day-of-week row, and a 6x7 grid of day cells. Days with a note file get a
/// dot; today gets an outline; the active date is filled.
/// Port of `web/src/lib/components/Calendar.svelte`.
pub fn view<'a>(
    calendar: YearMonth,
    today: &str,
    active: &str,
    notes_with_files: &[String],
    palette: &Palette,
) -> Element<'a, Message> {
    let header = row![
        button(text("\u{2039}").size(14))
            .on_press(Message::PrevMonth)
            .padding([2, 8]),
        container(text(month_label(calendar)).size(13)).center_x(Length::Fill),
        button(text("\u{203a}").size(14))
            .on_press(Message::NextMonth)
            .padding([2, 8]),
    ]
    .align_y(Alignment::Center);

    let mut dow_row = row![].spacing(2);
    for d in DOW {
        dow_row = dow_row
            .push(container(text(d).size(11).color(palette.muted)).center_x(Length::Fixed(CELL)));
    }

    let mut grid = column![dow_row].spacing(2);
    for week in month_grid(calendar.year, calendar.month) {
        let mut wk = row![].spacing(2);
        for cell in &week {
            let has_note = notes_with_files.iter().any(|d| d == &cell.date);
            wk = wk.push(day_cell(cell, today, active, has_note, palette));
        }
        grid = grid.push(wk);
    }

    column![header, grid].spacing(8).padding(12).into()
}

fn day_cell<'a>(
    cell: &MonthCell,
    today: &str,
    active: &str,
    has_note: bool,
    palette: &Palette,
) -> Element<'a, Message> {
    let day_num = cell.date[8..10].trim_start_matches('0').to_string();
    let day_num = if day_num.is_empty() {
        "0".to_string()
    } else {
        day_num
    };
    let is_today = cell.date == today;
    let is_selected = cell.date == active;
    let in_month = cell.in_month;
    let palette = *palette;

    let dot = text(if has_note { "\u{2022}" } else { " " }).size(9);
    let label = column![text(day_num).size(12), dot].align_x(Alignment::Center);

    button(label)
        .width(Length::Fixed(CELL))
        .height(Length::Fixed(CELL))
        .padding(0.0)
        .on_press(Message::OpenDate(cell.date.clone()))
        .style(move |_theme, status| {
            let background = if is_selected {
                Some(palette.accent.into())
            } else if status == button::Status::Hovered {
                Some(palette.edit_bar_bg.into())
            } else {
                None
            };
            let text_color = if is_selected {
                palette.bg
            } else if !in_month {
                palette.muted
            } else {
                palette.fg
            };
            button::Style {
                background,
                text_color,
                border: iced::Border {
                    color: if is_today {
                        palette.accent
                    } else {
                        iced::Color::TRANSPARENT
                    },
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: iced::Shadow::default(),
            }
        })
        .into()
}

fn month_label(ym: YearMonth) -> String {
    let name = MONTH_NAMES[(ym.month as usize - 1).min(11)];
    format!("{name} {}", ym.year)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_month_and_year() {
        assert_eq!(
            month_label(YearMonth {
                year: 2026,
                month: 6
            }),
            "June 2026"
        );
        assert_eq!(
            month_label(YearMonth {
                year: 2027,
                month: 1
            }),
            "January 2027"
        );
    }
}
```

- [ ] **Step 6: Rewrite `ui/agenda.rs`** — replace the whole file:

```rust
use iced::widget::{button, column, container, row, span, text};
use iced::{Element, Length};

use slugline_core::agenda::{AgendaItem, derive_agenda};

use crate::app::Message;
use crate::theme_iced::Palette;

/// The sidebar's Agenda section: scheduled meetings for the currently open note,
/// derived fresh from its lines on every render (no stored state, mirroring the
/// web's `$derived(deriveAgenda(app.editor.lines))`). Port of
/// `web/src/lib/components/Agenda.svelte`.
pub fn view<'a>(lines: &[String], active: &str, palette: &Palette) -> Element<'a, Message> {
    let items = derive_agenda(lines);
    let status_bar = palette.status_bar;

    let header = container(text("Agenda").size(13).color(palette.heading[1]));
    let body: Element<'a, Message> = if items.is_empty() {
        text("No scheduled meetings")
            .size(12)
            .color(palette.muted)
            .into()
    } else {
        let mut list = column![].spacing(2);
        for item in items {
            list = list.push(agenda_row(item, active, palette));
        }
        list.into()
    };

    container(column![header, body].spacing(6).width(Length::Fill))
        .padding([10, 12])
        .style(move |_theme| container::Style {
            border: iced::Border {
                color: status_bar,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn agenda_row<'a>(item: AgendaItem, active: &str, palette: &Palette) -> Element<'a, Message> {
    let done = item.ended.is_some();
    let name_color = if done { palette.todo_done } else { palette.fg };
    let fg = palette.fg;
    let todo_done = palette.todo_done;
    let edit_bar_bg = palette.edit_bar_bg;

    let mut label = row![
        text(item.time).size(12).color(palette.accent),
        iced::widget::rich_text([span(item.name).color(name_color).strikethrough(done)]).size(12),
    ]
    .spacing(6);
    if done {
        label = label.push(text("\u{2713}").size(11).color(todo_done));
    }

    button(label)
        .padding([2, 4])
        .width(Length::Fill)
        .on_press(Message::OpenDateAndLine(
            active.to_string(),
            item.heading_line_index,
        ))
        .style(move |_theme, status| {
            let background = if status == button::Status::Hovered {
                Some(edit_bar_bg.into())
            } else {
                None
            };
            button::Style {
                background,
                text_color: fg,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            }
        })
        .into()
}
```

- [ ] **Step 7: Rewrite `ui/todo_list.rs`** — replace the whole file:

```rust
use iced::widget::{button, column, container, row, span, text};
use iced::{Element, Length};

use slugline_core::todos::{TodoGroup, TodoItem};

use crate::app::Message;
use crate::theme_iced::Palette;

/// The sidebar's To Do section: the 7-day aggregation kept fresh in `App::todo_groups`
/// (a `Task`-driven disk read, unlike Agenda's per-render derivation — see design
/// Section 5). Port of `web/src/lib/components/TodoList.svelte`.
pub fn view<'a>(groups: &[TodoGroup], palette: &Palette) -> Element<'a, Message> {
    let status_bar = palette.status_bar;
    let header = container(text("To Do").size(13).color(palette.heading[1]));

    let body: Element<'a, Message> = if groups.is_empty() {
        text("No to dos in the last 7 days")
            .size(12)
            .color(palette.muted)
            .into()
    } else {
        let mut list = column![].spacing(8);
        for group in groups {
            list = list.push(group_view(group, palette));
        }
        list.into()
    };

    container(column![header, body].spacing(6).width(Length::Fill))
        .padding([10, 12])
        .style(move |_theme| container::Style {
            border: iced::Border {
                color: status_bar,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn group_view<'a>(group: &TodoGroup, palette: &Palette) -> Element<'a, Message> {
    let mut list = column![text(group.date.clone()).size(11).color(palette.muted)].spacing(2);
    for todo in &group.todos {
        list = list.push(todo_row(group.date.clone(), todo, palette));
    }
    list.into()
}

fn todo_row<'a>(date: String, todo: &TodoItem, palette: &Palette) -> Element<'a, Message> {
    let box_glyph = if todo.done { "\u{2611}" } else { "\u{2610}" }; // ☑ / ☐
    let text_color = if todo.done {
        palette.todo_done
    } else {
        palette.fg
    };
    let fg = palette.fg;
    let edit_bar_bg = palette.edit_bar_bg;

    let label = row![
        text(box_glyph).size(12),
        iced::widget::rich_text([span(todo.text.clone())
            .color(text_color)
            .strikethrough(todo.done)])
        .size(12),
    ]
    .spacing(6);

    button(label)
        .padding([2, 4])
        .width(Length::Fill)
        .on_press(Message::OpenDateAndLine(date, todo.line_index))
        .style(move |_theme, status| {
            let background = if status == button::Status::Hovered {
                Some(edit_bar_bg.into())
            } else {
                None
            };
            button::Style {
                background,
                text_color: fg,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            }
        })
        .into()
}
```

- [ ] **Step 8: Rewrite `ui/sidebar.rs`** — replace the whole file:

```rust
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use slugline_core::dates::YearMonth;
use slugline_core::todos::TodoGroup;

use crate::app::Message;
use crate::theme_iced::Palette;
use crate::ui::{agenda, calendar, todo_list};

/// The sidebar pane: a collapse header followed by the calendar, agenda, and to-do
/// sections, stacked and independently scrollable. Port of
/// `web/src/lib/components/Sidebar.svelte`.
pub fn view<'a>(
    calendar_month: YearMonth,
    today: &str,
    active: &str,
    notes_with_files: &[String],
    lines: &[String],
    todo_groups: &[TodoGroup],
    palette: &Palette,
) -> Element<'a, Message> {
    let header = row![
        container(text("Slugline").size(13)).width(Length::Fill),
        button(text("\u{ab}").size(13)) // «
            .on_press(Message::ToggleSidebar)
            .padding([2, 8]),
    ]
    .align_y(Alignment::Center)
    .padding([8, 10]);

    let body = column![
        calendar::view(calendar_month, today, active, notes_with_files, palette),
        agenda::view(lines, active, palette),
        todo_list::view(todo_groups, palette),
    ]
    .width(Length::Fill);

    column![header, scrollable(body).height(Length::Fill)]
        .width(Length::Fill)
        .into()
}

/// The slim rail shown instead of the sidebar when it is collapsed.
pub fn collapsed_rail<'a>() -> Element<'a, Message> {
    container(
        button(text("\u{bb}").size(13)) // »
            .on_press(Message::ToggleSidebar)
            .padding([6, 8]),
    )
    .padding([8, 4])
    .into()
}
```

- [ ] **Step 9: Rewrite `ui/command_palette.rs`** — replace the whole file:

```rust
use iced::alignment::{Horizontal, Vertical};
use iced::font::Weight;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Font, Length};

use slugline_core::doc::{ArgKind, COMMANDS, CommandSpec};

use crate::app::Message;
use crate::theme_iced::Palette;

const MONO: Font = Font::MONOSPACE;
const MAX_SUGGESTIONS: usize = 8;

/// A short usage hint shown after each command's name, derived from its `ArgKind`.
fn usage_hint(spec: &CommandSpec) -> &'static str {
    match spec.arg_kind {
        ArgKind::None => "",
        ArgKind::Text => " <text>",
        ArgKind::Time => " <HH:MM>",
        ArgKind::Date => " <YYYY-MM-DD>",
        ArgKind::Theme => " [light|dark]",
    }
}

/// A one-line description shown next to each command in the palette list.
fn description(spec: &CommandSpec) -> &'static str {
    match spec.name.canonical() {
        "meeting" => "Start a new meeting",
        "note" => "Start a new note",
        "section" => "Add a subsection here",
        "todo" => "Add a to-do item",
        "start" => "Record the meeting's start time",
        "end" => "Record the meeting's end time",
        "scheduled" => "Set the meeting's scheduled time",
        "purpose" => "Set the meeting's purpose",
        "topic" => "Set the note's topic",
        "people" => "Add people (or :p)",
        "goto" => "Jump to a date",
        "today" => "Jump to today",
        "tab" => "Open a date in a new tab",
        "close" => "Close the active tab",
        "w" => "Save now",
        "theme" => "Switch theme",
        _ => "",
    }
}

/// A subsequence fuzzy score between a typed `query` and a candidate `target`, both
/// compared case-insensitively. Every character of `query` must appear in `target`, in
/// order (not necessarily contiguous); returns `None` if it doesn't. Higher scores are
/// better: matches earlier in `target` and consecutive runs of matched characters score
/// higher. An empty `query` matches everything with a score of `0` (used to show every
/// command when nothing has been typed yet).
pub fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let q: Vec<char> = query.to_lowercase().chars().collect();
    let t: Vec<char> = target.to_lowercase().chars().collect();

    let mut score = 0i32;
    let mut ti = 0usize;
    let mut run = 0i32;
    for &qc in &q {
        let mut matched = false;
        while ti < t.len() {
            if t[ti] == qc {
                run += 1;
                score += if ti == 0 { 10 } else { 1 } + run;
                ti += 1;
                matched = true;
                break;
            }
            run = 0;
            ti += 1;
        }
        if !matched {
            return None;
        }
    }
    Some(score)
}

/// Every command whose name fuzzy-matches `query`, in the canonical `COMMANDS` order when
/// `query` is empty, or best-match-first otherwise. Capped at `MAX_SUGGESTIONS`.
pub fn filter_commands(query: &str) -> Vec<&'static CommandSpec> {
    let mut scored: Vec<(&'static CommandSpec, i32)> = COMMANDS
        .iter()
        .filter_map(|spec| fuzzy_score(query, spec.name.canonical()).map(|score| (spec, score)))
        .collect();
    if !query.is_empty() {
        scored.sort_by(|a, b| b.1.cmp(&a.1));
    }
    scored
        .into_iter()
        .take(MAX_SUGGESTIONS)
        .map(|(spec, _)| spec)
        .collect()
}

/// The command palette overlay: a top-centered box with the typed `:command` text and a
/// fuzzy-filtered list of known commands below it. Rendered in a `stack!` on top of the
/// base view whenever `editor.command.is_some()` (design Section 4).
pub fn view<'a>(typed: &str, palette: &Palette) -> Element<'a, Message> {
    let suggestions = filter_commands(typed);
    let edit_bar_bg = palette.edit_bar_bg;
    let rule = palette.rule;
    let bg = palette.bg;
    let status_bar = palette.status_bar;

    let input = container(
        text(format!(":{typed}"))
            .font(MONO)
            .size(15)
            .color(palette.fg),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(edit_bar_bg.into()),
        border: iced::Border {
            color: rule,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..container::Style::default()
    });

    let mut list = column![].spacing(1);
    if suggestions.is_empty() {
        list = list.push(
            container(text("No matching commands").size(12).color(palette.muted)).padding([4, 8]),
        );
    } else {
        for spec in suggestions {
            list = list.push(suggestion_row(spec, palette));
        }
    }

    let box_ = container(column![input, list].spacing(6).width(Length::Fixed(440.0)))
        .padding(12)
        .style(move |_theme| container::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: status_bar,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::BLACK,
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..container::Style::default()
        });

    container(box_)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Top)
        .padding(iced::Padding {
            top: 48.0,
            ..iced::Padding::ZERO
        })
        .into()
}

fn suggestion_row<'a>(spec: &'static CommandSpec, palette: &Palette) -> Element<'a, Message> {
    let name = spec.name.canonical();
    let accent = palette.accent;
    let muted = palette.muted;
    let fg = palette.fg;
    let edit_bar_bg = palette.edit_bar_bg;
    let label = row![
        text(format!(":{name}{}", usage_hint(spec)))
            .font(Font {
                weight: Weight::Bold,
                ..MONO
            })
            .size(12)
            .color(accent),
        text(description(spec)).size(12).color(muted),
    ]
    .spacing(10);

    button(label)
        .padding([3, 8])
        .width(Length::Fill)
        .on_press(Message::PaletteSuggestionClicked(name.to_string()))
        .style(move |_theme, status| {
            let background = if status == button::Status::Hovered {
                Some(edit_bar_bg.into())
            } else {
                None
            };
            button::Style {
                background,
                text_color: fg,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            }
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_everything_with_score_zero() {
        assert_eq!(fuzzy_score("", "todo"), Some(0));
    }

    #[test]
    fn requires_every_query_char_to_appear_in_order() {
        assert!(fuzzy_score("td", "todo").is_some()); // t...d
        assert!(fuzzy_score("dt", "todo").is_none()); // wrong order
        assert!(fuzzy_score("xyz", "todo").is_none()); // not present at all
    }

    #[test]
    fn an_exact_prefix_scores_higher_than_a_scattered_match() {
        let exact = fuzzy_score("to", "today").unwrap();
        let scattered = fuzzy_score("ty", "today").unwrap();
        assert!(exact > scattered);
    }

    #[test]
    fn filter_commands_with_empty_query_returns_the_first_page_in_canonical_order() {
        let names: Vec<&str> = filter_commands("")
            .into_iter()
            .map(|s| s.name.canonical())
            .collect();
        assert_eq!(
            names,
            vec![
                "meeting",
                "note",
                "section",
                "todo",
                "start",
                "end",
                "scheduled",
                "purpose"
            ]
        );
        assert_eq!(names.len(), MAX_SUGGESTIONS);
    }

    #[test]
    fn filter_commands_narrows_to_fuzzy_matches() {
        let names: Vec<&str> = filter_commands("mee")
            .into_iter()
            .map(|s| s.name.canonical())
            .collect();
        assert!(names.contains(&"meeting"));
        assert!(!names.contains(&"close"));
    }

    #[test]
    fn filter_commands_with_no_matches_is_empty() {
        assert!(filter_commands("zzzzz").is_empty());
    }
}
```

(Only `use crate::theme_iced::Palette;` and the two `view` signatures actually changed from the
Phase 5 version of this file — `fuzzy_score`/`filter_commands`/their tests are untouched, shown in
full here only because this step replaces the whole file.)

- [ ] **Step 10: Attempt a build** — `cargo build -p slugline`
Expected: fails — `app.rs`'s call sites (`sidebar::view(...)`, `editor_pane::view(...)`,
`tab_strip::view(...)`, `command_palette::view(...)`) don't pass `&Palette` yet, and `App` has no
`palette` field yet. Task 4 fixes both. This mirrors Phase 3's Task 2/4 "build fails until the next
task" checkpoint.

(Do not commit yet — Task 4 is required for this crate to build at all.)

---

### Task 4: Wire config into `App` — `theme`/`color_overrides`/`palette`/`config_path` fields

**Files:**
- Modify: `crates/slugline/src/app.rs`, `crates/slugline/src/main.rs`

- [ ] **Step 1: Update imports and add the `ERROR_TIMEOUT` constant** — in
`crates/slugline/src/app.rs`, replace the top of the file:

```rust
use std::time::{Duration, Instant};

use iced::widget::{column, pane_grid, row, stack};
use iced::{Element, Length, Subscription, Task, keyboard, time, window};

use slugline_core::dates::{YearMonth, add_days, now_hhmm, today_iso, year_month};
use slugline_core::editor::{
    AppEffect, CommandCtx, Cursor, EditorState, KeyInput, clamp_cursor, create_editor_state,
    handle_key,
};
use slugline_core::store::NotesStore;
use slugline_core::tabs::{
    TabsState, active_date, close_tab, init_tabs, next_tab, open_new_tab, prev_tab, retarget,
};
use slugline_core::todos::{TodoGroup, extract_todos, window_dates};

use crate::keys::key_string;
use crate::ui::{command_palette, editor_pane, sidebar, tab_strip};

const SAVE_DEBOUNCE: Duration = Duration::from_millis(750);
/// The sidebar's share of the window width when the app starts.
const INITIAL_SIDEBAR_RATIO: f32 = 0.22;
```

with:

```rust
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::widget::{column, pane_grid, row, stack};
use iced::{Element, Length, Subscription, Task, keyboard, time, window};

use slugline_core::config::{UiConfig, update_theme};
use slugline_core::dates::{YearMonth, add_days, now_hhmm, today_iso, year_month};
use slugline_core::editor::{
    AppEffect, CommandCtx, Cursor, EditorState, KeyInput, clamp_cursor, create_editor_state,
    handle_key,
};
use slugline_core::store::NotesStore;
use slugline_core::tabs::{
    TabsState, active_date, close_tab, init_tabs, next_tab, open_new_tab, prev_tab, retarget,
};
use slugline_core::theme::Tokens;
use slugline_core::todos::{TodoGroup, extract_todos, window_dates};

use crate::keys::key_string;
use crate::theme_iced::Palette;
use crate::ui::{command_palette, editor_pane, sidebar, status_line, tab_strip, toast};

const SAVE_DEBOUNCE: Duration = Duration::from_millis(750);
/// The sidebar's share of the window width when the app starts.
const INITIAL_SIDEBAR_RATIO: f32 = 0.22;
/// How long an error toast stays visible before auto-dismissing. Matches the web's
/// `Toast`/`setError` auto-dismiss window (design Section 6).
const ERROR_TIMEOUT: Duration = Duration::from_secs(5);
```

(`status_line`/`toast` are unused by `App` until Tasks 6/7 add their call sites in `view()`/
`main_pane()` — importing them now avoids touching this `use` line three times.)

- [ ] **Step 2: Add the new `App` fields** — replace:

```rust
    /// The sidebar | main split.
    panes: pane_grid::State<PaneKind>,
    /// True when the whole sidebar is collapsed to a slim rail.
    sidebar_collapsed: bool,
    #[allow(dead_code)]
    error: Option<String>,
}
```

with:

```rust
    /// The sidebar | main split.
    panes: pane_grid::State<PaneKind>,
    /// True when the whole sidebar is collapsed to a slim rail.
    sidebar_collapsed: bool,
    /// Where `:theme` persists (`update_theme`). Set once at startup from the CLI/config
    /// resolution in `main.rs`; never changes for the process's lifetime.
    config_path: PathBuf,
    /// The active theme name (`"light"` or `"dark"`), applied optimistically on `:theme`
    /// before the persistence `Task` resolves.
    theme: String,
    /// Per-theme color overrides from config (`ui.colors`), merged over the built-ins by
    /// `Palette::for_theme`.
    color_overrides: std::collections::BTreeMap<String, Tokens>,
    /// The current theme's tokens, resolved to Iced colors. Recomputed only when `theme`
    /// changes (boot + every `:theme`), not on every `view()` call.
    palette: Palette,
    error: Option<String>,
    /// When the current `error` should auto-dismiss (design Section 6: 5s, mirrors the
    /// web's `Toast`/`setError`). `None` whenever `error` is `None`.
    error_expires_at: Option<Instant>,
}
```

(`error` loses its `#[allow(dead_code)]` — Task 7 makes it live by rendering the toast.)

- [ ] **Step 3: Update `App::new`'s signature and body** — replace:

```rust
impl App {
    pub fn new(store: NotesStore, date: String) -> Self {
        let (content, error) = match store.read_or_create(&date) {
            Ok(c) => (c, None),
            Err(e) => (String::new(), Some(format!("Failed to load note: {e}"))),
        };
        let editor = create_editor_state(content.lines().map(str::to_string).collect(), Vec::new());
        let panes = pane_grid::State::with_configuration(pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: INITIAL_SIDEBAR_RATIO,
            a: Box::new(pane_grid::Configuration::Pane(PaneKind::Sidebar)),
            b: Box::new(pane_grid::Configuration::Pane(PaneKind::Main)),
        });
        Self {
            store,
            tabs: init_tabs(&date),
            editor,
            shared_register: Vec::new(),
            last_saved: content,
            dirty_since: None,
            saving: false,
            loading: false,
            calendar: year_month(&date),
            notes_with_files: Vec::new(),
            todo_groups: Vec::new(),
            pending_jump_line: None,
            panes,
            sidebar_collapsed: false,
            error,
        }
    }
```

with:

```rust
impl App {
    pub fn new(store: NotesStore, date: String, ui_config: UiConfig, config_path: PathBuf) -> Self {
        let (content, error) = match store.read_or_create(&date) {
            Ok(c) => (c, None),
            Err(e) => (String::new(), Some(format!("Failed to load note: {e}"))),
        };
        let editor = create_editor_state(content.lines().map(str::to_string).collect(), Vec::new());
        let panes = pane_grid::State::with_configuration(pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: INITIAL_SIDEBAR_RATIO,
            a: Box::new(pane_grid::Configuration::Pane(PaneKind::Sidebar)),
            b: Box::new(pane_grid::Configuration::Pane(PaneKind::Main)),
        });
        let theme = ui_config.theme;
        let color_overrides = ui_config.colors;
        let palette = Palette::for_theme(&theme, &color_overrides);
        let error_expires_at = error.as_ref().map(|_| Instant::now() + ERROR_TIMEOUT);
        Self {
            store,
            tabs: init_tabs(&date),
            editor,
            shared_register: Vec::new(),
            last_saved: content,
            dirty_since: None,
            saving: false,
            loading: false,
            calendar: year_month(&date),
            notes_with_files: Vec::new(),
            todo_groups: Vec::new(),
            pending_jump_line: None,
            panes,
            sidebar_collapsed: false,
            config_path,
            theme,
            color_overrides,
            palette,
            error,
            error_expires_at,
        }
    }
```

- [ ] **Step 4: Pass `&self.palette` at every existing `ui::*::view` call site** — in `view()`,
replace:

```rust
                    PaneKind::Sidebar => sidebar::view(
                        self.calendar,
                        &today_iso(),
                        &self.active_date(),
                        &self.notes_with_files,
                        &self.editor.lines,
                        &self.todo_groups,
                    ),
```

with:

```rust
                    PaneKind::Sidebar => sidebar::view(
                        self.calendar,
                        &today_iso(),
                        &self.active_date(),
                        &self.notes_with_files,
                        &self.editor.lines,
                        &self.todo_groups,
                        &self.palette,
                    ),
```

then a few lines below, replace:

```rust
        match &self.editor.command {
            Some(typed) => stack![base, command_palette::view(typed)].into(),
            None => base,
        }
    }

    fn main_pane(&self) -> Element<'_, Message> {
        column![tab_strip::view(&self.tabs), editor_pane::view(&self.editor)].into()
    }
```

with:

```rust
        let with_palette: Element<'_, Message> = match &self.editor.command {
            Some(typed) => stack![base, command_palette::view(typed, &self.palette)].into(),
            None => base,
        };
        with_palette
    }

    fn main_pane(&self) -> Element<'_, Message> {
        column![
            tab_strip::view(&self.tabs, &self.palette),
            editor_pane::view(&self.editor, &self.palette),
        ]
        .into()
    }
```

(This intermediate shape — a `with_palette` binding immediately returned — looks redundant on its
own; it exists so Task 7 can insert the toast-overlay `match` between it and the function's real
return without another diff to this exact spot. `main_pane` similarly gets its third row,
`status_line::view(...)`, in Task 6.)

- [ ] **Step 5: Update the test helper** — in `#[cfg(test)] mod tests`, replace:

```rust
    fn temp_app(date: &str) -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        let store = NotesStore::new(dir.path().to_path_buf());
        let app = App::new(store, date.to_string());
        (dir, app)
    }
```

with:

```rust
    fn temp_app(date: &str) -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        let store = NotesStore::new(dir.path().to_path_buf());
        let config_path = dir.path().join("config.toml");
        let app = App::new(store, date.to_string(), UiConfig::default(), config_path);
        (dir, app)
    }
```

(`UiConfig::default().theme == "light"` — every existing test's assertions about `app.editor`/
`app.tabs`/etc. are theme-independent, so this is the only test-helper change this task needs.)

- [ ] **Step 6: Wire `main.rs`** — replace:

```rust
    let store = NotesStore::new(resolved.notes_dir);
    let date = today_iso();

    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run_with(move || {
            let app = App::new(store.clone(), date.clone());
            let boot = app.boot();
            (app, boot)
        })
}
```

with:

```rust
    let store = NotesStore::new(resolved.notes_dir);
    let date = today_iso();
    let ui_config = config.ui;

    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run_with(move || {
            let app = App::new(
                store.clone(),
                date.clone(),
                ui_config.clone(),
                config_path.clone(),
            );
            let boot = app.boot();
            (app, boot)
        })
}
```

- [ ] **Step 7: Build and test** — `cargo build -p slugline && cargo test -p slugline`
Expected: builds clean; all pre-existing `slugline` tests still pass (41: the 37 from before this
phase plus Task 2's 4 `theme_iced::` tests — this task only added fields/plumbing, no new test
cases of its own).

- [ ] **Step 8: Commit Tasks 3-4 together** (Task 3's seven files only build with Task 4's `app.rs`/
`main.rs` changes):

```bash
git add crates/slugline/src/ui/ crates/slugline/src/app.rs crates/slugline/src/main.rs
git commit -m "feat(app): thread a runtime-switchable Palette through every ui view"
```

---

### Task 5: Implement `:theme` switching + persistence

**Files:**
- Modify: `crates/slugline/src/app.rs`

- [ ] **Step 1: Add the `ThemePersisted` message variant** — replace:

```rust
    /// A command palette suggestion was clicked: seed the command buffer with that
    /// command's name (plus a trailing space, ready for its argument) rather than
    /// running it — matches typing the name and leaves `Enter` as the one path that
    /// invokes `run_command`.
    PaletteSuggestionClicked(String),
}
```

with:

```rust
    /// A command palette suggestion was clicked: seed the command buffer with that
    /// command's name (plus a trailing space, ready for its argument) rather than
    /// running it — matches typing the name and leaves `Enter` as the one path that
    /// invokes `run_command`.
    PaletteSuggestionClicked(String),
    /// The `update_theme` persistence write for a `:theme` switch finished. `target` is
    /// the theme that was applied optimistically; `prev` is what to roll back to on
    /// failure.
    ThemePersisted {
        target: String,
        prev: String,
        res: Result<(), String>,
    },
}
```

(`Message::DismissError` is added in Task 7, once the toast that uses it exists — this task only
needs `ThemePersisted`.)

- [ ] **Step 2: Replace the `AppEffect::Theme` stub with `switch_theme`, and add `set_error`** —
in `run_effect`, replace:

```rust
    /// Translate an editor `AppEffect` into a follow-up `Task` (the web `runEffect`).
    fn run_effect(&mut self, effect: AppEffect) -> Task<Message> {
        match effect {
            AppEffect::Save => self.spawn_save(),
            AppEffect::Theme(_) => Task::none(), // wired in Phase 6
            nav => {
                let today = today_iso();
                let active = self.active_date();
                match plan_tabs(&self.tabs, &active, &today, &nav) {
                    Some(new_tabs) => self.navigate(new_tabs),
                    None => Task::none(),
                }
            }
        }
    }
```

with:

```rust
    /// Translate an editor `AppEffect` into a follow-up `Task` (the web `runEffect`).
    fn run_effect(&mut self, effect: AppEffect) -> Task<Message> {
        match effect {
            AppEffect::Save => self.spawn_save(),
            AppEffect::Theme(arg) => self.switch_theme(arg),
            nav => {
                let today = today_iso();
                let active = self.active_date();
                match plan_tabs(&self.tabs, &active, &today, &nav) {
                    Some(new_tabs) => self.navigate(new_tabs),
                    None => Task::none(),
                }
            }
        }
    }

    /// Apply a `:theme`/`:theme dark` command optimistically, then persist it. `arg` is
    /// `""` (toggle via `next_theme`), `"light"`, or `"dark"` — `validate_command` already
    /// rejected anything else before `run_command` ever produced this effect. Port of
    /// design Section 5's "`:theme` flows effect -> `update` swaps `config.theme` ->
    /// next `view` uses the new theme, and persists via ... `toml_edit` writer".
    fn switch_theme(&mut self, arg: String) -> Task<Message> {
        let prev = self.theme.clone();
        let target = if arg.is_empty() {
            slugline_core::theme::next_theme(&prev)
        } else {
            arg
        };
        if target == prev {
            return Task::none();
        }
        self.theme = target.clone();
        self.palette = Palette::for_theme(&self.theme, &self.color_overrides);

        let path = self.config_path.clone();
        let to_persist = target.clone();
        Task::perform(
            async move { update_theme(&path, &to_persist).map_err(|e| e.to_string()) },
            move |res| Message::ThemePersisted {
                target: target.clone(),
                prev: prev.clone(),
                res,
            },
        )
    }

    /// Set `error` (+ its 5s auto-dismiss expiry), matching the web's `setError`.
    fn set_error(&mut self, message: String) {
        self.error = Some(message);
        self.error_expires_at = Some(Instant::now() + ERROR_TIMEOUT);
    }
```

- [ ] **Step 3: Route the two existing direct `self.error = Some(...)` assignments through
`set_error`** — replace:

```rust
                    Err(e) => {
                        self.error =
                            Some(format!("Save failed \u{2014} edits kept, will retry: {e}"));
                        // dirty_since stays set, so the next Tick retries.
                    }
```

with:

```rust
                    Err(e) => {
                        self.set_error(format!("Save failed \u{2014} edits kept, will retry: {e}"));
                        // dirty_since stays set, so the next Tick retries.
                    }
```

and replace:

```rust
                    Err(e) => {
                        // Don't apply a queued jump against a buffer that never arrived.
                        self.pending_jump_line = None;
                        let date = self.active_date();
                        self.error = Some(format!("Failed to load note {date}: {e}"));
                        Task::none()
                    }
```

with:

```rust
                    Err(e) => {
                        // Don't apply a queued jump against a buffer that never arrived.
                        self.pending_jump_line = None;
                        let date = self.active_date();
                        self.set_error(format!("Failed to load note {date}: {e}"));
                        Task::none()
                    }
```

(These two spots previously set `error` without an expiry at all, since nothing rendered or
auto-dismissed it yet. From this task on, every error goes through `set_error` so the 5s dismiss —
wired in Task 7 — applies uniformly.)

- [ ] **Step 4: Handle `ThemePersisted` in `update()`** — replace:

```rust
            Message::PaletteSuggestionClicked(name) => {
                self.editor.command = Some(format!("{name} "));
                Task::none()
            }
        }
    }
```

with:

```rust
            Message::PaletteSuggestionClicked(name) => {
                self.editor.command = Some(format!("{name} "));
                Task::none()
            }
            Message::ThemePersisted { target, prev, res } => {
                if let Err(e) = res {
                    // Only roll back if we're still on the theme that failed to persist —
                    // if the user has since switched again, rolling back would clobber
                    // their newer choice.
                    if self.theme == target {
                        self.theme = prev;
                        self.palette = Palette::for_theme(&self.theme, &self.color_overrides);
                    }
                    self.set_error(format!("Failed to save theme: {e}"));
                }
                Task::none()
            }
        }
    }
```

- [ ] **Step 5: Add the tests** — in `#[cfg(test)] mod tests`, insert after
`palette_suggestion_clicked_seeds_the_command_with_a_trailing_space`:

```rust
    #[test]
    fn theme_effect_with_empty_arg_toggles_to_the_opposite_theme() {
        let (_dir, mut app) = temp_app("2026-06-23");
        assert_eq!(app.theme, "light");
        let _ = app.run_effect(AppEffect::Theme(String::new()));
        assert_eq!(app.theme, "dark");
        assert_eq!(
            app.palette.bg,
            Palette::for_theme("dark", &app.color_overrides).bg
        );
    }

    #[test]
    fn theme_effect_with_an_explicit_arg_sets_that_theme() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let _ = app.run_effect(AppEffect::Theme("dark".to_string()));
        assert_eq!(app.theme, "dark");
    }

    #[test]
    fn theme_effect_to_the_current_theme_is_a_noop() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let before = app.palette;
        let _ = app.run_effect(AppEffect::Theme("light".to_string()));
        assert_eq!(app.theme, "light");
        assert_eq!(app.palette, before);
    }

    #[test]
    fn theme_persisted_failure_rolls_back_to_the_previous_theme() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let _ = app.run_effect(AppEffect::Theme("dark".to_string()));
        assert_eq!(app.theme, "dark");
        let _ = app.update(Message::ThemePersisted {
            target: "dark".to_string(),
            prev: "light".to_string(),
            res: Err("disk full".to_string()),
        });
        assert_eq!(app.theme, "light");
        assert_eq!(
            app.palette.bg,
            Palette::for_theme("light", &app.color_overrides).bg
        );
        assert!(app.error.unwrap().contains("Failed to save theme"));
    }

    #[test]
    fn theme_persisted_failure_does_not_roll_back_a_since_changed_theme() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let _ = app.run_effect(AppEffect::Theme("dark".to_string()));
        let _ = app.run_effect(AppEffect::Theme("light".to_string())); // user switched again
        let _ = app.update(Message::ThemePersisted {
            target: "dark".to_string(), // the stale, now-superseded persistence result
            prev: "light".to_string(),
            res: Err("disk full".to_string()),
        });
        assert_eq!(app.theme, "light"); // untouched by the stale rollback
    }
```

- [ ] **Step 6: Run the tests** — `cargo test -p slugline app::tests::theme`
Expected: PASS (5 tests).

- [ ] **Step 7: Run the full `slugline` suite** — `cargo test -p slugline`
Expected: PASS (46 tests: 41 existing + 5 new).

- [ ] **Step 8: Commit**

```bash
git add crates/slugline/src/app.rs
git commit -m "feat(app): wire :theme to actually switch and persist"
```

---

### Task 6: Add the status line

**Files:**
- Create: `crates/slugline/src/ui/status_line.rs`
- Modify: `crates/slugline/src/app.rs`

- [ ] **Step 1: Write the failing tests** — create `crates/slugline/src/ui/status_line.rs`:

```rust
use iced::widget::{container, row, text};
use iced::{Element, Length};

use slugline_core::doc::{Context, resolve_context, scan_document};
use slugline_core::editor::{EditorState, Mode};

use crate::app::Message;
use crate::theme_iced::Palette;

/// A one-line breadcrumb for where the cursor currently is, for the status line's
/// left segment (outside command mode). Port of `web/src/lib/components/StatusLine.svelte`'s
/// inline `context` derivation.
fn context_label(editor: &EditorState) -> String {
    let model = scan_document(&editor.lines);
    match resolve_context(&model, editor.cursor.line) {
        Context::None => String::new(),
        Context::Todo { .. } => "To Do".to_string(),
        Context::Meeting { block, .. } => format!("Meetings \u{203a} {}", block.name),
        Context::Note { block, .. } => format!("Notes \u{203a} {}", block.name),
        Context::Other { section } => section.title,
    }
}

/// The footer status line: mode + cursor-context breadcrumb on the left (or the typed
/// `:command` while command mode is active, matching the web's footer even though the
/// command palette overlay is the actual input surface now), the current editor message
/// right-aligned. Port of `web/src/lib/components/StatusLine.svelte`.
pub fn view<'a>(editor: &EditorState, palette: &Palette) -> Element<'a, Message> {
    let left: Element<'a, Message> = match &editor.command {
        Some(typed) => text(format!(":{typed}")).size(12).color(palette.fg).into(),
        None => {
            let mode_label = match editor.mode {
                Mode::Insert => "-- INSERT --",
                Mode::Normal => "-- NORMAL --",
            };
            row![
                text(mode_label).size(12).color(palette.fg),
                text(context_label(editor)).size(12).color(palette.muted),
            ]
            .spacing(16)
            .into()
        }
    };

    let status_bar = palette.status_bar;
    container(
        row![
            left,
            container(text(editor.message.clone()).size(12).color(palette.accent))
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(16)
        .width(Length::Fill),
    )
    .padding([4, 16])
    .width(Length::Fill)
    .style(move |_theme| iced::widget::container::Style {
        background: Some(status_bar.into()),
        ..iced::widget::container::Style::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use slugline_core::editor::create_editor_state;

    fn lines(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn labels_the_to_do_section() {
        let raw = ["# T", "", "## To Do", "- [ ] x", "", "## Notes", ""];
        let mut editor = create_editor_state(lines(&raw), Vec::new());
        editor.cursor.line = 3;
        assert_eq!(context_label(&editor), "To Do");
    }

    #[test]
    fn labels_a_meeting_block_with_its_name() {
        let raw = [
            "# T",
            "",
            "## Meetings",
            "### Sync",
            "body",
            "",
            "## Notes",
            "",
        ];
        let mut editor = create_editor_state(lines(&raw), Vec::new());
        editor.cursor.line = 4;
        assert_eq!(context_label(&editor), "Meetings \u{203a} Sync");
    }

    #[test]
    fn is_empty_on_the_title_line() {
        let raw = ["# T", "", "## Notes", ""];
        let editor = create_editor_state(lines(&raw), Vec::new());
        assert_eq!(context_label(&editor), "");
    }
}
```

`ui/mod.rs` already declares `pub mod status_line;` (added in Task 3, Step 2) — no change needed
there.

- [ ] **Step 2: Wire it into `main_pane`** — in `crates/slugline/src/app.rs`, replace:

```rust
    fn main_pane(&self) -> Element<'_, Message> {
        column![
            tab_strip::view(&self.tabs, &self.palette),
            editor_pane::view(&self.editor, &self.palette),
        ]
        .into()
    }
```

with:

```rust
    fn main_pane(&self) -> Element<'_, Message> {
        column![
            tab_strip::view(&self.tabs, &self.palette),
            editor_pane::view(&self.editor, &self.palette),
            status_line::view(&self.editor, &self.palette),
        ]
        .into()
    }
```

- [ ] **Step 3: Run the tests** — `cargo test -p slugline status_line::`
Expected: PASS (3 tests).

- [ ] **Step 4: Build and run the full suite** — `cargo build -p slugline && cargo test -p slugline`
Expected: builds clean; PASS (49 tests: 46 from Task 5 + 3 new).

- [ ] **Step 5: Commit**

```bash
git add crates/slugline/src/ui/status_line.rs crates/slugline/src/app.rs
git commit -m "feat(app): add the status line (mode/context/message)"
```

---

### Task 7: Add the error toast + 5s auto-dismiss

**Files:**
- Create: `crates/slugline/src/ui/toast.rs`
- Modify: `crates/slugline/src/app.rs`

- [ ] **Step 1: Create the toast widget** — create `crates/slugline/src/ui/toast.rs`:

```rust
use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length};

use crate::app::Message;

/// A fixed-position, bottom-centered error toast with a dismiss button. Colors are
/// hardcoded (not theme tokens) — port of `web/src/lib/components/Toast.svelte`, which
/// also hardcodes its red `#b3261e` background rather than reading a CSS variable.
pub fn view<'a>(message: &str) -> Element<'a, Message> {
    let bar = container(
        row![
            text(message.to_string())
                .size(13)
                .color(iced::Color::WHITE),
            button(text("\u{d7}").size(15).color(iced::Color::WHITE))
                .on_press(Message::DismissError)
                .padding([0, 6])
                .style(|_theme, _status| button::Style {
                    background: None,
                    text_color: iced::Color::WHITE,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                }),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding([8, 14])
    .style(|_theme| container::Style {
        background: Some(iced::Color::from_rgb8(0xb3, 0x26, 0x1e).into()),
        border: iced::Border {
            radius: 8.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow {
            color: iced::Color::BLACK,
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..container::Style::default()
    });

    container(bar)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(iced::alignment::Vertical::Bottom)
        .padding(iced::Padding {
            bottom: 40.0,
            ..iced::Padding::ZERO
        })
        .into()
}
```

`ui/mod.rs` already declares `pub mod toast;` (added in Task 3, Step 2) — no change needed there.

- [ ] **Step 2: Add `Message::DismissError`** — in `crates/slugline/src/app.rs`, replace:

```rust
    /// The `update_theme` persistence write for a `:theme` switch finished. `target` is
    /// the theme that was applied optimistically; `prev` is what to roll back to on
    /// failure.
    ThemePersisted {
        target: String,
        prev: String,
        res: Result<(), String>,
    },
}
```

with:

```rust
    /// The `update_theme` persistence write for a `:theme` switch finished. `target` is
    /// the theme that was applied optimistically; `prev` is what to roll back to on
    /// failure.
    ThemePersisted {
        target: String,
        prev: String,
        res: Result<(), String>,
    },
    /// The error toast's dismiss (×) button was clicked.
    DismissError,
}
```

- [ ] **Step 3: Clear an expired error on `Tick`** — replace:

```rust
            Message::Tick => {
                if self.loading || self.saving {
                    return Task::none();
                }
                let idle = self
                    .dirty_since
                    .map(|t| t.elapsed() >= SAVE_DEBOUNCE)
                    .unwrap_or(false);
                if idle {
                    return self.spawn_save();
                }
                Task::none()
            }
```

with:

```rust
            Message::Tick => {
                if self.error_expires_at.is_some_and(|t| Instant::now() >= t) {
                    self.error = None;
                    self.error_expires_at = None;
                }
                if self.loading || self.saving {
                    return Task::none();
                }
                let idle = self
                    .dirty_since
                    .map(|t| t.elapsed() >= SAVE_DEBOUNCE)
                    .unwrap_or(false);
                if idle {
                    return self.spawn_save();
                }
                Task::none()
            }
```

- [ ] **Step 4: Handle `DismissError`** — replace:

```rust
            Message::ThemePersisted { target, prev, res } => {
                if let Err(e) = res {
                    // Only roll back if we're still on the theme that failed to persist —
                    // if the user has since switched again, rolling back would clobber
                    // their newer choice.
                    if self.theme == target {
                        self.theme = prev;
                        self.palette = Palette::for_theme(&self.theme, &self.color_overrides);
                    }
                    self.set_error(format!("Failed to save theme: {e}"));
                }
                Task::none()
            }
        }
    }
```

with:

```rust
            Message::ThemePersisted { target, prev, res } => {
                if let Err(e) = res {
                    // Only roll back if we're still on the theme that failed to persist —
                    // if the user has since switched again, rolling back would clobber
                    // their newer choice.
                    if self.theme == target {
                        self.theme = prev;
                        self.palette = Palette::for_theme(&self.theme, &self.color_overrides);
                    }
                    self.set_error(format!("Failed to save theme: {e}"));
                }
                Task::none()
            }
            Message::DismissError => {
                self.error = None;
                self.error_expires_at = None;
                Task::none()
            }
        }
    }
```

- [ ] **Step 5: Render the toast on top of everything in `view()`** — replace:

```rust
        let with_palette: Element<'_, Message> = match &self.editor.command {
            Some(typed) => stack![base, command_palette::view(typed, &self.palette)].into(),
            None => base,
        };
        with_palette
    }
```

with:

```rust
        let with_palette: Element<'_, Message> = match &self.editor.command {
            Some(typed) => stack![base, command_palette::view(typed, &self.palette)].into(),
            None => base,
        };

        // The error toast (design Section 6): floats on top of everything, including the
        // command palette, whenever `error.is_some()`.
        match &self.error {
            Some(message) => stack![with_palette, toast::view(message)].into(),
            None => with_palette,
        }
    }
```

- [ ] **Step 6: Add the tests** — in `#[cfg(test)] mod tests`, insert after
`theme_persisted_failure_does_not_roll_back_a_since_changed_theme`:

```rust
    #[test]
    fn dismiss_error_clears_the_error_and_its_expiry() {
        let (_dir, mut app) = temp_app("2026-06-23");
        app.set_error("boom".to_string());
        assert!(app.error.is_some());
        let _ = app.update(Message::DismissError);
        assert_eq!(app.error, None);
        assert_eq!(app.error_expires_at, None);
    }

    #[test]
    fn tick_clears_an_expired_error_but_leaves_a_fresh_one() {
        let (_dir, mut app) = temp_app("2026-06-23");
        app.set_error("boom".to_string());
        app.error_expires_at = Some(Instant::now() - Duration::from_secs(1)); // already expired
        let _ = app.update(Message::Tick);
        assert_eq!(app.error, None);

        app.set_error("fresh".to_string());
        let _ = app.update(Message::Tick);
        assert_eq!(app.error, Some("fresh".to_string())); // not expired yet
    }
```

- [ ] **Step 7: Run the tests** — `cargo test -p slugline app::tests::`
Expected: PASS (32 tests: the app module's share of the suite, including this task's 2 new ones).

- [ ] **Step 8: Build and run the full suite** — `cargo build -p slugline && cargo test -p slugline`
Expected: builds clean; PASS (51 tests: 49 from Task 6 + 2 new).

- [ ] **Step 9: Commit**

```bash
git add crates/slugline/src/ui/toast.rs crates/slugline/src/app.rs
git commit -m "feat(app): add the error toast with 5s auto-dismiss"
```

---

### Task 8: Workspace hygiene gate + manual smoke

**Files:** none (verification only)

- [ ] **Step 1: Full workspace test** — `cargo test --workspace`
Expected: green — `slugline-core` (164 tests: 158 from before this phase + Task 1's 6 `theme::`
tests) and `slugline` (51 tests: 37 from before this phase + Task 2's 4 + Task 5's 5 + Task 6's 3 +
Task 7's 2).

- [ ] **Step 2: Format + clippy** — `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean. Fix and re-run if needed.

- [ ] **Step 3: Manual smoke — theme switching, persistence, status line, toast**

Run: `cargo run -p slugline -- --notes-dir ./dev-notes --config ./dev-notes/config.toml` (use a
scratch `./dev-notes` directory so this doesn't touch your real notes or `~/.config/slugline`).

Verify all of:
1. First run with no existing `./dev-notes/config.toml`: the window opens light-themed (bright
   background, dark text) — matches the freshly-written `config.toml`'s `theme = "light"` default.
2. A status line is visible at the bottom of the editor pane, showing `-- NORMAL --` plus a context
   breadcrumb that updates as you move the cursor into different sections (e.g. into the `## To
   Do` section shows `To Do`; into a `### <name>` under `## Meetings` shows `Meetings › <name>`).
   Press `i` to enter INSERT — the mode label changes to `-- INSERT --`. Press Escape to return to
   NORMAL.
3. Press `:`, type `theme dark`, press Enter: the whole UI — editor background/text, sidebar,
   status line, tab strip — switches to the dark palette immediately, and the palette overlay
   closes. Quit the app (Cmd/Ctrl-Q or close the window) and reopen it: it opens dark again (open
   `./dev-notes/config.toml` and confirm `theme = "dark"` was written, with the rest of the file's
   structure/comments intact if you'd hand-added any).
4. Press ⌘K, type `theme`, press Enter (bare `:theme`, no argument): the UI toggles back to light
   (confirms the toggle/`next_theme` path, distinct from step 3's explicit `dark`).
5. With the notes directory's write permission temporarily removed (`chmod -w ./dev-notes` on
   macOS/Linux) run `:theme dark` again: a red toast appears at the bottom of the window reading
   "Failed to save theme: ..." with an × dismiss button, and the UI **rolls back** to light (the
   optimistic switch is undone). Click the ×: the toast disappears. Restore permissions
   (`chmod +w ./dev-notes`) afterward.
6. Trigger a different error (e.g. temporarily point `--notes-dir` at a read-only location and
   edit + wait ~1s for autosave to fail): a toast appears, and — without clicking anything — it
   auto-dismisses on its own after 5 seconds.
7. Everything from Phases 2-5's smoke tests still works unmodified: day/tab navigation, calendar/
   agenda/todo sidebar sections, command palette + all `:` commands, shared register,
   flush-before-navigate, autosave, flush-on-exit — now rendering in whichever theme is currently
   active rather than always dark.

- [ ] **Step 4: Commit any fixups**

```bash
git add -A
git commit -m "chore: fmt + clippy clean for phase 6" || echo "nothing to commit"
```

---

## Self-Review (performed while writing this plan)

- **Verification method:** every non-trivial piece of this plan (the `Tokens` type alias as a
  direct `BTreeMap<String, String>` merge target for `UiConfig::colors`, `Palette::for_theme`'s hex
  parsing and its `Color::BLACK` fallback, threading `&Palette` through seven `ui/*.rs` files
  without breaking any existing widget's `move` closure captures, `switch_theme`'s stale-rollback
  guard, the `stack!` toast-over-palette-over-base ordering, `Tick`'s expiry check) was implemented
  in a disposable scratch worktree off the real `master` tip (`d920d43`), compiled, tested (`cargo
  test --workspace`: 164 `slugline-core` tests + 51 `slugline` tests, all green, up from 158 + 37),
  formatted, linted (`cargo clippy --workspace --all-targets -- -D warnings`: clean), and
  smoke-run (`cargo run -p slugline`, confirmed the window opens, writes a default `theme = "light"`
  config, materializes today's note, and there is no panic) before being copied into this
  document. The scratch worktree was then discarded — none of this work is committed on `master`
  yet; that's what executing this plan does.
- **Spec coverage:** implements design Section 4's "Theming" paragraph in full — `core` owns token
  structs + light/dark palettes + config overrides (`core::theme`); the UI adapts them to Iced
  colors (`theme_iced::Palette`, this port's stand-in for the design's sketched
  `iced::Theme::Custom`, see "Deferred on purpose"); `:theme` flows effect → `update` swaps
  `theme`/`palette` → next `view` uses it → persists via the existing comment-preserving
  `toml_edit` writer (`switch_theme`/`ThemePersisted`). Implements design Section 6's error
  handling in full for the pieces this phase owns: "Model holds `error: Option<String>` + an
  expiry; rendered as a toast via `stack`" (`error_expires_at`, `ui/toast.rs`, the `view()` `stack!`
  order), "`Tick` clears it once expired" (Task 7 Step 3), and "Theme-persist failure → roll back
  `config.theme` in the Model, re-render + toast" (`switch_theme`'s rollback branch in
  `Message::ThemePersisted`). Implements roadmap Phase 6's exact scope line: "light/dark palettes,
  `:theme` switch + comment-preserving persistence, status line, toast/error surface" — all four
  clauses have a task (1-3 for palettes, 5 for the switch+persistence, 6 for the status line, 7 for
  the toast).
- **Type consistency (against the real committed/verified code):** `Palette`'s 13 fields exactly
  match the union of every color the old `ui::palette` module had (`BG`→`bg`, `FG`→`fg`,
  `MUTED`→`muted`, `CURSOR`→`cursor`, `EDIT_BAR_BG`→`edit_bar_bg`, `RULE`→`rule`,
  `HIGHLIGHT_BG`→`highlight_bg`, `LINK`→ consolidated into `accent` since no web `--link` token
  ever existed for it to diverge from, `TODO_DONE`→`todo_done`, `BLOCKQUOTE_BORDER`→
  `blockquote_border`, `ACCENT`→`accent`, `STATUS_BAR`→`status_bar`, `HEADING`→`heading`) — every
  call site in Task 3's seven rewritten files reads only fields defined in Task 2's `Palette`.
  `AppEffect::Theme(String)` (already existing since Phase 1b/5) is matched exactly once, in
  `run_effect`, and its `String` payload flows unchanged into `switch_theme(arg: String)`.
  `Message::ThemePersisted { target: String, prev: String, res: Result<(), String> }` matches the
  design Section 2 sketch's field names and is constructed with matching field names in
  `switch_theme` and consumed with matching field names in `update()`.
- **Placeholder scan:** no `todo!()`/TODO/"handle later" in any shipped code; every step shows the
  exact code to write, not a description of it. The one explicit `// wired in Phase 6` placeholder
  this plan exists to resolve (`crates/slugline/src/app.rs`'s pre-Phase-6
  `AppEffect::Theme(_) => Task::none()`) is removed in Task 5, Step 2.
- **Deliberate divergences (noted in-plan):** (1) `iced::Theme::Custom` is not adopted — every
  widget already ignored the Iced-native `_theme` argument pre-Phase-6, so this phase generalizes
  that existing pattern (a `Palette` value instead of hardcoded constants) rather than migrating
  onto Iced's own theme propagation; noted under "Deferred on purpose" with its rationale. (2) the
  `--link` web token never existed, so link coloring is consolidated onto `--accent` in
  `editor_pane.rs`'s `inline()`, documented inline at the change site. (3) `--todo-done`'s absence
  from the web's in-code `DARK` object (a CSS-cascade accident that happened to render identically
  to `LIGHT`'s value in practice) is made explicit in `core::theme::dark()` rather than silently
  reproduced as a gap, documented in that function's doc comment.
- **Reachability:** `Message::ThemePersisted` is reachable via `switch_theme`'s `Task::perform`
  every time `:theme` runs (which Phase 5 already made reachable via `:`, the command palette, and
  ⌘K); `Message::DismissError` is reachable via the toast's × button, itself reachable any time
  `error.is_some()` (already-reachable failure paths: save failures, load failures, and this
  phase's new theme-persist failures) — no dead code waiting on a later phase.

---

## Handoff to Phase 7

Phase 6 is the last phase before cutover. Once this plan's Task 8 is green, the roadmap's Phase 7
("Cutover: delete `web/`, `src/app.rs`, `src/assets.rs`, and the axum/`rust-embed`/`mime_guess`/
`tower` deps; tag `web-final`; finalize CLI; update README/docs") has everything it needs: every
design-Section-4 native-refined feature (pane_grid sidebar, command palette, theming) and every
Section 6 error-handling behavior this port commits to are in place and covered by tests.

