# Phase 5 — Command Mode & the Fuzzy Command Palette — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **This is a port, with one new native-refined piece on top.** Behavioral truth for the ported
> pieces is `web/src/lib/doc/context.ts` (+ `context.test.ts`), `web/src/lib/doc/command.ts` (+
> `command.test.ts`), and `web/src/lib/editor/commands.ts` (+ `commands.test.ts`), plus
> `web/src/lib/editor/keymap.ts`'s `handleCommandMode` (covered by
> `keymap.test.ts`'s `': opens the command line and Enter runs it'` case). The command **palette
> overlay** itself (design Section 4) has no web counterpart — the web app had a bottom
> command-line, retired by this port — so its UI and its fuzzy-matcher are a fresh implementation
> with fresh tests, same rationale as Phase 3's `shift_month` and Phase 4's 7-day aggregation.
>
> **Iced API caution:** method/type names target iced `0.13.x` (`stack!`, `container::align_x`/
> `align_y`, `iced::Padding`). Every non-trivial API call in this plan was implemented in a
> disposable scratch worktree off the real `iced-port` tip, compiled, tested (`cargo test
> --workspace`: 158 `slugline-core` tests + 34 `slugline` tests, all green), formatted, linted
> (`cargo clippy --workspace --all-targets -- -D warnings`: clean), and smoke-run (`cargo run -p
> slugline`, confirmed the window opens, a note materializes on disk, and there is no panic) before
> being copied into this document — but if a signature has drifted in your checkout, confirm it and
> adjust; the *intent* is the contract.

**Goal:** Wire up `:` command mode end-to-end — parsing, validation, the full set of buffer-mutating
and navigation commands (`meeting/note/section/todo/start/end/scheduled/purpose/topic/people` plus
`goto/today/tab/close/w/theme`) — and present it through a floating, fuzzy-filtered command palette
overlay instead of the web's retired bottom command-line, with a `Cmd/Ctrl-K` shortcut that seeds it
from anywhere.

**Architecture:** Three new `slugline-core` modules complete the ported engine:
`doc::context` (what block/section the cursor is in — needed by several commands),
`doc::command` (parse + validate the typed `:` line into a `CommandName`/`arg` pair), and
`editor::commands` (the section/block/meta-mutating helpers plus `run_command`, which turns a
validated command into a new `EditorState` and, for the navigation/save/theme commands, the
existing `AppEffect` — no new `AppEffect` variants are needed; Phase 2 already ported all of them).
`editor::keymap::handle_key` gains a third parameter, `ctx: &CommandCtx` (today's `nowHHMM`, for
`:start`/`:end`), and a new branch: whenever `state.command.is_some()`, keys are routed to a new
`handle_command_mode` (Escape clears it, Backspace shortens it, printable characters append to it,
Enter calls `run_command`) instead of NORMAL/INSERT handling; `:` in NORMAL mode now opens it
(previously a no-op placeholder). On the UI side, one new file, `ui/command_palette.rs`, renders a
top-centered overlay — the typed `:command` plus a fuzzy-filtered, click-to-autocomplete list of
every known command — stacked on top of the base view via `iced::widget::stack!` whenever
`editor.command.is_some()`. `app.rs` threads a freshly-built `CommandCtx` into every `handle_key`
call, adds a `Cmd/Ctrl-K` interception in the keyboard subscription that bypasses the vim keymap
entirely (`Message::OpenPalette`), and a click handler for palette suggestions
(`Message::PaletteSuggestionClicked`).

**Tech Stack:** Rust, existing `slugline-core` (`doc::scan`, `doc::classify`, `editor::state`),
Iced `0.13.x` `stack!`/`container` alignment for the overlay. No new crate dependencies — the fuzzy
matcher is a small hand-rolled subsequence scorer (see Task 7's rationale), matching the project's
existing preference for straightforward hand-written logic over pulling in a crate for something
this size.

---

## Prerequisites

- **Phases 0, 1a, 1b, 1c, 2, 3, 4 are complete and committed on `iced-port`, and `cargo test
  --workspace` is green** (113 `slugline-core` tests + 28 tests in `slugline`, as of `1239b0d
  "feat(app): sidebar Agenda + 7-day To Do aggregation, click-to-navigate"`). Phase 5 builds
  directly on `crates/slugline-core/src/editor/{state,keymap}.rs` (`EditorState.command:
  Option<String>` already exists as an unused field — Phase 1b added it for signature stability),
  `crates/slugline-core/src/doc/scan.rs` (`Section`/`Block`/`SectionKind`/`scan_document`, Phase 4),
  `crates/slugline-core/src/dates.rs` (`add_days`, Phase 2), and `crates/slugline/src/app.rs`'s
  `run_effect`/`plan_tabs` (Phase 2/4 — already handle every `AppEffect` variant this phase's
  `run_command` can emit, including a `Task::none()` stub for `AppEffect::Theme` explicitly
  commented `// wired in Phase 6`).

## Scope

**In this phase:**
- `core::doc::context`: `Context` enum (`None`/`Todo`/`Meeting`/`Note`/`Other`), `resolve_context`,
  `nearest_heading_level` — a direct port of `web/src/lib/doc/context.ts`, tested against the same
  `fixtures/full-day.md`/`fixtures/subsections.md` Phase 3/4 already added.
- `core::doc::command`: `CommandName` (16 variants), `ArgKind`, `CommandSpec`, the `COMMANDS` table,
  `parse_command_line`, `validate_command` (with the `p` → `people` alias) — a port of
  `web/src/lib/doc/command.ts`.
- `core::dates::now_hhmm()` — today's local time as `HH:MM`, feeding `CommandCtx` the way
  `appState.svelte.ts`'s inline `nowHHMM(this.now)` does. No TS test exists for this formatting
  helper in isolation (it's inlined in the web); this phase adds a fresh regex-shaped test, same as
  Phase 4's `todo_dates_to_read`.
- `core::editor::commands`: the section/block/meta helpers (`append_block`,
  `append_line_to_section`, `upsert_meta`, `append_meta`, `ensure_section`,
  `end_of_enclosing_section`), `CommandCtx`, `CommandResult`, and `run_command` — a port of
  `web/src/lib/editor/commands.ts`, covering all 16 commands including `:people`/`:p`.
- `core::editor::keymap`: `handle_key` gains a `ctx: &CommandCtx` parameter; a new
  `handle_command_mode` branch (checked first, before mode dispatch) handles Escape/Enter/
  Backspace/printable-character input while `state.command.is_some()`; NORMAL mode's `:` now opens
  command mode (`state.command = Some(String::new())`) instead of falling through as a no-op.
- `ui/command_palette.rs`: the floating overlay — typed text plus a fuzzy-filtered, capped list of
  suggestions (name, argument-kind hint, one-line description), each clickable to autocomplete the
  command name into the buffer. A hand-rolled case-insensitive subsequence `fuzzy_score` (pure,
  tested) ranks matches.
- `app.rs`: `Message::Key` now builds a `CommandCtx { now_hhmm: now_hhmm() }` per keystroke and
  passes it to `handle_key`; two new `Message` variants — `OpenPalette` (`Cmd/Ctrl-K`, intercepted
  in the keyboard subscription ahead of the normal `Message::Key` path) and
  `PaletteSuggestionClicked(String)` (seeds `editor.command` with `"{name} "`); `view()` wraps the
  existing base layout in `stack![base, command_palette::view(typed)]` whenever
  `editor.command.is_some()`.

**Deferred on purpose:**
- **Actually applying `:theme`** (swapping the live palette, persisting via `toml_edit`). This
  phase's `run_command` already returns `AppEffect::Theme(arg)` for a validated `:theme` command —
  that plumbing has existed since Phase 2 — but `run_effect`'s `AppEffect::Theme(_) => Task::none()`
  stub is untouched here. The roadmap's Phase 6 ("Theming & polish") owns making that effect do
  something; this phase only needs `:theme`/`:theme dark` to parse, validate, and close the palette
  cleanly, exactly like every other command.
- **Displaying `editor.message`** (the validation-error/status text `run_command` sets on failure,
  e.g. `"Unknown command: :xyz"`, `"Not in a meeting"`). `run_command` always clears
  `editor.command` back to `None` before or as it sets `message` — by the time an error is visible,
  the palette has already closed — so there is no in-palette moment that needs it. The roadmap's
  Phase 6 owns the status line that will eventually surface it; nothing in this phase silently
  swallows an error that would otherwise be shown.
- **Backdrop-click-to-dismiss** for the palette. Escape already closes it; a click-outside handler
  is a `mouse_area` layered under the overlay for a secondary dismissal path the roadmap doesn't
  call out, same "don't add untested secondary interactions" reasoning Phase 3/4 used for
  modifier-click-to-open-in-new-tab.
- **Keyboard navigation within the suggestion list** (arrow keys to highlight, Tab to accept). The
  design calls for "a fuzzy-filtered list", not list navigation; clicking a suggestion already
  autocompletes it, and typing more characters is the primary filtering path (a vim modal editor's
  users are typists, not mouse-first). Revisit only if it turns out to matter in practice.

---

## File Structure (files added/changed in Phase 5)

```
crates/slugline-core/
  src/
    dates.rs                        # + now_hhmm() (tested)
    doc/
      mod.rs                        # + pub mod command; pub mod context; re-exports
      context.rs                    # NEW: Context, resolve_context(), nearest_heading_level() (tested)
      command.rs                    # NEW: CommandName/ArgKind/CommandSpec/COMMANDS, parse_command_line(),
                                     #      validate_command() (tested)
    editor/
      mod.rs                        # + pub mod commands; re-exports
      commands.rs                   # NEW: append_block/append_line_to_section/upsert_meta/append_meta/
                                     #      ensure_section/end_of_enclosing_section, CommandCtx,
                                     #      CommandResult, run_command() (tested)
      keymap.rs                     # REWRITE: handle_key(..., ctx), handle_command_mode(), ':' opens it

crates/slugline/
  src/
    app.rs                          # REWRITE: CommandCtx wiring, OpenPalette/PaletteSuggestionClicked,
                                     #          Cmd/Ctrl-K interception, stack! overlay in view()
    ui/mod.rs                       # + pub mod command_palette;
    ui/command_palette.rs           # NEW: palette overlay + fuzzy_score()/filter_commands() (tested)
```

---

### Task 1: Port `resolve_context`/`nearest_heading_level` into `core::doc::context`

**Files:**
- Create: `crates/slugline-core/src/doc/context.rs`
- Modify: `crates/slugline-core/src/doc/mod.rs`

- [ ] **Step 1: Write the failing tests** — create `crates/slugline-core/src/doc/context.rs` with
the type, both functions, and their ported tests (read `fixtures/*.md` at test time, the same files
`web/src/lib/doc/context.test.ts` uses via `fixtureLines`):

```rust
use super::scan::{Block, DocModel, Section, SectionKind};

/// What the cursor's current line is "inside", for commands that write relative to it
/// (`:scheduled`, `:todo`'s meeting-tag suffix, `:section`'s nesting level, ...). Port of
/// `web/src/lib/doc/context.ts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Context {
    None,
    Todo { section: Section },
    Meeting { block: Block, section: Section },
    Note { block: Block, section: Section },
    Other { section: Section },
}

/// Which top-level section (and, for Meetings/Notes, which H3 block) `line_index` falls
/// inside. Never panics: a `line_index` outside every section yields `Context::None`.
pub fn resolve_context(model: &DocModel, line_index: usize) -> Context {
    let Some(section) = model
        .sections
        .iter()
        .find(|s| line_index >= s.start_line && line_index <= s.end_line)
    else {
        return Context::None;
    };

    match section.kind {
        SectionKind::Todo => Context::Todo {
            section: section.clone(),
        },
        SectionKind::Meetings | SectionKind::Notes => {
            let block = section
                .blocks
                .iter()
                .find(|b| line_index >= b.start_line && line_index <= b.end_line);
            match (section.kind, block) {
                (SectionKind::Meetings, Some(b)) => Context::Meeting {
                    block: b.clone(),
                    section: section.clone(),
                },
                (SectionKind::Notes, Some(b)) => Context::Note {
                    block: b.clone(),
                    section: section.clone(),
                },
                _ => Context::Other {
                    section: section.clone(),
                },
            }
        }
        SectionKind::Other => Context::Other {
            section: section.clone(),
        },
    }
}

/// The level (1-6) of the nearest heading at or above `line_index`, or `None` if there
/// is no heading above it. Used by `:section` to nest one level deeper than whatever
/// heading encloses the cursor.
pub fn nearest_heading_level(lines: &[String], line_index: usize) -> Option<u8> {
    if lines.is_empty() {
        return None;
    }
    let start = line_index.min(lines.len() - 1);
    for i in (0..=start).rev() {
        if let super::classify::Line::Heading { level, .. } =
            super::classify::classify_line(&lines[i])
        {
            return Some(level);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::scan_document;

    fn fixture_lines(name: &str) -> Vec<String> {
        let path = format!("{}/../../fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"))
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn returns_the_meeting_when_the_cursor_is_inside_an_h3_under_meetings() {
        let lines = fixture_lines("full-day.md");
        let model = scan_document(&lines);
        let meetings = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Meetings)
            .unwrap();
        let sync = &meetings.blocks[0];
        match resolve_context(&model, sync.heading_line_index + 1) {
            Context::Meeting { block, .. } => assert_eq!(block.name, "Weekly Sync"),
            other => panic!("expected Context::Meeting, got {other:?}"),
        }
    }

    #[test]
    fn returns_the_note_when_the_cursor_is_inside_an_h3_under_notes() {
        let lines = fixture_lines("full-day.md");
        let model = scan_document(&lines);
        let notes = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Notes)
            .unwrap();
        let arch = &notes.blocks[0];
        match resolve_context(&model, arch.heading_line_index + 1) {
            Context::Note { .. } => {}
            other => panic!("expected Context::Note, got {other:?}"),
        }
    }

    #[test]
    fn returns_todo_when_the_cursor_is_inside_the_to_do_section() {
        let lines = fixture_lines("full-day.md");
        let model = scan_document(&lines);
        let todo = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Todo)
            .unwrap();
        match resolve_context(&model, todo.heading_line_index + 1) {
            Context::Todo { .. } => {}
            other => panic!("expected Context::Todo, got {other:?}"),
        }
    }

    #[test]
    fn returns_none_when_the_cursor_is_on_the_title_line() {
        let lines = fixture_lines("full-day.md");
        let model = scan_document(&lines);
        assert_eq!(resolve_context(&model, 0), Context::None);
    }

    #[test]
    fn nearest_heading_level_finds_the_level_of_the_nearest_enclosing_heading() {
        let lines = fixture_lines("subsections.md");
        let idx = lines.iter().position(|l| l == "Cut scope.").unwrap();
        // Inside the "Mitigations" (H5) area -> nearest heading level is 5.
        assert_eq!(nearest_heading_level(&lines, idx), Some(5));
    }

    #[test]
    fn nearest_heading_level_returns_none_above_any_heading() {
        let lines = vec![String::new(), "no heading yet".to_string()];
        assert_eq!(nearest_heading_level(&lines, 1), None);
    }
}
```

- [ ] **Step 2: Declare the module** — in `crates/slugline-core/src/doc/mod.rs`, replace:

```rust
pub mod classify;
pub mod render_inline;
pub mod scan;

pub use classify::{Line, classify_line};
pub use render_inline::{Span, render_inline};
pub use scan::{Block, DocModel, MetaEntry, Section, SectionKind, scan_document};
```

with:

```rust
pub mod classify;
pub mod context;
pub mod render_inline;
pub mod scan;

pub use classify::{Line, classify_line};
pub use context::{Context, nearest_heading_level, resolve_context};
pub use render_inline::{Span, render_inline};
pub use scan::{Block, DocModel, MetaEntry, Section, SectionKind, scan_document};
```

- [ ] **Step 3: Run the tests** — `cargo test -p slugline-core doc::context::`
Expected: PASS (6 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/slugline-core/src/doc/context.rs crates/slugline-core/src/doc/mod.rs
git commit -m "feat(core): port resolveContext/nearestHeadingLevel"
```

---

### Task 2: Port `parseCommandLine`/`validateCommand` into `core::doc::command`

**Files:**
- Create: `crates/slugline-core/src/doc/command.rs`
- Modify: `crates/slugline-core/src/doc/mod.rs`

- [ ] **Step 1: Write the failing tests** — create `crates/slugline-core/src/doc/command.rs`:

```rust
use crate::date::is_valid_date;

/// The text typed after the leading `:`, split into a lowercased name and the rest of
/// the line (trimmed). Port of `web/src/lib/doc/command.ts` `parseCommandLine`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    pub name: String,
    pub arg: String,
}

/// Parse the text typed after the leading `:` (the colon is not included).
pub fn parse_command_line(input: &str) -> ParsedCommand {
    let trimmed = input.trim_start();
    match trimmed.find(' ') {
        None => ParsedCommand {
            name: trimmed.to_lowercase(),
            arg: String::new(),
        },
        Some(sp) => ParsedCommand {
            name: trimmed[..sp].to_lowercase(),
            arg: trimmed[sp + 1..].trim().to_string(),
        },
    }
}

/// Every recognized `:` command. Mirrors `web/src/lib/doc/command.ts`'s `CommandName` union.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Meeting,
    Note,
    Section,
    Todo,
    Start,
    End,
    Scheduled,
    Purpose,
    Topic,
    People,
    Goto,
    Today,
    Tab,
    Close,
    W,
    Theme,
}

impl CommandName {
    /// The typed name that resolves to this command, lowercase, before alias resolution.
    pub fn canonical(self) -> &'static str {
        match self {
            CommandName::Meeting => "meeting",
            CommandName::Note => "note",
            CommandName::Section => "section",
            CommandName::Todo => "todo",
            CommandName::Start => "start",
            CommandName::End => "end",
            CommandName::Scheduled => "scheduled",
            CommandName::Purpose => "purpose",
            CommandName::Topic => "topic",
            CommandName::People => "people",
            CommandName::Goto => "goto",
            CommandName::Today => "today",
            CommandName::Tab => "tab",
            CommandName::Close => "close",
            CommandName::W => "w",
            CommandName::Theme => "theme",
        }
    }
}

/// The kind of argument a command expects, and how it is validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    None,
    Text,
    Time,
    Date,
    Theme,
}

/// A command's shape: which argument kind it takes and whether that argument is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: CommandName,
    pub arg_kind: ArgKind,
    pub arg_required: bool,
}

/// Every command, in the canonical order the command palette lists them. Mirrors
/// `web/src/lib/doc/command.ts`'s `COMMANDS` (a `Record`, whose `Object.keys` order is
/// insertion order — preserved here as array order).
pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: CommandName::Meeting,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Note,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Section,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Todo,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Start,
        arg_kind: ArgKind::None,
        arg_required: false,
    },
    CommandSpec {
        name: CommandName::End,
        arg_kind: ArgKind::None,
        arg_required: false,
    },
    CommandSpec {
        name: CommandName::Scheduled,
        arg_kind: ArgKind::Time,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Purpose,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Topic,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::People,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Goto,
        arg_kind: ArgKind::Date,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Today,
        arg_kind: ArgKind::None,
        arg_required: false,
    },
    CommandSpec {
        name: CommandName::Tab,
        arg_kind: ArgKind::Date,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Close,
        arg_kind: ArgKind::None,
        arg_required: false,
    },
    CommandSpec {
        name: CommandName::W,
        arg_kind: ArgKind::None,
        arg_required: false,
    },
    CommandSpec {
        name: CommandName::Theme,
        arg_kind: ArgKind::Theme,
        arg_required: false,
    },
];

/// Look up a command's spec by its `CommandName` (not the typed/aliased string — see
/// `lookup` for that). Used by the command palette to show each command's argument kind.
pub fn spec_for(name: CommandName) -> &'static CommandSpec {
    COMMANDS
        .iter()
        .find(|s| s.name.canonical() == name.canonical())
        .expect("every CommandName has a COMMANDS entry")
}

fn lookup(typed_name: &str) -> Option<&'static CommandSpec> {
    COMMANDS.iter().find(|s| s.name.canonical() == typed_name)
}

/// Short aliases resolved before `COMMANDS` lookup. Add future shortcuts here.
fn resolve_alias(typed_name: &str) -> &str {
    match typed_name {
        "p" => "people",
        other => other,
    }
}

/// The outcome of validating a typed command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    Ok { command: CommandName, arg: String },
    Err { error: String },
}

fn validate_arg(kind: ArgKind, arg: &str) -> Option<&'static str> {
    match kind {
        ArgKind::None | ArgKind::Text => None,
        ArgKind::Time => {
            let ok = TIME_RE.is_match(arg);
            if ok { None } else { Some("Expected HH:MM") }
        }
        ArgKind::Date => {
            if is_valid_date(arg) {
                None
            } else {
                Some("Expected YYYY-MM-DD")
            }
        }
        ArgKind::Theme => {
            if arg.is_empty() || arg == "light" || arg == "dark" {
                None
            } else {
                Some("Expected light or dark")
            }
        }
    }
}

use std::sync::LazyLock;

use regex::Regex;

static TIME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([01]\d|2[0-3]):[0-5]\d$").unwrap());

/// Validate a full typed command line (the text after `:`, colon excluded). Port of
/// `web/src/lib/doc/command.ts` `validateCommand`.
pub fn validate_command(input: &str) -> ValidationResult {
    let ParsedCommand { name, arg } = parse_command_line(input);
    let resolved = resolve_alias(&name);
    let Some(spec) = lookup(resolved) else {
        return ValidationResult::Err {
            error: format!("Unknown command: :{name}"),
        };
    };

    if spec.arg_required && arg.is_empty() {
        return ValidationResult::Err {
            error: format!(":{name} requires an argument"),
        };
    }
    if let Some(error) = validate_arg(spec.arg_kind, &arg) {
        return ValidationResult::Err {
            error: error.to_string(),
        };
    }

    ValidationResult::Ok {
        command: spec.name,
        arg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_name_and_rest_of_line_argument() {
        let p = parse_command_line("meeting Daily Standup");
        assert_eq!(p.name, "meeting");
        assert_eq!(p.arg, "Daily Standup");
    }

    #[test]
    fn lowercases_the_name_and_handles_no_arg_commands() {
        let p = parse_command_line("Today");
        assert_eq!(p.name, "today");
        assert_eq!(p.arg, "");
    }

    #[test]
    fn accepts_a_valid_text_command() {
        let r = validate_command("meeting Weekly Sync");
        assert_eq!(
            r,
            ValidationResult::Ok {
                command: CommandName::Meeting,
                arg: "Weekly Sync".to_string(),
            }
        );
    }

    #[test]
    fn rejects_unknown_commands() {
        match validate_command("meetng x") {
            ValidationResult::Err { error } => assert!(error.contains("Unknown command")),
            other => panic!("expected Err, got {other:?}"),
        }
    }

    #[test]
    fn requires_arguments_where_mandated() {
        assert!(matches!(
            validate_command("todo"),
            ValidationResult::Err { .. }
        ));
    }

    #[test]
    fn validates_hh_mm_for_scheduled() {
        assert!(matches!(
            validate_command("scheduled 14:30"),
            ValidationResult::Ok { .. }
        ));
        assert!(matches!(
            validate_command("scheduled 25:00"),
            ValidationResult::Err { .. }
        ));
    }

    #[test]
    fn validates_yyyy_mm_dd_for_goto() {
        assert!(matches!(
            validate_command("goto 2026-06-23"),
            ValidationResult::Ok { .. }
        ));
        assert!(matches!(
            validate_command("goto 2026-13-01"),
            ValidationResult::Err { .. }
        ));
    }

    #[test]
    fn validates_theme_values() {
        assert!(matches!(
            validate_command("theme dark"),
            ValidationResult::Ok { .. }
        ));
        assert!(matches!(
            validate_command("theme neon"),
            ValidationResult::Err { .. }
        ));
    }

    #[test]
    fn accepts_no_arg_commands() {
        assert!(matches!(
            validate_command("start"),
            ValidationResult::Ok { .. }
        ));
        assert!(matches!(
            validate_command("close"),
            ValidationResult::Ok { .. }
        ));
    }

    #[test]
    fn allows_theme_with_no_argument_toggle() {
        match validate_command("theme") {
            ValidationResult::Ok { arg, .. } => assert_eq!(arg, ""),
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn p_alice_resolves_to_command_people_via_validate_command() {
        match validate_command("p Alice Smith") {
            ValidationResult::Ok { command, arg } => {
                assert_eq!(command.canonical(), "people");
                assert_eq!(arg, "Alice Smith");
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn p_with_no_argument_fails_validation() {
        assert!(matches!(
            validate_command("p"),
            ValidationResult::Err { .. }
        ));
    }

    #[test]
    fn people_resolves_directly() {
        match validate_command("people Bob Jones") {
            ValidationResult::Ok { command, arg } => {
                assert_eq!(command.canonical(), "people");
                assert_eq!(arg, "Bob Jones");
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn every_command_name_has_a_spec() {
        for spec in COMMANDS {
            assert_eq!(spec_for(spec.name).name.canonical(), spec.name.canonical());
        }
    }
}
```

- [ ] **Step 2: Declare the module** — in `crates/slugline-core/src/doc/mod.rs`, replace:

```rust
pub mod classify;
pub mod context;
pub mod render_inline;
pub mod scan;

pub use classify::{Line, classify_line};
pub use context::{Context, nearest_heading_level, resolve_context};
pub use render_inline::{Span, render_inline};
pub use scan::{Block, DocModel, MetaEntry, Section, SectionKind, scan_document};
```

with:

```rust
pub mod classify;
pub mod command;
pub mod context;
pub mod render_inline;
pub mod scan;

pub use classify::{Line, classify_line};
pub use command::{
    ArgKind, COMMANDS, CommandName, CommandSpec, ParsedCommand, ValidationResult,
    parse_command_line, spec_for, validate_command,
};
pub use context::{Context, nearest_heading_level, resolve_context};
pub use render_inline::{Span, render_inline};
pub use scan::{Block, DocModel, MetaEntry, Section, SectionKind, scan_document};
```

- [ ] **Step 3: Run the tests** — `cargo test -p slugline-core doc::command::`
Expected: PASS (14 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/slugline-core/src/doc/command.rs crates/slugline-core/src/doc/mod.rs
git commit -m "feat(core): port parseCommandLine/validateCommand"
```

---

### Task 3: Add `core::dates::now_hhmm`

**Files:**
- Modify: `crates/slugline-core/src/dates.rs`

- [ ] **Step 1: Write the failing test** — in `crates/slugline-core/src/dates.rs`, inside `mod
tests`, add (right after `today_iso_is_a_valid_yyyy_mm_dd`):

```rust
    #[test]
    fn now_hhmm_is_zero_padded_hh_mm() {
        let t = now_hhmm();
        let re = regex::Regex::new(r"^([01]\d|2[0-3]):[0-5]\d$").unwrap();
        assert!(re.is_match(&t), "expected HH:MM, got {t:?}");
    }
```

- [ ] **Step 2: Run it to confirm it fails to compile** — `cargo test -p slugline-core
dates::tests::now_hhmm`
Expected: FAIL — `now_hhmm` is not defined yet.

- [ ] **Step 3: Implement `now_hhmm`** — in `crates/slugline-core/src/dates.rs`, right after
`today_iso`:

```rust
/// Today's date in the local timezone, formatted `YYYY-MM-DD`.
pub fn today_iso() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// The current local time, formatted `HH:MM` (24-hour, zero-padded). Feeds `CommandCtx`
/// for `:start`/`:end`, mirroring the web's inline `nowHHMM(this.now)` in
/// `appState.svelte.ts`.
pub fn now_hhmm() -> String {
    Local::now().format("%H:%M").to_string()
}
```

- [ ] **Step 4: Run the test** — `cargo test -p slugline-core dates::tests::now_hhmm`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/slugline-core/src/dates.rs
git commit -m "feat(core): add now_hhmm() for :start/:end"
```

---

### Task 4: Port `runCommand` and its helpers into `core::editor::commands`

**Files:**
- Create: `crates/slugline-core/src/editor/commands.rs`
- Modify: `crates/slugline-core/src/editor/mod.rs`

- [ ] **Step 1: Write the failing tests** — create `crates/slugline-core/src/editor/commands.rs`:

```rust
use crate::doc::{
    Block, CommandName, Context, Line, Section, SectionKind, ValidationResult, classify_line,
    nearest_heading_level, resolve_context, scan_document, validate_command,
};

use super::keymap::AppEffect;
use super::state::{Cursor, EditorState, clamp_cursor, push_undo};

/// Append an H3 (or any heading text) at the end of a section's content.
pub fn append_block(lines: &[String], section: &Section, heading: &str) -> (Vec<String>, usize) {
    let idx = section.end_line + 1;
    let mut out = lines.to_vec();
    out.splice(idx..idx, [heading.to_string(), String::new()]);
    (out, idx)
}

/// Append a single line after the last non-blank line of a section (or right after its
/// heading).
pub fn append_line_to_section(
    lines: &[String],
    section: &Section,
    text: &str,
) -> (Vec<String>, usize) {
    let mut insert_at = section.start_line + 1;
    for i in (section.start_line + 1)..=section.end_line {
        if !lines.get(i).map(|l| l.trim()).unwrap_or("").is_empty() {
            insert_at = i + 1;
        }
    }
    let mut out = lines.to_vec();
    out.insert(insert_at, text.to_string());
    (out, insert_at)
}

/// Insert or update a `meta:key value` line within a block's meta region.
pub fn upsert_meta(
    lines: &[String],
    block: &Block,
    key: &str,
    value: &str,
) -> (Vec<String>, usize) {
    let meta_line = format!("meta:{key} {value}");
    let mut out = lines.to_vec();
    if let Some(existing) = block.meta.iter().find(|m| m.key == key) {
        let line_index = existing.line_index;
        out[line_index] = meta_line;
        return (out, line_index);
    }
    let insert_at = block.meta_end_line + 1; // meta_end_line == heading when no meta yet
    out.insert(insert_at, meta_line);
    (out, insert_at)
}

/// Append a new value to an existing `meta:key` line, or create it if absent. Values are
/// joined with `", "`.
pub fn append_meta(
    lines: &[String],
    block: &Block,
    key: &str,
    new_value: &str,
) -> (Vec<String>, usize) {
    let trimmed = new_value.trim();
    if let Some(existing) = block.meta.iter().find(|m| m.key == key) {
        let existing_value = existing.value.trim();
        if !existing_value.is_empty() {
            let combined = format!("{existing_value}, {trimmed}");
            return upsert_meta(lines, block, key, &combined);
        }
    }
    upsert_meta(lines, block, key, trimmed)
}

/// The three standard top-level sections a note may need created on demand, in the
/// canonical order they appear when all three are present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardSection {
    Todo,
    Meetings,
    Notes,
}

const STANDARD_SECTION_ORDER: [StandardSection; 3] = [
    StandardSection::Todo,
    StandardSection::Meetings,
    StandardSection::Notes,
];

impl StandardSection {
    fn heading(self) -> &'static str {
        match self {
            StandardSection::Todo => "## To Do",
            StandardSection::Meetings => "## Meetings",
            StandardSection::Notes => "## Notes",
        }
    }

    fn matches(self, kind: SectionKind) -> bool {
        matches!(
            (self, kind),
            (StandardSection::Todo, SectionKind::Todo)
                | (StandardSection::Meetings, SectionKind::Meetings)
                | (StandardSection::Notes, SectionKind::Notes)
        )
    }
}

/// Ensure a standard section exists; if missing, insert it in canonical order.
pub fn ensure_section(lines: &[String], kind: StandardSection) -> (Vec<String>, Section) {
    let model = scan_document(lines);
    if let Some(found) = model.sections.iter().find(|s| kind.matches(s.kind)) {
        return (lines.to_vec(), found.clone());
    }

    let order_idx = STANDARD_SECTION_ORDER
        .iter()
        .position(|&k| k == kind)
        .expect("kind is a StandardSection variant");
    let mut insert_at = lines.len();
    let mut placed = false;
    for prev_kind in STANDARD_SECTION_ORDER[..order_idx].iter().rev() {
        if let Some(prev) = model.sections.iter().find(|s| prev_kind.matches(s.kind)) {
            insert_at = prev.end_line + 1;
            placed = true;
            break;
        }
    }
    if !placed {
        insert_at = model.title_line_index.map_or(0, |i| i + 1);
    }

    let mut out = lines.to_vec();
    out.splice(
        insert_at..insert_at,
        [String::new(), kind.heading().to_string(), String::new()],
    );
    let rescanned = scan_document(&out);
    let section = rescanned
        .sections
        .iter()
        .find(|s| kind.matches(s.kind))
        .expect("just inserted this section")
        .clone();
    (out, section)
}

/// Index just past the enclosing heading's content (before the next same/shallower
/// heading, or EOF). Never panics on an empty document.
pub fn end_of_enclosing_section(lines: &[String], cursor_line: usize, level: u8) -> usize {
    if lines.is_empty() {
        return 0;
    }
    let clamped = cursor_line.min(lines.len() - 1);

    let mut start = None;
    for i in (0..=clamped).rev() {
        if let Line::Heading { level: lvl, .. } = classify_line(&lines[i])
            && lvl == level
        {
            start = Some(i);
            break;
        }
    }
    let start = start.unwrap_or(cursor_line);

    for (i, l) in lines.iter().enumerate().skip(start + 1) {
        if let Line::Heading { level: lvl, .. } = classify_line(l)
            && lvl <= level
        {
            return i;
        }
    }
    lines.len()
}

/// What `nowHHMM` was for `:start`/`:end` when a command ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCtx {
    pub now_hhmm: String,
}

/// The outcome of running a validated `:` command.
pub struct CommandResult {
    pub state: EditorState,
    pub effect: Option<AppEffect>,
}

fn add_block(state: &EditorState, kind: StandardSection, name: &str) -> EditorState {
    let ns = push_undo(state);
    let (lines, section) = ensure_section(&ns.lines, kind);
    let (lines, heading_index) = append_block(&lines, &section, &format!("### {name}"));
    let mut ns = ns;
    ns.lines = lines;
    ns.cursor = Cursor {
        line: heading_index,
        col: 0,
    };
    ns.message = String::new();
    clamp_cursor(&ns)
}

fn add_todo(state: &EditorState, text: &str) -> EditorState {
    let ns = push_undo(state);
    let model = scan_document(&ns.lines);
    let suffix = match resolve_context(&model, ns.cursor.line) {
        Context::Meeting { block, .. } => format!(" _({})_", block.name),
        _ => String::new(),
    };
    let (lines, section) = ensure_section(&ns.lines, StandardSection::Todo);
    let (lines, _) = append_line_to_section(&lines, &section, &format!("- [ ] {text}{suffix}"));
    let mut ns = ns;
    ns.lines = lines;
    ns.message = String::new(); // cursor stays
    ns
}

fn add_subsection(state: &EditorState, name: &str) -> EditorState {
    let Some(level) = nearest_heading_level(&state.lines, state.cursor.line) else {
        let mut ns = state.clone();
        ns.message = "No enclosing heading".to_string();
        return ns;
    };
    if level >= 6 {
        let mut ns = state.clone();
        ns.message = "Max heading depth".to_string();
        return ns;
    }
    let ns = push_undo(state);
    let heading = format!("{} {name}", "#".repeat((level + 1) as usize));
    let insert_at = end_of_enclosing_section(&ns.lines, ns.cursor.line, level);
    let mut ns = ns;
    ns.lines.insert(insert_at, String::new());
    ns.lines.insert(insert_at, heading);
    ns.cursor = Cursor {
        line: insert_at,
        col: 0,
    };
    ns.message = String::new();
    clamp_cursor(&ns)
}

/// Which kind of block `:scheduled`/`:purpose`/`:start`/`:end` (meeting) or `:topic`
/// (note) requires the cursor to be inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequiredContext {
    Meeting,
    Note,
}

fn set_meta(state: &EditorState, required: RequiredContext, key: &str, value: &str) -> EditorState {
    let model = scan_document(&state.lines);
    let block = match (required, resolve_context(&model, state.cursor.line)) {
        (RequiredContext::Meeting, Context::Meeting { block, .. }) => block,
        (RequiredContext::Note, Context::Note { block, .. }) => block,
        _ => {
            let mut ns = state.clone();
            ns.message = match required {
                RequiredContext::Meeting => "Not in a meeting".to_string(),
                RequiredContext::Note => "Not in a note".to_string(),
            };
            return ns;
        }
    };
    let ns = push_undo(state);
    let (lines, _) = upsert_meta(&ns.lines, &block, key, value);
    let mut ns = ns;
    ns.lines = lines;
    ns.message = String::new();
    ns
}

fn people(base: &EditorState, arg: &str) -> EditorState {
    let model = scan_document(&base.lines);
    let block = match resolve_context(&model, base.cursor.line) {
        Context::Meeting { block, .. } | Context::Note { block, .. } => block,
        _ => {
            let mut ns = base.clone();
            ns.message = "Not in a meeting or note".to_string();
            return ns;
        }
    };
    let ns = push_undo(base);
    let (lines, _) = append_meta(&ns.lines, &block, "people", arg);
    let mut ns = ns;
    ns.lines = lines;
    ns.message = String::new();
    ns
}

/// Run the command currently typed into `state.command` (colon excluded). Always clears
/// `command` back to `None` — on success *and* on a validation error, matching the web's
/// "close the palette either way, leave the error in `message`" behavior. Port of
/// `web/src/lib/editor/commands.ts` `runCommand`.
pub fn run_command(state: &EditorState, ctx: &CommandCtx) -> CommandResult {
    let mut base = state.clone();
    base.command = None;
    let typed = state.command.as_deref().unwrap_or("");

    let (command, arg) = match validate_command(typed) {
        ValidationResult::Err { error } => {
            base.message = error;
            return CommandResult {
                state: base,
                effect: None,
            };
        }
        ValidationResult::Ok { command, arg } => (command, arg),
    };

    match command {
        CommandName::Goto => {
            base.message = String::new();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Goto(arg)),
            }
        }
        CommandName::Today => {
            base.message = String::new();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Today),
            }
        }
        CommandName::Tab => {
            base.message = String::new();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Tab(arg)),
            }
        }
        CommandName::Close => {
            base.message = String::new();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Close),
            }
        }
        CommandName::W => {
            base.message = "Written".to_string();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Save),
            }
        }
        CommandName::Theme => {
            base.message = String::new();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Theme(arg)),
            }
        }
        CommandName::Meeting => CommandResult {
            state: add_block(&base, StandardSection::Meetings, &arg),
            effect: None,
        },
        CommandName::Note => CommandResult {
            state: add_block(&base, StandardSection::Notes, &arg),
            effect: None,
        },
        CommandName::Todo => CommandResult {
            state: add_todo(&base, &arg),
            effect: None,
        },
        CommandName::Section => CommandResult {
            state: add_subsection(&base, &arg),
            effect: None,
        },
        CommandName::Scheduled => CommandResult {
            state: set_meta(&base, RequiredContext::Meeting, "scheduled", &arg),
            effect: None,
        },
        CommandName::Purpose => CommandResult {
            state: set_meta(&base, RequiredContext::Meeting, "purpose", &arg),
            effect: None,
        },
        CommandName::Start => CommandResult {
            state: set_meta(&base, RequiredContext::Meeting, "started", &ctx.now_hhmm),
            effect: None,
        },
        CommandName::End => CommandResult {
            state: set_meta(&base, RequiredContext::Meeting, "ended", &ctx.now_hhmm),
            effect: None,
        },
        CommandName::Topic => CommandResult {
            state: set_meta(&base, RequiredContext::Note, "topic", &arg),
            effect: None,
        },
        CommandName::People => CommandResult {
            state: people(&base, &arg),
            effect: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::state::create_editor_state;

    const TEMPLATE: [&str; 8] = [
        "# 2026-06-23-TUE",
        "",
        "## To Do",
        "",
        "## Meetings",
        "",
        "## Notes",
        "",
    ];

    fn lines(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn append_block_adds_an_h3_at_the_end_of_a_section() {
        let tpl = lines(&TEMPLATE);
        let meetings = scan_document(&tpl)
            .sections
            .into_iter()
            .find(|s| s.kind == SectionKind::Meetings)
            .unwrap();
        let (out, heading_index) = append_block(&tpl, &meetings, "### Sync");
        assert_eq!(out[heading_index], "### Sync");
    }

    #[test]
    fn append_line_to_section_adds_after_the_last_non_blank_line() {
        let tpl = lines(&TEMPLATE);
        let todo = scan_document(&tpl)
            .sections
            .into_iter()
            .find(|s| s.kind == SectionKind::Todo)
            .unwrap();
        let (out, index) = append_line_to_section(&tpl, &todo, "- [ ] Buy milk");
        assert_eq!(out[index], "- [ ] Buy milk");
        let meetings_at = out.iter().position(|l| l == "## Meetings").unwrap();
        assert!(meetings_at > index);
    }

    #[test]
    fn upsert_meta_inserts_then_updates_in_place() {
        let mut ls = lines(&["## Meetings", "### Sync", ""]);
        let mut block = scan_document(&ls).sections[0].blocks[0].clone();
        let (out, _) = upsert_meta(&ls, &block, "scheduled", "14:30");
        ls = out;
        assert!(ls.contains(&"meta:scheduled 14:30".to_string()));
        block = scan_document(&ls).sections[0].blocks[0].clone();
        let (out, _) = upsert_meta(&ls, &block, "scheduled", "15:00");
        ls = out;
        assert_eq!(
            ls.iter()
                .filter(|l| l.starts_with("meta:scheduled"))
                .count(),
            1
        );
        assert!(ls.contains(&"meta:scheduled 15:00".to_string()));
    }

    #[test]
    fn ensure_section_recreates_a_missing_section_in_canonical_order() {
        let (out, section) = ensure_section(
            &lines(&["# T", "", "## Notes", ""]),
            StandardSection::Meetings,
        );
        let kinds: Vec<SectionKind> = scan_document(&out)
            .sections
            .iter()
            .map(|s| s.kind)
            .collect();
        let meetings_idx = kinds
            .iter()
            .position(|k| *k == SectionKind::Meetings)
            .unwrap();
        let notes_idx = kinds.iter().position(|k| *k == SectionKind::Notes).unwrap();
        assert!(meetings_idx < notes_idx);
        assert_eq!(section.kind, SectionKind::Meetings);
    }

    #[test]
    fn end_of_enclosing_section_finds_the_next_same_or_shallower_heading() {
        assert_eq!(
            end_of_enclosing_section(&lines(&["### A", "body", "### B"]), 1, 3),
            2
        );
    }

    #[test]
    fn append_meta_inserts_meta_people_when_no_prior_value_exists() {
        let ls = lines(&["## Meetings", "### Sync", ""]);
        let block = scan_document(&ls).sections[0].blocks[0].clone();
        let (out, _) = append_meta(&ls, &block, "people", "Alice");
        assert!(out.contains(&"meta:people Alice".to_string()));
    }

    #[test]
    fn append_meta_appends_comma_separated_to_an_existing_value() {
        let ls = lines(&["## Meetings", "### Sync", "meta:people Alice", ""]);
        let block = scan_document(&ls).sections[0].blocks[0].clone();
        let (out, _) = append_meta(&ls, &block, "people", "Bob");
        assert!(out.contains(&"meta:people Alice, Bob".to_string()));
        assert_eq!(
            out.iter().filter(|l| l.starts_with("meta:people")).count(),
            1
        );
    }

    #[test]
    fn append_meta_trims_whitespace_from_the_new_value_before_appending() {
        let ls = lines(&["## Meetings", "### Sync", "meta:people Alice", ""]);
        let block = scan_document(&ls).sections[0].blocks[0].clone();
        let (out, _) = append_meta(&ls, &block, "people", "  Bob  ");
        assert!(out.contains(&"meta:people Alice, Bob".to_string()));
    }

    fn with_cmd(raw: &[&str], cmd: &str, line: usize) -> EditorState {
        let mut s = create_editor_state(lines(raw), Vec::new());
        s.command = Some(cmd.to_string());
        s.cursor = Cursor { line, col: 0 };
        s
    }

    fn ctx() -> CommandCtx {
        CommandCtx {
            now_hhmm: "09:30".to_string(),
        }
    }

    #[test]
    fn reports_unknown_commands_and_clears_the_command_line() {
        let r = run_command(&with_cmd(&TEMPLATE, "meetng x", 0), &ctx());
        assert_eq!(r.state.command, None);
        assert!(r.state.message.contains("Unknown command"));
    }

    #[test]
    fn meeting_adds_a_heading_and_moves_the_cursor_to_it() {
        let r = run_command(&with_cmd(&TEMPLATE, "meeting Daily Standup", 0), &ctx());
        assert_eq!(r.state.lines[r.state.cursor.line], "### Daily Standup");
    }

    #[test]
    fn todo_appends_to_to_do_and_keeps_the_cursor() {
        let r = run_command(&with_cmd(&TEMPLATE, "todo Buy milk", 0), &ctx());
        assert!(r.state.lines.contains(&"- [ ] Buy milk".to_string()));
        assert_eq!(r.state.cursor.line, 0);
    }

    #[test]
    fn todo_inside_a_meeting_tags_the_meeting_name() {
        let raw = [
            "# T",
            "",
            "## To Do",
            "",
            "## Meetings",
            "### Sync",
            "",
            "## Notes",
            "",
        ];
        let r = run_command(&with_cmd(&raw, "todo Prep", 5), &ctx());
        assert!(r.state.lines.iter().any(|l| l == "- [ ] Prep _(Sync)_"));
    }

    #[test]
    fn scheduled_errors_when_not_in_a_meeting() {
        let r = run_command(&with_cmd(&TEMPLATE, "scheduled 14:30", 2), &ctx());
        assert_eq!(r.state.message, "Not in a meeting");
    }

    #[test]
    fn start_records_the_current_time_on_the_enclosing_meeting() {
        let raw = ["# T", "", "## Meetings", "### Sync", "", "## Notes", ""];
        let r = run_command(&with_cmd(&raw, "start", 3), &ctx());
        assert!(r.state.lines.contains(&"meta:started 09:30".to_string()));
    }

    #[test]
    fn section_nests_one_level_deeper_than_the_enclosing_heading() {
        let raw = ["## Meetings", "### Sync", "body", "## Notes", ""];
        let r = run_command(&with_cmd(&raw, "section Risks", 2), &ctx());
        assert!(r.state.lines.iter().any(|l| l == "#### Risks"));
    }

    #[test]
    fn goto_emits_an_effect_without_mutating_the_buffer() {
        let r = run_command(&with_cmd(&TEMPLATE, "goto 2026-06-01", 0), &ctx());
        assert_eq!(r.effect, Some(AppEffect::Goto("2026-06-01".to_string())));
        assert_eq!(r.state.lines, lines(&TEMPLATE));
    }

    #[test]
    fn people_sets_meta_people_in_a_meeting_block() {
        let raw = ["# T", "", "## Meetings", "### Sync", "", "## Notes", ""];
        let r = run_command(&with_cmd(&raw, "people Alice", 4), &ctx());
        assert!(r.state.lines.contains(&"meta:people Alice".to_string()));
        assert_eq!(r.state.message, "");
    }

    #[test]
    fn people_appends_to_existing_meta_people_in_a_meeting_block() {
        let raw = [
            "# T",
            "",
            "## Meetings",
            "### Sync",
            "meta:people Alice",
            "",
            "## Notes",
            "",
        ];
        let r = run_command(&with_cmd(&raw, "people Bob", 5), &ctx());
        assert!(
            r.state
                .lines
                .contains(&"meta:people Alice, Bob".to_string())
        );
    }

    #[test]
    fn people_sets_meta_people_in_a_note_block() {
        let raw = [
            "# T",
            "",
            "## Meetings",
            "",
            "## Notes",
            "### Retro",
            "",
            "",
        ];
        let r = run_command(&with_cmd(&raw, "people Alice", 6), &ctx());
        assert!(r.state.lines.contains(&"meta:people Alice".to_string()));
        assert_eq!(r.state.message, "");
    }

    #[test]
    fn people_errors_when_not_in_a_meeting_or_note_block() {
        // TEMPLATE cursor line 2 = "## To Do" section heading, not inside any block.
        let r = run_command(&with_cmd(&TEMPLATE, "people Alice", 2), &ctx());
        assert!(r.state.message.contains("meeting or note"));
    }

    #[test]
    fn p_shortcut_works_end_to_end_through_run_command() {
        let raw = ["# T", "", "## Meetings", "### Sync", "", "## Notes", ""];
        let r = run_command(&with_cmd(&raw, "p Alice", 4), &ctx());
        assert!(r.state.lines.contains(&"meta:people Alice".to_string()));
    }
}
```

- [ ] **Step 2: Declare the module** — in `crates/slugline-core/src/editor/mod.rs`, replace:

```rust
pub mod edits;
pub mod insert;
pub mod keymap;
pub mod motions;
pub mod state;

pub use keymap::{AppEffect, KeyInput, KeyResult, handle_key};
pub use state::{Cursor, EditorState, Mode, Pending, clamp_cursor, create_editor_state};
```

with:

```rust
pub mod commands;
pub mod edits;
pub mod insert;
pub mod keymap;
pub mod motions;
pub mod state;

pub use commands::{CommandCtx, CommandResult, run_command};
pub use keymap::{AppEffect, KeyInput, KeyResult, handle_key};
pub use state::{Cursor, EditorState, Mode, Pending, clamp_cursor, create_editor_state};
```

- [ ] **Step 3: Run the tests** — `cargo test -p slugline-core editor::commands::`
Expected: PASS (21 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/slugline-core/src/editor/commands.rs crates/slugline-core/src/editor/mod.rs
git commit -m "feat(core): port runCommand + section/block/meta helpers"
```

---

### Task 5: Wire command mode into `handle_key`

**Files:**
- Modify: `crates/slugline-core/src/editor/keymap.rs`

- [ ] **Step 1: Update the imports** — replace:

```rust
use super::state::{EditorState, Mode, Pending};
use super::{edits, insert, motions, state};
```

with:

```rust
use super::commands::{CommandCtx, run_command};
use super::state::{EditorState, Mode, Pending};
use super::{edits, insert, motions, state};
```

- [ ] **Step 2: Give `handle_key` a `ctx` parameter and route command mode first** — replace:

```rust
pub fn handle_key(state: &EditorState, key: &KeyInput) -> KeyResult {
    // Phase 1: command mode (`:`) is never entered; it is added in Phase 5.
    if state.mode == Mode::Insert {
        return state_only(handle_insert(state, key));
    }
    handle_normal(state, key)
}
```

with:

```rust
pub fn handle_key(state: &EditorState, key: &KeyInput, ctx: &CommandCtx) -> KeyResult {
    if state.command.is_some() {
        return handle_command_mode(state, key, ctx);
    }
    if state.mode == Mode::Insert {
        return state_only(handle_insert(state, key));
    }
    handle_normal(state, key)
}

fn handle_command_mode(state: &EditorState, key: &KeyInput, ctx: &CommandCtx) -> KeyResult {
    match key.key.as_str() {
        "Escape" => {
            let mut ns = state.clone();
            ns.command = None;
            ns.message = String::new();
            state_only(ns)
        }
        "Enter" => {
            let result = run_command(state, ctx);
            KeyResult {
                state: result.state,
                effect: result.effect,
            }
        }
        "Backspace" => {
            let mut ns = state.clone();
            let mut typed = ns.command.unwrap_or_default();
            typed.pop();
            ns.command = Some(typed);
            state_only(ns)
        }
        _ => {
            if key.key.chars().count() == 1 && !key.ctrl && !key.meta {
                let mut ns = state.clone();
                let mut typed = ns.command.unwrap_or_default();
                typed.push_str(&key.key);
                ns.command = Some(typed);
                state_only(ns)
            } else {
                state_only(state.clone())
            }
        }
    }
}
```

- [ ] **Step 3: Make `:` open command mode** — in `handle_normal`, replace:

```rust
        "o" => insert::open_below(s),
        "O" => insert::open_above(s),
        "Enter" => motions::move_down(s),
        // ":" command mode is deferred to Phase 5.
        _ => {
```

with:

```rust
        "o" => insert::open_below(s),
        "O" => insert::open_above(s),
        "Enter" => motions::move_down(s),
        ":" => {
            let mut ns = s.clone();
            ns.command = Some(String::new());
            ns.message = String::new();
            ns
        }
        _ => {
```

- [ ] **Step 4: Update every existing test call site to pass a `ctx()`** — in `mod tests`, add a
helper right after the existing `key()` helper:

```rust
    fn ctx() -> CommandCtx {
        CommandCtx {
            now_hhmm: "09:30".to_string(),
        }
    }
```

Then update every `handle_key(...)` call in the file to pass `&ctx()` as the third argument. There
are 13 call sites across `i_enters_insert_then_typing_inserts`, `dd_requires_two_keystrokes`,
`ctrl_r_redoes` (two calls plus the explicit `KeyInput` one), `j_moves_down_in_normal_mode`,
`escape_exits_insert_mode` (three calls), `gg_jumps_to_first_line` (two calls),
`gt_emits_tab_next_effect` (two calls), `shift_gt_emits_tab_prev_effect` (two calls),
`bracket_keys_emit_day_navigation` (two calls), and `ctrl_t_emits_today_effect` (one explicit
`KeyInput` call) — for example:

```rust
    #[test]
    fn i_enters_insert_then_typing_inserts() {
        let s = create_editor_state(vec!["ac".into()], vec![]);
        let s = handle_key(&s, &key("i"), &ctx()).state;
        assert_eq!(s.mode, Mode::Insert);
        let s = handle_key(&s, &key("x"), &ctx()).state;
        assert_eq!(s.lines, vec!["xac".to_string()]);
    }
```

Apply the same `, &ctx()` insertion (as the third positional argument to `handle_key`) at every
other call site named above — the compiler will point at each one with a "this function takes 3
arguments but 2 arguments were supplied" error if any are missed.

- [ ] **Step 5: Replace the "deferred" placeholder comment with real command-mode tests** —
replace:

```rust
    // TODO(phase 5): command mode (`:`) tests — deferred to Phase 5

    // Review flag: keymap.ts:137-138 maps Escape in NORMAL mode to enterInsert(...).
    // This looks like a latent bug and is deliberately NOT replicated here.
}
```

with:

```rust
    // Ported from web/src/lib/editor/keymap.test.ts — command mode.

    #[test]
    fn colon_opens_the_command_line_and_enter_runs_it() {
        let s = create_editor_state(
            vec!["# T".into(), "".into(), "## To Do".into(), "".into()],
            vec![],
        );
        let s = handle_key(&s, &key(":"), &ctx()).state;
        assert_eq!(s.command, Some(String::new()));
        let mut s = s;
        for ch in ["t", "o", "d", "o", " ", "m"] {
            s = handle_key(&s, &key(ch), &ctx()).state;
        }
        let r = handle_key(&s, &key("Enter"), &ctx());
        assert!(r.state.lines.contains(&"- [ ] m".to_string()));
        assert_eq!(r.state.command, None);
    }

    #[test]
    fn escape_in_command_mode_clears_the_command_without_running_it() {
        let mut s = create_editor_state(vec!["a".into()], vec![]);
        s = handle_key(&s, &key(":"), &ctx()).state;
        s = handle_key(&s, &key("x"), &ctx()).state;
        assert_eq!(s.command, Some("x".to_string()));
        let s = handle_key(&s, &key("Escape"), &ctx()).state;
        assert_eq!(s.command, None);
        assert_eq!(s.lines, vec!["a".to_string()]); // untouched
    }

    #[test]
    fn backspace_in_command_mode_shortens_the_typed_text() {
        let mut s = create_editor_state(vec!["a".into()], vec![]);
        s = handle_key(&s, &key(":"), &ctx()).state;
        s = handle_key(&s, &key("t"), &ctx()).state;
        s = handle_key(&s, &key("o"), &ctx()).state;
        assert_eq!(s.command, Some("to".to_string()));
        let s = handle_key(&s, &key("Backspace"), &ctx()).state;
        assert_eq!(s.command, Some("t".to_string()));
    }

    // Review flag: keymap.ts:137-138 maps Escape in NORMAL mode to enterInsert(...).
    // This looks like a latent bug and is deliberately NOT replicated here.
}
```

- [ ] **Step 6: Run the whole core test suite** — `cargo test -p slugline-core`
Expected: PASS — 158 tests (113 baseline + 6 `doc::context::` + 14 `doc::command::` + 1
`dates::tests::now_hhmm_is_zero_padded_hh_mm` + 21 `editor::commands::` + 3 new
`editor::keymap::tests::` command-mode cases).

- [ ] **Step 7: Format + clippy the core crate** —
`cargo fmt --all -- --check && cargo clippy -p slugline-core --all-targets -- -D warnings`
Expected: clean.

- [ ] **Step 8: Commit**

```bash
git add crates/slugline-core/src/editor/keymap.rs
git commit -m "feat(core): wire command mode into handle_key"
```

---

### Task 6: Thread `CommandCtx` into `app.rs` and add the palette messages

**Files:**
- Modify: `crates/slugline/src/app.rs`

This task updates every call site and message plumbing except the palette overlay's own file
(`ui/command_palette.rs`), which doesn't exist until Task 7. `cargo build -p slugline` will fail
after this task purely because `crate::ui::command_palette` is unresolved — that's expected and
resolved by Task 7 (mirrors Phase 4's Task 5/6 split for the same reason).

- [ ] **Step 1: Update imports** — replace:

```rust
use iced::widget::{column, pane_grid, row};
use iced::{Element, Length, Subscription, Task, keyboard, time, window};

use slugline_core::dates::{YearMonth, add_days, today_iso, year_month};
use slugline_core::editor::{
    AppEffect, Cursor, EditorState, KeyInput, clamp_cursor, create_editor_state, handle_key,
};
```

with:

```rust
use iced::widget::{column, pane_grid, row, stack};
use iced::{Element, Length, Subscription, Task, keyboard, time, window};

use slugline_core::dates::{YearMonth, add_days, now_hhmm, today_iso, year_month};
use slugline_core::editor::{
    AppEffect, CommandCtx, Cursor, EditorState, KeyInput, clamp_cursor, create_editor_state,
    handle_key,
};
```

- [ ] **Step 2: Import the new UI module** — replace:

```rust
use crate::keys::key_string;
use crate::ui::{editor_pane, sidebar, tab_strip};
```

with:

```rust
use crate::keys::key_string;
use crate::ui::{command_palette, editor_pane, sidebar, tab_strip};
```

- [ ] **Step 3: Add the two new `Message` variants** — replace:

```rust
    /// An Agenda or To Do row was clicked: jump to `line` in `date`'s note, navigating
    /// there first if it isn't already active.
    OpenDateAndLine(String, usize),
}
```

with:

```rust
    /// An Agenda or To Do row was clicked: jump to `line` in `date`'s note, navigating
    /// there first if it isn't already active.
    OpenDateAndLine(String, usize),
    /// `Cmd/Ctrl-K` was pressed: seed command mode from anywhere, mirroring what typing
    /// `:` does in NORMAL mode (design Section 4). Bypasses `handle_key`/the vim keymap
    /// entirely — this is a native-refined shortcut layered on top of the ported engine,
    /// not part of it.
    OpenPalette,
    /// A command palette suggestion was clicked: seed the command buffer with that
    /// command's name (plus a trailing space, ready for its argument) rather than
    /// running it — matches typing the name and leaves `Enter` as the one path that
    /// invokes `run_command`.
    PaletteSuggestionClicked(String),
}
```

- [ ] **Step 4: Build a `CommandCtx` per keystroke** — replace:

```rust
            Message::Key(input) => {
                let before = self.editor.lines.clone();
                let result = handle_key(&self.editor, &input);
                self.editor = result.state;
```

with:

```rust
            Message::Key(input) => {
                let before = self.editor.lines.clone();
                let ctx = CommandCtx {
                    now_hhmm: now_hhmm(),
                };
                let result = handle_key(&self.editor, &input, &ctx);
                self.editor = result.state;
```

- [ ] **Step 5: Handle the two new messages** — replace:

```rust
                }
                window::close(id)
            }
        }
    }
```

with:

```rust
                }
                window::close(id)
            }
            Message::OpenPalette => {
                self.editor.command = Some(String::new());
                self.editor.message = String::new();
                Task::none()
            }
            Message::PaletteSuggestionClicked(name) => {
                self.editor.command = Some(format!("{name} "));
                Task::none()
            }
        }
    }
```

- [ ] **Step 6: Stack the palette overlay on top of the base view** — replace:

```rust
    pub fn view(&self) -> Element<'_, Message> {
        if self.sidebar_collapsed {
            row![sidebar::collapsed_rail(), self.main_pane()]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            pane_grid(&self.panes, |_pane, kind, _is_maximized| {
                pane_grid::Content::new(match kind {
                    PaneKind::Sidebar => sidebar::view(
                        self.calendar,
                        &today_iso(),
                        &self.active_date(),
                        &self.notes_with_files,
                        &self.editor.lines,
                        &self.todo_groups,
                    ),
                    PaneKind::Main => self.main_pane(),
                })
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .on_resize(6, Message::PaneResized)
            .into()
        }
    }
```

with:

```rust
    pub fn view(&self) -> Element<'_, Message> {
        let base: Element<'_, Message> = if self.sidebar_collapsed {
            row![sidebar::collapsed_rail(), self.main_pane()]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            pane_grid(&self.panes, |_pane, kind, _is_maximized| {
                pane_grid::Content::new(match kind {
                    PaneKind::Sidebar => sidebar::view(
                        self.calendar,
                        &today_iso(),
                        &self.active_date(),
                        &self.notes_with_files,
                        &self.editor.lines,
                        &self.todo_groups,
                    ),
                    PaneKind::Main => self.main_pane(),
                })
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .on_resize(6, Message::PaneResized)
            .into()
        };

        // The command palette overlay (design Section 4): floats on top of everything
        // else whenever command mode is active (`:` or Cmd/Ctrl-K), and disappears the
        // instant `editor.command` goes back to `None` (Escape, or Enter via `run_command`).
        match &self.editor.command {
            Some(typed) => stack![base, command_palette::view(typed)].into(),
            None => base,
        }
    }
```

- [ ] **Step 7: Intercept `Cmd/Ctrl-K` in the keyboard subscription** — replace:

```rust
            keyboard::on_key_press(|key, mods| {
                key_string(&key).map(|k| {
                    Message::Key(KeyInput {
                        key: k,
                        ctrl: mods.control(),
                        meta: mods.logo(),
                        shift: mods.shift(),
                    })
                })
            }),
```

with:

```rust
            keyboard::on_key_press(|key, mods| {
                let k = key_string(&key)?;
                if (mods.control() || mods.logo()) && (k == "k" || k == "K") {
                    return Some(Message::OpenPalette);
                }
                Some(Message::Key(KeyInput {
                    key: k,
                    ctrl: mods.control(),
                    meta: mods.logo(),
                    shift: mods.shift(),
                }))
            }),
```

- [ ] **Step 8: Add tests for the two new messages** — insert at the end of the `#[cfg(test)] mod
tests` block, right before the final closing `}`:

```rust
    #[test]
    fn open_palette_seeds_an_empty_command_from_any_state() {
        let (_dir, mut app) = temp_app("2026-06-23");
        assert_eq!(app.editor.command, None);
        let _ = app.update(Message::OpenPalette);
        assert_eq!(app.editor.command, Some(String::new()));
    }

    #[test]
    fn palette_suggestion_clicked_seeds_the_command_with_a_trailing_space() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let _ = app.update(Message::PaletteSuggestionClicked("meeting".to_string()));
        assert_eq!(app.editor.command, Some("meeting ".to_string()));
    }
```

- [ ] **Step 9: Attempt a build** — `cargo build -p slugline`
Expected: fails only because `crate::ui::command_palette` doesn't exist yet — Task 7 adds it. This
mirrors Phase 4's Task 5/6 split.

(Do not commit yet — Task 7 finishes the crate so it builds and tests pass, then both tasks'
changes commit together, since the crate is only buildable as a whole.)

---

### Task 7: Build `ui/command_palette.rs`

**Files:**
- Create: `crates/slugline/src/ui/command_palette.rs`
- Modify: `crates/slugline/src/ui/mod.rs`

- [ ] **Step 1: Create the palette widget and its fuzzy matcher** — create
`crates/slugline/src/ui/command_palette.rs`:

```rust
use iced::alignment::{Horizontal, Vertical};
use iced::font::Weight;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Font, Length};

use slugline_core::doc::{ArgKind, COMMANDS, CommandSpec};

use crate::app::Message;
use crate::ui::palette;

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
/// command when nothing has been typed yet). No TS counterpart — the command palette is
/// new native-refined UI (design Section 4); this is a fresh implementation with fresh
/// tests, same rationale as Phase 3/4's `shift_month`/7-day-aggregation tests.
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
/// base view whenever `editor.command.is_some()` (design Section 4). Clicking a
/// suggestion seeds the command buffer with that command's name
/// (`Message::PaletteSuggestionClicked`); `Enter` always runs whatever is currently typed
/// through the same `run_command` path as typing `:cmd` directly — this overlay only ever
/// edits `editor.command`, never runs a command itself.
pub fn view<'a>(typed: &str) -> Element<'a, Message> {
    let suggestions = filter_commands(typed);

    let input = container(
        text(format!(":{typed}"))
            .font(MONO)
            .size(15)
            .color(palette::FG),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(palette::EDIT_BAR_BG.into()),
        border: iced::Border {
            color: palette::RULE,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..container::Style::default()
    });

    let mut list = column![].spacing(1);
    if suggestions.is_empty() {
        list = list.push(
            container(text("No matching commands").size(12).color(palette::MUTED)).padding([4, 8]),
        );
    } else {
        for spec in suggestions {
            list = list.push(suggestion_row(spec));
        }
    }

    let box_ = container(column![input, list].spacing(6).width(Length::Fixed(440.0)))
        .padding(12)
        .style(|_theme| container::Style {
            background: Some(palette::BG.into()),
            border: iced::Border {
                color: palette::STATUS_BAR,
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

fn suggestion_row<'a>(spec: &'static CommandSpec) -> Element<'a, Message> {
    let name = spec.name.canonical();
    let label = row![
        text(format!(":{name}{}", usage_hint(spec)))
            .font(Font {
                weight: Weight::Bold,
                ..MONO
            })
            .size(12)
            .color(palette::ACCENT),
        text(description(spec)).size(12).color(palette::MUTED),
    ]
    .spacing(10);

    button(label)
        .padding([3, 8])
        .width(Length::Fill)
        .on_press(Message::PaletteSuggestionClicked(name.to_string()))
        .style(|_theme, status| {
            let background = if status == button::Status::Hovered {
                Some(palette::EDIT_BAR_BG.into())
            } else {
                None
            };
            button::Style {
                background,
                text_color: palette::FG,
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
                "meeting", "note", "section", "todo", "start", "end", "scheduled", "purpose"
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

- [ ] **Step 2: Declare the module** — in `crates/slugline/src/ui/mod.rs`, replace:

```rust
pub mod agenda;
pub mod calendar;
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
pub mod palette;
pub mod sidebar;
pub mod tab_strip;
pub mod todo_list;
```

- [ ] **Step 3: Run the whole workspace test suite** — `cargo test --workspace`
Expected: PASS — 158 `slugline-core` tests + 34 `slugline` tests (28 baseline + 6
`ui::command_palette::tests::` + 2 `app::tests::` from Task 6's Step 8).

- [ ] **Step 4: Format + clippy the whole workspace** —
`cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean.

- [ ] **Step 5: Smoke-run the app** — in one terminal:

```bash
cargo run -p slugline -- --notes-dir /tmp/slugline-phase5-smoke
```

Confirm: the window opens, a note materializes at
`/tmp/slugline-phase5-smoke/<today>.md`, and there is no panic in the terminal. Then, in the
running window: press `:`, confirm the palette overlay appears top-center showing `:` and a list of
commands; type `today`, confirm the list narrows to just `:today`; press `Escape`, confirm the
overlay disappears and the buffer is unchanged; press `Cmd-K` (macOS) or `Ctrl-K`, confirm the
palette reopens empty; click the `:goto` suggestion, confirm the command buffer now reads
`goto ` with the cursor implicitly ready for a date argument (the overlay's input line updates to
show it). Quit the app (window close) once satisfied.

- [ ] **Step 6: Commit**

```bash
git add crates/slugline/src/app.rs crates/slugline/src/ui/command_palette.rs crates/slugline/src/ui/mod.rs
git commit -m "feat(app): command palette overlay + Cmd/Ctrl-K, CommandCtx wiring"
```

---

## "Done" definition for Phase 5

- `cargo test --workspace` green: 158 `slugline-core` tests + 34 `slugline` tests.
- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` both
  clean.
- Manual smoke (Task 7 Step 5): pressing `:` or `Cmd/Ctrl-K` opens the palette; typing narrows the
  fuzzy-filtered suggestion list; clicking a suggestion autocompletes the command name; `Enter` runs
  the typed command through `run_command` exactly as it would from the old bottom command-line
  (verified via the ported `:todo`/`:meeting`/`:goto` tests); `Escape` closes the palette without
  mutating the buffer; all 16 commands (`meeting/note/section/todo/start/end/scheduled/purpose/
  topic/people` + `goto/today/tab/close/w/theme`) validate and run correctly, including the `:p`
  alias for `:people`.
- Explicitly **not** done yet (by design, per Scope): `:theme` has no visible effect until Phase 6;
  command validation errors (`editor.message`) have no visible surface until Phase 6's status line;
  the palette has no backdrop-click-to-dismiss or arrow-key list navigation.

