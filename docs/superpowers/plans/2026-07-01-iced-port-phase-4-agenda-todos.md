# Phase 4 — Agenda & 7-Day To Do Aggregation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **This is a port.** Behavioral truth is `web/src/lib/doc/scan.ts` (+ `scan.test.ts`),
> `web/src/lib/agenda.ts` (+ `agenda.test.ts`), and `web/src/lib/todos.ts` (+ `todos.test.ts`), plus
> the 7-day aggregation logic inline in `web/src/lib/appState.svelte.ts`'s `refreshTodos()` (no
> existing TS test — this plan writes a fresh one, same rationale as Phase 3's `shift_month`). UI
> shape comes from `web/src/lib/components/Agenda.svelte`, `TodoList.svelte`, and `Sidebar.svelte`.
>
> **Iced API caution:** method/type names target iced `0.13.x` (`rich_text`, `span`,
> `text::Span::strikethrough`, `button::Style`). Every non-trivial API call in this plan was
> implemented in a disposable scratch worktree off the real `iced-port` tip, compiled, tested
> (`cargo test --workspace`: 113 + 28 tests green), formatted, linted (`cargo clippy --workspace
> --all-targets -- -D warnings`: clean), and smoke-run (`cargo run -p slugline`, confirmed the
> window opens with the Agenda/To Do sections rendered and no panics) before being copied into this
> document — but if a signature has drifted in your checkout, confirm it and adjust; the *intent*
> is the contract.

**Goal:** Add the Agenda (scheduled meetings for the open note) and To Do (7-day todo aggregation)
sections to the sidebar, both stacked below the calendar, with click-to-navigate: an Agenda row
jumps to its meeting heading in the current note; a To Do row jumps to its line, navigating there
first if it's a different date.

**Architecture:** One new `slugline-core::doc::scan` module (`Section`/`Block`/`MetaEntry`/
`DocModel`/`scan_document`) that `agenda` and `todos` (two new top-level `slugline-core` modules)
both build on — this is the one piece of shared parsing infrastructure the web's `agenda.ts`/
`todos.ts` already both depended on (`scanDocument`) but that hadn't been ported yet, since only
`classify`/`render_inline` were needed through Phase 3. Two new UI-only files (`ui/agenda.rs`,
`ui/todo_list.rs`) render the derived/aggregated data; `sidebar.rs` stacks them under the calendar
in a scrollable column. `app.rs` gains a `todo_groups: Vec<TodoGroup>` Model field refreshed via a
new `Task` (`TodosRefreshed`), a `pending_jump_line: Option<usize>` field to sequence
navigate-then-jump, and one new `Message` (`OpenDateAndLine`) that either jumps the cursor directly
(same date) or navigates first and queues the jump (different date). Agenda, unlike To Do, carries
no Model state at all — it's derived fresh from `editor.lines` on every `view()`, exactly like the
web's `$derived`, since it never needs to read any file but the one already open.

**Tech Stack:** Rust, existing `slugline-core` (`doc::classify`, `dates::add_days`), Iced `0.13.x`
`rich_text`/`span` for strikethrough-on-done styling (already used in `ui/editor_pane.rs`).

---

## Prerequisites

- **Phases 0, 1a, 1b, 1c, 2, 3 are complete and committed on `iced-port`, and `cargo test
  --workspace` is green** (100 `slugline-core` tests + 19 tests in `slugline`, as of `94549c2
  "feat(app): sidebar calendar in a resizable/collapsible pane_grid"`). Phase 4 builds directly on
  the Phase 3 `App` (`crates/slugline/src/app.rs`: `calendar`, `notes_with_files`, `pane_grid`,
  `sidebar_collapsed`, `Message::{OpenDate,NotesListed,PrevMonth,NextMonth,PaneResized,
  ToggleSidebar}`, `navigate()`), `core::dates::add_days`, `core::doc::classify_line`, and
  `core::editor::{Cursor, clamp_cursor}` (already ported in Phase 1b, reused as-is — `clamp_cursor`
  has been unused outside `editor` until now).

## Scope

**In this phase:**
- `core::doc::scan`: `MetaEntry`, `SectionKind { Todo, Meetings, Notes, Other }`, `Block`,
  `Section`, `DocModel`, `scan_document(lines) -> DocModel` — a direct port of
  `web/src/lib/doc/scan.ts`, with its `scan.test.ts` cases ported against the same `fixtures/*.md`
  files the TS tests already use (read at test time via `CARGO_MANIFEST_DIR`, not duplicated as
  literals — one fixture set, two test suites).
- `core::agenda`: `AgendaItem`, `derive_agenda(lines) -> Vec<AgendaItem>` — port of
  `web/src/lib/agenda.ts`, with `agenda.test.ts` ported against the same fixtures.
- `core::todos`: `TodoItem`, `TodoGroup`, `extract_todos(lines) -> Vec<TodoItem>`,
  `window_dates(active_date, days) -> Vec<String>` — port of `web/src/lib/todos.ts`, with
  `todos.test.ts` ported (the TS default parameter `days = 7` has no Rust equivalent; the one
  caller, `app.rs`, always passes `7` explicitly, and the ported test does too).
- An Agenda widget (`ui/agenda.rs`): "Agenda" header, empty state ("No scheduled meetings"), or a
  row per scheduled meeting (`HH:MM` + name, struck through with a ✓ once `ended` is set). Always
  derived from the currently open note's lines — no Model state, no disk I/O beyond what's already
  open. Clicking a row emits `OpenDateAndLine(active_date, heading_line_index)`.
- A To Do widget (`ui/todo_list.rs`): "To Do" header, empty state ("No to dos in the last 7 days"),
  or one date-labeled group per date in `App::todo_groups`, each a list of checkbox-glyph rows.
  Clicking a row emits `OpenDateAndLine(group.date, todo.line_index)`.
- `App::todo_groups: Vec<TodoGroup>`, refreshed by a new `refresh_todos_task()` (mirrors
  `list_notes_task()`'s shape) that reads `window_dates(active, 7)`, skipping any non-active date
  that doesn't already have a file (`core::store::read_or_create` is never called speculatively for
  a date nobody has opened — ports the web's "never materialize other days" comment verbatim), and
  keeps only dates with at least one todo. Wired in after every `NotesListed` (startup +
  post-navigation, mirroring `loadActive`'s `refreshNotesList(); refreshTodos()` sequence — the
  *existing*-files check needs a just-refreshed list) and after every same-tab `Saved` success
  (mirroring `flush()`'s `await this.refreshTodos()`).
- `Message::OpenDateAndLine(String, usize)`: same-date jumps the cursor immediately
  (`clamp_cursor`, no navigation); different-date sets `pending_jump_line` and navigates, and the
  `Navigated` success arm applies the queued jump once the new `editor` exists (its cursor would
  otherwise reset to `0,0`). A failed navigation clears the pending jump rather than applying it to
  the untouched buffer.
- `palette::STATUS_BAR`, a new token for the panel-separator border matching the web's dark-theme
  `--status-bar: #1f2535` (distinct from `RULE`'s `#2d3650`, which is a different separator color
  in the web's palette).
- Sidebar becomes independently scrollable (`scrollable` wraps calendar+agenda+todo column), since
  three stacked sections plus a 7-day todo aggregation can now exceed the pane's height.

**Deferred on purpose:**
- **Per-section chevron collapse** (design Section 4's `TogglePanel(Panel)`). Phase 3 deferred this
  "until Agenda/Todos land" — they have now, so a `Panel` enum would no longer be pure speculation,
  but the roadmap's Phase 4 entry only calls out "Agenda derivation + 7-day To Do aggregation,
  click-to-navigate," not collapsible sections, and Phase 6 ("Theming & polish") is where the
  design groups the rest of the sidebar's chrome work. Keeping it there avoids scope creep here.
- **`:meeting`/`:scheduled`/`:started`/`:ended` commands** that *write* the `meta:` lines Agenda
  reads — command mode is Phase 5. This phase only renders meetings that already have `scheduled`
  meta (e.g. hand-edited, or from the `fixtures/full-day.md`-style template); nothing in this phase
  needs a way to create one from the UI yet.
- **Ctrl/Cmd-click a todo/agenda row to open in a new tab** — same rationale as Phase 3's identical
  deferral for the calendar: modifier-tracking on a plain click is real added complexity for a
  secondary interaction the roadmap doesn't call out ("click-to-navigate", singular, in the active
  tab).
- **Status line / toast surface** for save/load errors — Phase 6.

---

## File Structure (files added/changed in Phase 4)

```
crates/slugline-core/
  src/
    lib.rs                          # + pub mod agenda; pub mod todos;
    agenda.rs                       # NEW: AgendaItem, derive_agenda() (tested)
    todos.rs                        # NEW: TodoItem, TodoGroup, extract_todos(), window_dates() (tested)
    doc/
      mod.rs                        # + pub mod scan; re-exports
      scan.rs                       # NEW: MetaEntry/SectionKind/Block/Section/DocModel, scan_document() (tested)

crates/slugline/
  src/
    app.rs                          # REWRITE: todo_groups/pending_jump_line, TodosRefreshed/OpenDateAndLine
    ui/mod.rs                       # + pub mod agenda; pub mod todo_list;
    ui/palette.rs                   # + STATUS_BAR color token
    ui/agenda.rs                    # NEW: Agenda section widget (no tests — pure UI)
    ui/todo_list.rs                 # NEW: To Do section widget (no tests — pure UI)
    ui/sidebar.rs                   # REWRITE: stacks agenda/todo_list under calendar, scrollable
```

---

### Task 1: Port `scan_document` into `core::doc::scan`

**Files:**
- Create: `crates/slugline-core/src/doc/scan.rs`
- Modify: `crates/slugline-core/src/doc/mod.rs`

- [ ] **Step 1: Write the failing tests** — create `crates/slugline-core/src/doc/scan.rs` with the
types, `scan_document`, and its ported tests (read `fixtures/*.md` at test time, the same files
`web/src/lib/doc/scan.test.ts` uses via `fixtureLines`):

```rust
use super::classify::{Line, classify_line};

/// A `meta:key value` line inside a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaEntry {
    pub key: String,
    pub value: String,
    pub line_index: usize,
}

/// The kind of a top-level (H2) section, inferred from its title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Todo,
    Meetings,
    Notes,
    Other,
}

/// An H3 block nested under a `Meetings`/`Notes` section (e.g. one meeting).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub name: String,
    pub level: u8,
    pub heading_line_index: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub meta: Vec<MetaEntry>,
    /// Index of the last meta line, or `heading_line_index` when the block has no meta.
    pub meta_end_line: usize,
}

/// A top-level (H2) section of the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub kind: SectionKind,
    pub title: String,
    pub level: u8,
    pub heading_line_index: usize,
    pub start_line: usize,
    pub end_line: usize,
    /// H3 blocks for `Meetings`/`Notes`; empty otherwise.
    pub blocks: Vec<Block>,
}

/// The whole document: an optional title (first H1) and its top-level sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocModel {
    pub title: Option<String>,
    pub title_line_index: Option<usize>,
    pub sections: Vec<Section>,
}

fn section_kind(title: &str) -> SectionKind {
    match title.trim().to_lowercase().as_str() {
        "to do" | "todo" => SectionKind::Todo,
        "meetings" => SectionKind::Meetings,
        "notes" => SectionKind::Notes,
        _ => SectionKind::Other,
    }
}

/// Scans forward from `from` to `to` (inclusive) and returns the index just before the
/// first heading whose level is `<= max_level`, or `to` if none is found. Used to find
/// where an H3 block or H2 section ends.
fn find_boundary_end(classified: &[Line], from: usize, to: usize, max_level: u8) -> usize {
    for (j, c) in classified.iter().enumerate().take(to + 1).skip(from) {
        if let Line::Heading { level, .. } = c
            && *level <= max_level
        {
            return j - 1;
        }
    }
    to
}

fn collect_blocks(classified: &[Line], from: usize, to: usize) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut i = from;
    while i <= to {
        if let Line::Heading { level: 3, text } = &classified[i] {
            let start = i;
            let end = find_boundary_end(classified, i + 1, to, 3);

            let mut meta = Vec::new();
            let mut meta_end_line = start;
            let mut k = start + 1;
            while k <= end {
                match &classified[k] {
                    Line::Meta { key, text: value } => {
                        meta.push(MetaEntry {
                            key: key.clone(),
                            value: value.clone(),
                            line_index: k,
                        });
                        meta_end_line = k;
                        k += 1;
                    }
                    _ => break,
                }
            }

            blocks.push(Block {
                name: text.clone(),
                level: 3,
                heading_line_index: start,
                start_line: start,
                end_line: end,
                meta,
                meta_end_line,
            });
            i = end + 1;
        } else {
            i += 1;
        }
    }
    blocks
}

/// Scan a document's raw lines into title + top-level sections (with H3 blocks for
/// `Meetings`/`Notes`). Never panics: malformed/heading-less documents just yield an
/// empty section list. Port of `web/src/lib/doc/scan.ts`.
pub fn scan_document(lines: &[String]) -> DocModel {
    let classified: Vec<Line> = lines.iter().map(|l| classify_line(l)).collect();

    let mut title = None;
    let mut title_line_index = None;
    for (i, c) in classified.iter().enumerate() {
        if let Line::Heading { level: 1, text } = c {
            title = Some(text.clone());
            title_line_index = Some(i);
            break;
        }
    }

    let mut sections = Vec::new();
    if !classified.is_empty() {
        let last = classified.len() - 1;
        let mut i = 0;
        while i <= last {
            if let Line::Heading { level: 2, text } = &classified[i] {
                let start = i;
                let end = find_boundary_end(&classified, i + 1, last, 2);
                let kind = section_kind(text);
                let blocks = match kind {
                    SectionKind::Meetings | SectionKind::Notes => {
                        collect_blocks(&classified, start + 1, end)
                    }
                    SectionKind::Todo | SectionKind::Other => Vec::new(),
                };
                sections.push(Section {
                    kind,
                    title: text.clone(),
                    level: 2,
                    heading_line_index: start,
                    start_line: start,
                    end_line: end,
                    blocks,
                });
                i = end + 1;
            } else {
                i += 1;
            }
        }
    }

    DocModel {
        title,
        title_line_index,
        sections,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_lines(name: &str) -> Vec<String> {
        let path = format!("{}/../../fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"))
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn reads_the_title_from_the_first_h1() {
        let model = scan_document(&fixture_lines("full-day.md"));
        assert_eq!(model.title, Some("2026-06-23-TUE".to_string()));
        assert_eq!(model.title_line_index, Some(0));
    }

    #[test]
    fn finds_the_three_standard_sections_by_kind() {
        let model = scan_document(&fixture_lines("full-day.md"));
        let kinds: Vec<SectionKind> = model.sections.iter().map(|s| s.kind).collect();
        assert_eq!(
            kinds,
            vec![SectionKind::Todo, SectionKind::Meetings, SectionKind::Notes]
        );
    }

    #[test]
    fn collects_h3_blocks_under_meetings_with_their_meta() {
        let model = scan_document(&fixture_lines("full-day.md"));
        let meetings = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Meetings)
            .unwrap();
        let names: Vec<&str> = meetings.blocks.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, vec!["Weekly Sync", "Standup"]);

        let sync = &meetings.blocks[0];
        let scheduled = sync.meta.iter().find(|m| m.key == "scheduled").unwrap();
        assert_eq!(scheduled.value, "14:30");
        let keys: Vec<&str> = sync.meta.iter().map(|m| m.key.as_str()).collect();
        assert_eq!(keys, vec!["purpose", "scheduled", "started", "ended"]);
    }

    #[test]
    fn bounds_a_block_to_the_line_before_the_next_heading() {
        let model = scan_document(&fixture_lines("full-day.md"));
        let meetings = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Meetings)
            .unwrap();
        let sync = &meetings.blocks[0];
        assert!(sync.meta_end_line > sync.heading_line_index);
        assert!(sync.end_line >= sync.meta_end_line);
    }

    #[test]
    fn does_not_panic_on_malformed_documents() {
        let model = scan_document(&fixture_lines("malformed.md"));
        assert_eq!(model.title, Some("Just a title".to_string()));
        assert_eq!(model.sections, Vec::new());
    }

    #[test]
    fn treats_a_block_with_no_meta_as_meta_end_line_eq_heading_line_index() {
        let model = scan_document(&fixture_lines("subsections.md"));
        let meetings = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Meetings)
            .unwrap();
        let planning = &meetings.blocks[0];
        assert_eq!(planning.name, "Planning");
        let keys: Vec<&str> = planning.meta.iter().map(|m| m.key.as_str()).collect();
        assert_eq!(keys, vec!["scheduled"]);
    }
}
```

- [ ] **Step 2: Declare the module** — in `crates/slugline-core/src/doc/mod.rs`, replace:

```rust
pub mod classify;
pub mod render_inline;

pub use classify::{Line, classify_line};
pub use render_inline::{Span, render_inline};
```

with:

```rust
pub mod classify;
pub mod render_inline;
pub mod scan;

pub use classify::{Line, classify_line};
pub use render_inline::{Span, render_inline};
pub use scan::{Block, DocModel, MetaEntry, Section, SectionKind, scan_document};
```

- [ ] **Step 3: Run the tests** — `cargo test -p slugline-core doc::scan::`
Expected: PASS (6 tests, all listed above). They should already pass on the first run since the
implementation was written alongside the tests above — but this is the first time `cargo test`
actually exercises them; if any fixture path is wrong you'll see a panic naming the missing file
rather than an assertion failure.

- [ ] **Step 4: Commit**

```bash
git add crates/slugline-core/src/doc/scan.rs crates/slugline-core/src/doc/mod.rs
git commit -m "feat(core): port scanDocument (sections/blocks/meta)"
```

---

### Task 2: Port `derive_agenda` into `core::agenda`

**Files:**
- Create: `crates/slugline-core/src/agenda.rs`
- Modify: `crates/slugline-core/src/lib.rs`

- [ ] **Step 1: Write the failing tests** — create `crates/slugline-core/src/agenda.rs`:

```rust
use crate::doc::{SectionKind, scan_document};

/// A scheduled meeting derived from the `## Meetings` section of a note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgendaItem {
    pub time: String,
    pub name: String,
    pub heading_line_index: usize,
    pub started: Option<String>,
    pub ended: Option<String>,
}

/// Scheduled meetings for a note, sorted ascending by `HH:MM`. Meetings without a
/// scheduled time are omitted. Port of `web/src/lib/agenda.ts`.
pub fn derive_agenda(lines: &[String]) -> Vec<AgendaItem> {
    let model = scan_document(lines);
    let Some(meetings) = model
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Meetings)
    else {
        return Vec::new();
    };

    let mut items = Vec::new();
    for block in &meetings.blocks {
        let Some(scheduled) = block.meta.iter().find(|m| m.key == "scheduled") else {
            continue;
        };
        let time = scheduled.value.trim();
        if time.is_empty() {
            continue;
        }
        items.push(AgendaItem {
            time: time.to_string(),
            name: block.name.clone(),
            heading_line_index: block.heading_line_index,
            started: block
                .meta
                .iter()
                .find(|m| m.key == "started")
                .map(|m| m.value.trim().to_string()),
            ended: block
                .meta
                .iter()
                .find(|m| m.key == "ended")
                .map(|m| m.value.trim().to_string()),
        });
    }
    items.sort_by(|a, b| a.time.cmp(&b.time));
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_lines(name: &str) -> Vec<String> {
        let path = format!("{}/../../fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"))
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn lists_scheduled_meetings_sorted_by_time() {
        let items = derive_agenda(&fixture_lines("full-day.md"));
        let names: Vec<&str> = items.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["Standup", "Weekly Sync"]);
        assert_eq!(items[0].time, "09:00");
    }

    #[test]
    fn captures_started_ended_status_when_present() {
        let items = derive_agenda(&fixture_lines("full-day.md"));
        let sync = items.iter().find(|i| i.name == "Weekly Sync").unwrap();
        assert_eq!(sync.ended, Some("15:02".to_string()));
    }

    #[test]
    fn omits_meetings_without_a_scheduled_time() {
        let lines: Vec<String> = vec![
            "## Meetings".into(),
            "### A".into(),
            "meta:scheduled 10:00".into(),
            "### B".into(),
            "".into(),
        ];
        let names: Vec<String> = derive_agenda(&lines).into_iter().map(|i| i.name).collect();
        assert_eq!(names, vec!["A".to_string()]);
    }

    #[test]
    fn returns_empty_when_there_is_no_meetings_section() {
        let lines: Vec<String> = vec!["# T".into(), "".into(), "## Notes".into(), "".into()];
        assert_eq!(derive_agenda(&lines), Vec::new());
    }
}
```

- [ ] **Step 2: Declare the module** — in `crates/slugline-core/src/lib.rs`, replace:

```rust
//! Slugline core: headless domain logic (no UI framework dependency).
pub mod config;
pub mod date;
pub mod dates;
pub mod doc;
pub mod editor;
pub mod store;
pub mod tabs;
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
```

(`todos` is added to this same list in Task 3, right after `tabs` — don't add it yet.)

- [ ] **Step 3: Run the tests** — `cargo test -p slugline-core agenda::`
Expected: PASS (4 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/slugline-core/src/agenda.rs crates/slugline-core/src/lib.rs
git commit -m "feat(core): port deriveAgenda"
```

---

### Task 3: Port `extract_todos`/`window_dates` into `core::todos`

**Files:**
- Create: `crates/slugline-core/src/todos.rs`
- Modify: `crates/slugline-core/src/lib.rs`

- [ ] **Step 1: Write the failing tests** — create `crates/slugline-core/src/todos.rs`:

```rust
use crate::dates::add_days;
use crate::doc::{Line, SectionKind, classify_line, scan_document};

/// A single `- [ ]`/`- [x]` line inside the `## To Do` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoItem {
    pub text: String,
    pub done: bool,
    pub line_index: usize,
}

/// All of one date's todos, grouped for the sidebar's 7-day aggregation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoGroup {
    pub date: String,
    pub todos: Vec<TodoItem>,
}

/// Task items in the `## To Do` section (both states), skipping blanks.
/// Port of `web/src/lib/todos.ts` `extractTodos`.
pub fn extract_todos(lines: &[String]) -> Vec<TodoItem> {
    let model = scan_document(lines);
    let Some(section) = model.sections.iter().find(|s| s.kind == SectionKind::Todo) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for i in (section.start_line + 1)..=section.end_line {
        let raw = lines.get(i).map(String::as_str).unwrap_or("");
        if let Line::Task { done, text } = classify_line(raw)
            && !text.trim().is_empty()
        {
            out.push(TodoItem {
                text,
                done,
                line_index: i,
            });
        }
    }
    out
}

/// The `days` dates ending on `active_date` (inclusive), most-recent first.
/// Port of `web/src/lib/todos.ts` `windowDates` (the TS default of `days = 7` has no
/// Rust equivalent; callers pass `7` explicitly).
pub fn window_dates(active_date: &str, days: usize) -> Vec<String> {
    (0..days as i64)
        .map(|i| add_days(active_date, -i))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_lines(name: &str) -> Vec<String> {
        let path = format!("{}/../../fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"))
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn extracts_task_items_with_done_state_and_line_indices() {
        let todos = extract_todos(&fixture_lines("full-day.md"));
        let texts: Vec<&str> = todos.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["Buy milk", "Send invoice", "Prep deck _(Weekly Sync)_"]
        );
        let done: Vec<bool> = todos.iter().map(|t| t.done).collect();
        assert_eq!(done, vec![false, true, false]);
        assert_eq!(todos[0].line_index, 4);
    }

    #[test]
    fn returns_empty_without_a_todo_section() {
        let lines: Vec<String> = vec!["# T".into(), "".into(), "## Notes".into(), "".into()];
        assert_eq!(extract_todos(&lines), Vec::new());
    }

    #[test]
    fn returns_7_dates_most_recent_first_ending_on_the_active_date() {
        let d = window_dates("2026-06-23", 7);
        assert_eq!(d.len(), 7);
        assert_eq!(d[0], "2026-06-23");
        assert_eq!(d[6], "2026-06-17");
    }
}
```

- [ ] **Step 2: Declare the module** — in `crates/slugline-core/src/lib.rs`, replace:

```rust
pub mod agenda;
pub mod config;
pub mod date;
pub mod dates;
pub mod doc;
pub mod editor;
pub mod store;
pub mod tabs;
```

with:

```rust
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

- [ ] **Step 3: Run the whole core test suite** — `cargo test -p slugline-core`
Expected: PASS — 113 tests (100 from Phase 3 + 6 `doc::scan::` + 4 `agenda::` + 3 `todos::`).

- [ ] **Step 4: Format + clippy the core crate** —
`cargo fmt --all -- --check && cargo clippy -p slugline-core --all-targets -- -D warnings`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add crates/slugline-core/src/todos.rs crates/slugline-core/src/lib.rs
git commit -m "feat(core): port extractTodos/windowDates"
```

---

### Task 4: Add the `STATUS_BAR` color token

**Files:**
- Modify: `crates/slugline/src/ui/palette.rs`

- [ ] **Step 1: Add the token** — in `crates/slugline/src/ui/palette.rs`, insert after `ACCENT`:

```rust
pub const BLOCKQUOTE_BORDER: Color = hex(0x3b82f6);
pub const ACCENT: Color = hex(0x2f6df6);
pub const STATUS_BAR: Color = hex(0x1f2535);
```

(Matches the web's dark-theme `--status-bar: #1f2535` — the panel-separator color, distinct from
`RULE`'s `#2d3650`, which the editor pane uses for the active-line hairline border. Reusing `RULE`
here would be a silent detail loss since the web deliberately uses two different separator shades.)

- [ ] **Step 2: No build/test yet** — `palette` is a leaf module with no tests of its own; this
compiles as part of Task 6's build.

---

### Task 5: Rewrite `app.rs` — todo aggregation, pending jump, and the `OpenDateAndLine` message

**Files:**
- Modify: `crates/slugline/src/app.rs`

This task wires all of the new state and messages except the two new UI files (`ui/agenda.rs`,
`ui/todo_list.rs`), which don't exist until Task 6, and the `sidebar::view` signature they require.
`cargo build -p slugline` will fail after this task — that's expected and resolved by Task 6
(mirrors Phase 3's Task 2/4 split for the same reason).

- [ ] **Step 1: Update imports** — replace:

```rust
use slugline_core::dates::{YearMonth, add_days, today_iso, year_month};
use slugline_core::editor::{AppEffect, EditorState, KeyInput, create_editor_state, handle_key};
use slugline_core::store::NotesStore;
use slugline_core::tabs::{
    TabsState, active_date, close_tab, init_tabs, next_tab, open_new_tab, prev_tab, retarget,
};
```

with:

```rust
use slugline_core::dates::{YearMonth, add_days, today_iso, year_month};
use slugline_core::editor::{
    AppEffect, Cursor, EditorState, KeyInput, clamp_cursor, create_editor_state, handle_key,
};
use slugline_core::store::NotesStore;
use slugline_core::tabs::{
    TabsState, active_date, close_tab, init_tabs, next_tab, open_new_tab, prev_tab, retarget,
};
use slugline_core::todos::{TodoGroup, extract_todos, window_dates};
```

- [ ] **Step 2: Add the two new Model fields** — replace:

```rust
    /// Dates (`YYYY-MM-DD`) with a note file on disk, for the calendar's has-note dots.
    notes_with_files: Vec<String>,
    /// The sidebar | main split.
    panes: pane_grid::State<PaneKind>,
```

with:

```rust
    /// Dates (`YYYY-MM-DD`) with a note file on disk, for the calendar's has-note dots.
    notes_with_files: Vec<String>,
    /// The 7-day To Do aggregation shown in the sidebar (dates that have at least one
    /// todo, most-recent first). Unlike the calendar's dots, this is not derived at
    /// render time: it requires reading several files off disk, so it is refreshed via
    /// a `Task` (`refresh_todos_task`) instead. The Agenda section, by contrast, is
    /// derived fresh from `editor.lines` on every `view()` call — it never needs disk
    /// I/O beyond the note already open, so it carries no Model state of its own.
    todo_groups: Vec<TodoGroup>,
    /// Set by `OpenDateAndLine` when the target date differs from the active one:
    /// the cursor jump can't be applied until after the pending `navigate()` finishes
    /// and rebuilds `editor` (which resets the cursor to `0,0`).
    pending_jump_line: Option<usize>,
    /// The sidebar | main split.
    panes: pane_grid::State<PaneKind>,
```

- [ ] **Step 3: Add the two new Message variants** — replace:

```rust
    /// The store's list of dated note files finished loading (has-note dots).
    NotesListed(Vec<String>),
    PrevMonth,
    NextMonth,
    PaneResized(pane_grid::ResizeEvent),
    ToggleSidebar,
}
```

with:

```rust
    /// The store's list of dated note files finished loading (has-note dots).
    NotesListed(Vec<String>),
    /// The 7-day To Do aggregation finished reading from disk.
    TodosRefreshed(Vec<TodoGroup>),
    PrevMonth,
    NextMonth,
    PaneResized(pane_grid::ResizeEvent),
    ToggleSidebar,
    /// An Agenda or To Do row was clicked: jump to `line` in `date`'s note, navigating
    /// there first if it isn't already active.
    OpenDateAndLine(String, usize),
}
```

- [ ] **Step 4: Add the `todo_dates_to_read` pure helper** — insert immediately above
`fn plan_tabs`:

```rust
/// Pure: which of the 7-day To Do window's dates should be read from disk when
/// refreshing the aggregation — `active` always, plus any other date that already has
/// a materialized note file. Mirrors the web's inline filter in `refreshTodos`
/// (`appState.svelte.ts`): "never materialize other days" just to check them for todos.
fn todo_dates_to_read(active: &str, notes_with_files: &[String]) -> Vec<String> {
    window_dates(active, 7)
        .into_iter()
        .filter(|d| d == active || notes_with_files.contains(d))
        .collect()
}

/// Pure: compute the new tab set for a navigation effect. `active`/`today` are injected so
```

(The doc comment for `plan_tabs` already exists below — this step only adds the new function
above it.)

- [ ] **Step 5: Initialize the new fields in `App::new`** — replace:

```rust
            calendar: year_month(&date),
            notes_with_files: Vec::new(),
            panes,
```

with:

```rust
            calendar: year_month(&date),
            notes_with_files: Vec::new(),
            todo_groups: Vec::new(),
            pending_jump_line: None,
            panes,
```

- [ ] **Step 6: Add `refresh_todos_task`** — insert immediately above `fn spawn_save`:

```rust
    /// Refresh the sidebar's 7-day To Do aggregation from disk.
    fn refresh_todos_task(&self) -> Task<Message> {
        let store = self.store.clone();
        let dates = todo_dates_to_read(&self.active_date(), &self.notes_with_files);
        Task::perform(
            async move {
                let mut groups = Vec::new();
                for date in dates {
                    if let Ok(content) = store.read_or_create(&date) {
                        let lines: Vec<String> = content.lines().map(str::to_string).collect();
                        let todos = extract_todos(&lines);
                        if !todos.is_empty() {
                            groups.push(TodoGroup { date, todos });
                        }
                    }
                }
                groups
            },
            Message::TodosRefreshed,
        )
    }

    /// Spawn an atomic save of the current buffer to the active date.
```

- [ ] **Step 7: Wire the refresh into `Saved`, `Navigated`, `OpenDate`/`OpenDateAndLine`, and
`NotesListed`** — replace:

```rust
            Message::Saved { date, res } => {
                self.saving = false;
                match res {
                    Ok(content) => {
                        // Ignore a save that finished for a date we've since navigated away from.
                        if date == self.active_date() {
                            self.last_saved = content;
                            if self.content() == self.last_saved {
                                self.dirty_since = None;
                            }
                        }
                    }
                    Err(e) => {
                        self.error =
                            Some(format!("Save failed \u{2014} edits kept, will retry: {e}"));
                        // dirty_since stays set, so the next Tick retries.
                    }
                }
                Task::none()
            }
            Message::Navigated { tabs, body } => {
                self.loading = false;
                self.tabs = tabs;
                match body {
                    Ok(content) => {
                        self.editor = create_editor_state(
                            content.lines().map(str::to_string).collect(),
                            self.shared_register.clone(),
                        );
                        self.last_saved = content;
                        self.dirty_since = None;
                        self.calendar = year_month(&self.active_date());
                        self.list_notes_task()
                    }
                    Err(e) => {
                        let date = self.active_date();
                        self.error = Some(format!("Failed to load note {date}: {e}"));
                        Task::none()
                    }
                }
            }
            Message::OpenDate(date) => {
                if date == self.active_date() {
                    return Task::none();
                }
                self.navigate(retarget(&self.tabs, &date))
            }
            Message::NotesListed(dates) => {
                self.notes_with_files = dates;
                Task::none()
            }
```

with:

```rust
            Message::Saved { date, res } => {
                self.saving = false;
                match res {
                    Ok(content) => {
                        // Ignore a save that finished for a date we've since navigated away from.
                        if date == self.active_date() {
                            self.last_saved = content;
                            if self.content() == self.last_saved {
                                self.dirty_since = None;
                            }
                            return self.refresh_todos_task();
                        }
                    }
                    Err(e) => {
                        self.error =
                            Some(format!("Save failed \u{2014} edits kept, will retry: {e}"));
                        // dirty_since stays set, so the next Tick retries.
                    }
                }
                Task::none()
            }
            Message::Navigated { tabs, body } => {
                self.loading = false;
                self.tabs = tabs;
                match body {
                    Ok(content) => {
                        self.editor = create_editor_state(
                            content.lines().map(str::to_string).collect(),
                            self.shared_register.clone(),
                        );
                        self.last_saved = content;
                        self.dirty_since = None;
                        self.calendar = year_month(&self.active_date());
                        if let Some(line) = self.pending_jump_line.take() {
                            self.editor.cursor = Cursor { line, col: 0 };
                            self.editor = clamp_cursor(&self.editor);
                        }
                        self.list_notes_task()
                    }
                    Err(e) => {
                        // Don't apply a queued jump against a buffer that never arrived.
                        self.pending_jump_line = None;
                        let date = self.active_date();
                        self.error = Some(format!("Failed to load note {date}: {e}"));
                        Task::none()
                    }
                }
            }
            Message::OpenDate(date) => {
                if date == self.active_date() {
                    return Task::none();
                }
                self.navigate(retarget(&self.tabs, &date))
            }
            Message::OpenDateAndLine(date, line) => {
                if date == self.active_date() {
                    self.editor.cursor = Cursor { line, col: 0 };
                    self.editor = clamp_cursor(&self.editor);
                    return Task::none();
                }
                self.pending_jump_line = Some(line);
                self.navigate(retarget(&self.tabs, &date))
            }
            Message::NotesListed(dates) => {
                self.notes_with_files = dates;
                self.refresh_todos_task()
            }
            Message::TodosRefreshed(groups) => {
                self.todo_groups = groups;
                Task::none()
            }
```

Note the sequencing: `NotesListed` (not `Navigated` directly) is what triggers
`refresh_todos_task()`, because `todo_dates_to_read` needs an up-to-date `notes_with_files` to know
which non-active dates already have files — and `list_notes_task()` (kicked off from `Navigated`'s
`Ok` arm, unchanged from Phase 3) is exactly what refreshes that list. This chain — `Navigated` →
`list_notes_task` → `NotesListed` → `refresh_todos_task` → `TodosRefreshed` — runs on every
successful navigation *and* at startup (`App::boot()` already calls `list_notes_task()` directly).
`Saved`, by contrast, refreshes todos directly without going through `notes_with_files` again,
mirroring the web's `flush()` (which calls `refreshTodos()` but not `refreshNotesList()`) — a save
never changes which dates have files, only what one file contains.

- [ ] **Step 8: Pass the new data into `sidebar::view`** — replace:

```rust
                    PaneKind::Sidebar => sidebar::view(
                        self.calendar,
                        &today_iso(),
                        &self.active_date(),
                        &self.notes_with_files,
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
                    ),
```

- [ ] **Step 9: Add the new tests** — insert at the end of the `#[cfg(test)] mod tests` block,
right before the final closing `}`:

```rust
    #[test]
    fn todo_dates_to_read_always_includes_active_and_known_files() {
        let existing = vec!["2026-06-20".to_string(), "2026-06-23".to_string()];
        let dates = todo_dates_to_read("2026-06-23", &existing);
        // 2026-06-23 (active, always) and 2026-06-20 (has a file); the other 5 days in
        // the window have neither property and are skipped.
        assert_eq!(
            dates,
            vec!["2026-06-23".to_string(), "2026-06-20".to_string()]
        );
    }

    #[test]
    fn todo_dates_to_read_includes_the_active_date_even_without_a_file() {
        let dates = todo_dates_to_read("2026-06-23", &[]);
        assert_eq!(dates, vec!["2026-06-23".to_string()]);
    }

    #[test]
    fn todos_refreshed_replaces_the_todo_groups() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let groups = vec![TodoGroup {
            date: "2026-06-23".to_string(),
            todos: Vec::new(),
        }];
        let _ = app.update(Message::TodosRefreshed(groups.clone()));
        assert_eq!(app.todo_groups, groups);
    }

    #[test]
    fn open_date_and_line_on_the_active_date_jumps_the_cursor_without_navigating() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let _ = app.update(Message::OpenDateAndLine("2026-06-23".to_string(), 2));
        assert_eq!(app.editor.cursor.line, 2);
        assert!(!app.loading);
    }

    #[test]
    fn open_date_and_line_to_a_new_date_navigates_and_queues_the_jump() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let _ = app.update(Message::OpenDateAndLine("2026-06-24".to_string(), 3));
        assert!(app.loading);
        assert_eq!(app.pending_jump_line, Some(3));
    }

    #[test]
    fn navigated_applies_a_pending_jump_line() {
        let (_dir, mut app) = temp_app("2026-06-23");
        app.pending_jump_line = Some(2);
        let tabs = init_tabs("2026-06-24");
        let _ = app.update(Message::Navigated {
            tabs,
            body: Ok("# a\n## To Do\nfoo\n".to_string()),
        });
        assert_eq!(app.editor.cursor.line, 2);
        assert_eq!(app.pending_jump_line, None);
    }

    #[test]
    fn navigated_error_clears_a_pending_jump_line() {
        let (_dir, mut app) = temp_app("2026-06-23");
        app.pending_jump_line = Some(2);
        let tabs = init_tabs("2026-06-24");
        let _ = app.update(Message::Navigated {
            tabs,
            body: Err("boom".to_string()),
        });
        assert_eq!(app.pending_jump_line, None);
    }

    #[test]
    fn saved_success_on_the_active_date_updates_last_saved() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let _ = app.update(Message::Saved {
            date: "2026-06-23".to_string(),
            res: Ok("# updated\n".to_string()),
        });
        assert_eq!(app.last_saved, "# updated\n");
    }

    #[test]
    fn saved_success_on_a_stale_date_is_ignored() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let before = app.last_saved.clone();
        let _ = app.update(Message::Saved {
            date: "2026-06-24".to_string(),
            res: Ok("# stale\n".to_string()),
        });
        assert_eq!(app.last_saved, before);
    }
```

- [ ] **Step 10: Attempt a build** — `cargo build -p slugline`
Expected: fails only because `crate::ui::agenda`/`crate::ui::todo_list` don't exist yet and
`sidebar::view` still has the old 4-argument signature — Task 6 adds/fixes both. This mirrors
Phase 3's Task 2/4 split.

(Do not commit yet — Task 6 finishes the crate so it builds and tests pass, then Tasks 4–6 commit
together, since the crate is only buildable as a whole.)

---

### Task 6: Build `ui/agenda.rs` and `ui/todo_list.rs`, and rewrite `ui/sidebar.rs`

**Files:**
- Create: `crates/slugline/src/ui/agenda.rs`
- Create: `crates/slugline/src/ui/todo_list.rs`
- Modify: `crates/slugline/src/ui/sidebar.rs`
- Modify: `crates/slugline/src/ui/mod.rs`

- [ ] **Step 1: Create the Agenda widget** — create `crates/slugline/src/ui/agenda.rs`:

```rust
use iced::widget::{button, column, container, row, span, text};
use iced::{Element, Length};

use slugline_core::agenda::{AgendaItem, derive_agenda};

use crate::app::Message;
use crate::ui::palette;

/// The sidebar's Agenda section: scheduled meetings for the currently open note,
/// derived fresh from its lines on every render (no stored state, mirroring the
/// web's `$derived(deriveAgenda(app.editor.lines))`). Port of
/// `web/src/lib/components/Agenda.svelte`.
pub fn view<'a>(lines: &[String], active: &str) -> Element<'a, Message> {
    let items = derive_agenda(lines);

    let header = container(text("Agenda").size(13).color(palette::HEADING[1]));
    let body: Element<'a, Message> = if items.is_empty() {
        text("No scheduled meetings")
            .size(12)
            .color(palette::MUTED)
            .into()
    } else {
        let mut list = column![].spacing(2);
        for item in items {
            list = list.push(agenda_row(item, active));
        }
        list.into()
    };

    container(column![header, body].spacing(6).width(Length::Fill))
        .padding([10, 12])
        .style(|_theme| container::Style {
            border: iced::Border {
                color: palette::STATUS_BAR,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn agenda_row<'a>(item: AgendaItem, active: &str) -> Element<'a, Message> {
    let done = item.ended.is_some();
    let name_color = if done {
        palette::TODO_DONE
    } else {
        palette::FG
    };

    let mut label = row![
        text(item.time).size(12).color(palette::ACCENT),
        iced::widget::rich_text([span(item.name).color(name_color).strikethrough(done)]).size(12),
    ]
    .spacing(6);
    if done {
        label = label.push(text("\u{2713}").size(11).color(palette::TODO_DONE));
    }

    button(label)
        .padding([2, 4])
        .width(Length::Fill)
        .on_press(Message::OpenDateAndLine(
            active.to_string(),
            item.heading_line_index,
        ))
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
```

`agenda_row` takes `item: AgendaItem` by value (not `&AgendaItem`) deliberately: `derive_agenda`
returns owned `Vec<AgendaItem>` freshly built on every call (there's no long-lived document to
borrow from), so moving each item into its row — and from there into the `on_press` closure's
`item.heading_line_index` and the `span`'s owned `String`s — avoids a lifetime tangle for no
benefit, matching the same reasoning Phase 3's `day_cell` used for taking `cell` data by value out
of a loop-local `Vec`.

- [ ] **Step 2: Create the To Do widget** — create `crates/slugline/src/ui/todo_list.rs`:

```rust
use iced::widget::{button, column, container, row, span, text};
use iced::{Element, Length};

use slugline_core::todos::{TodoGroup, TodoItem};

use crate::app::Message;
use crate::ui::palette;

/// The sidebar's To Do section: the 7-day aggregation kept fresh in `App::todo_groups`
/// (a `Task`-driven disk read, unlike Agenda's per-render derivation — see design
/// Section 5). Port of `web/src/lib/components/TodoList.svelte`.
pub fn view<'a>(groups: &[TodoGroup]) -> Element<'a, Message> {
    let header = container(text("To Do").size(13).color(palette::HEADING[1]));

    let body: Element<'a, Message> = if groups.is_empty() {
        text("No to dos in the last 7 days")
            .size(12)
            .color(palette::MUTED)
            .into()
    } else {
        let mut list = column![].spacing(8);
        for group in groups {
            list = list.push(group_view(group));
        }
        list.into()
    };

    container(column![header, body].spacing(6).width(Length::Fill))
        .padding([10, 12])
        .style(|_theme| container::Style {
            border: iced::Border {
                color: palette::STATUS_BAR,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn group_view<'a>(group: &TodoGroup) -> Element<'a, Message> {
    let mut list = column![text(group.date.clone()).size(11).color(palette::MUTED)].spacing(2);
    for todo in &group.todos {
        list = list.push(todo_row(group.date.clone(), todo));
    }
    list.into()
}

fn todo_row<'a>(date: String, todo: &TodoItem) -> Element<'a, Message> {
    let box_glyph = if todo.done { "\u{2611}" } else { "\u{2610}" }; // ☑ / ☐
    let text_color = if todo.done {
        palette::TODO_DONE
    } else {
        palette::FG
    };

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
```

`todo_row` takes `group.date.clone()` as an owned `String` (not `&group.date`) because
`Message::OpenDateAndLine` owns its `String` field — the clone happens once per row, on every
`view()`, which is the same cost Phase 3's `day_cell` already pays for `cell.date.clone()` in the
calendar grid (42 cells vs. up to ~dozens of todo rows here; both are cheap, small strings).

- [ ] **Step 3: Rewrite `sidebar.rs`** — replace the whole file:

```rust
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use slugline_core::dates::YearMonth;
use slugline_core::todos::TodoGroup;

use crate::app::Message;
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
        calendar::view(calendar_month, today, active, notes_with_files),
        agenda::view(lines, active),
        todo_list::view(todo_groups),
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

(Only the `view` function's body and signature change — `collapsed_rail` is copied unchanged from
Phase 3 for context; you can leave it as-is if your editor supports a partial replace instead.)

- [ ] **Step 4: Declare both new modules** — `crates/slugline/src/ui/mod.rs`:

```rust
pub mod agenda;
pub mod calendar;
pub mod editor_pane;
pub mod palette;
pub mod sidebar;
pub mod tab_strip;
pub mod todo_list;
```

- [ ] **Step 5: Build the whole app** — `cargo build -p slugline`
Expected: compiles clean (`app.rs`'s `view()` now resolves the new `sidebar::view` signature,
which resolves `agenda::view`/`todo_list::view`).

- [ ] **Step 6: Run the tests to verify they pass** — `cargo test -p slugline`
Expected: PASS — 28 tests (19 from Phase 3 + 9 new `app::tests` from Task 5).

- [ ] **Step 7: Commit Tasks 4–6 together** (the crate only builds with all three):

```bash
git add crates/slugline/src/app.rs crates/slugline/src/ui/
git commit -m "feat(app): sidebar Agenda + 7-day To Do aggregation, click-to-navigate"
```

---

### Task 7: Workspace hygiene gate + manual smoke

**Files:** none (verification only)

- [ ] **Step 1: Full workspace test** — `cargo test --workspace`
Expected: green — `slugline-core` (113 tests: 100 from Phase 3 plus 6 `doc::scan::`, 4 `agenda::`,
3 `todos::`) and `slugline` (28 tests: 19 from Phase 3 plus 9 new `app::tests`).

- [ ] **Step 2: Format + clippy** —
`cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean. Fix and re-run if needed.

- [ ] **Step 3: Manual smoke — Agenda, To Do, click-to-navigate**

Create a scratch notes directory with a note that has scheduled meetings and todos (the repo's
`fixtures/full-day.md` is exactly this shape), copied to today's date so it's the one that opens:

```bash
mkdir -p /tmp/slugline-smoke
cp fixtures/full-day.md "/tmp/slugline-smoke/$(date +%Y-%m-%d).md"
```

Run: `cargo run -p slugline -- --notes-dir /tmp/slugline-smoke`

Verify all of:
1. The sidebar shows, top to bottom: the calendar (unchanged from Phase 3), an "Agenda" section
   listing "09:00　Standup" and "14:30　Weekly Sync" (Weekly Sync struck through with a ✓, since the
   fixture gives it an `ended` time; Standup isn't), and a "To Do" section showing today's date
   with "Buy milk" (unchecked), "Send invoice" (☑, struck through), and "Prep deck _(Weekly Sync)_"
   (unchecked, markdown emphasis rendered literally since To Do rows are plain text, not
   `render_inline` — matching the web, which also renders `todo.text` as plain text).
2. Click "Weekly Sync" in the Agenda: the editor scrolls/cursor jumps to the `### Weekly Sync`
   heading line in the currently-open note. No navigation happens (it's the same date already
   open) — confirm by checking the tab strip's active tab is unchanged.
3. Click "Buy milk" in the To Do section: the cursor jumps to that `- [ ] Buy milk` line in the
   editor (same date, so also no navigation).
4. In the calendar, click a day 2-3 days before today with no note yet, type a `- [ ] test` line
   under a `## To Do` heading (materialize the section first if the template doesn't have one —
   the default template does), then navigate back to today via `Ctrl-t` or clicking today's cell.
   Wait ~1s for autosave. The To Do section should now show a second date group with "test" listed
   — confirming the 7-day aggregation picks up a different date's todos once that date has a file.
5. Click the "test" todo under the earlier date's group: the app navigates to that date (tab strip
   updates, editor content changes) and the cursor lands on the `test` line — confirming
   cross-date `OpenDateAndLine` navigates first, then jumps.
6. Resize/collapse the sidebar (Phase 3 features) — still works, and the Agenda/To Do sections
   scroll together with the calendar in the resized width.
7. Everything from Phase 2/3's smoke tests still works unmodified: day/tab navigation, shared
   register, flush-before-navigate, autosave, flush-on-exit, calendar month paging.

- [ ] **Step 4: Clean up the smoke-test directory**

```bash
rm -rf /tmp/slugline-smoke
```

- [ ] **Step 5: Commit any fixups**

```bash
git add -A
git commit -m "chore: fmt + clippy clean for phase 4" || echo "nothing to commit"
```

---

## Self-Review (performed while writing this plan)

- **Verification method:** every piece of this plan (`scan_document`'s traversal logic and its
  interaction with `classify_line`'s `Line` enum, the `let`-chain syntax used in `find_boundary_end`
  and `extract_todos`, `rich_text`/`span`/`strikethrough` for done-item styling, the
  `button::Style`/`Status` hover closures, `scrollable` wrapping a stacked `column`, the
  `Cursor`/`clamp_cursor` reuse for jump-to-line) was implemented in a disposable scratch worktree
  off the real `iced-port` tip (commit `94549c2`), compiled, tested (`cargo test --workspace`:
  113 `slugline-core` + 28 `slugline` tests green), formatted, linted (`cargo clippy --workspace
  --all-targets -- -D warnings`: clean), and smoke-run (`cargo run -p slugline` against a notes
  directory seeded with `fixtures/full-day.md` as today's note, confirmed the window opens with
  Agenda/To Do sections populated and no panics) before being copied into this document. The
  scratch worktree was then removed and its branch deleted — none of this work is committed on
  `iced-port` yet; that's what executing this plan does.
- **Spec coverage:** implements design Section 2's `TodosRefreshed(Vec<TodoGroup>)` Message exactly
  as named in the sketch, and Section 4's "sidebar pane (calendar → agenda → todos, stacked)"
  layout order. Matches design Section 5's data-flow description ("`refreshTodos()`... `Task`s so
  results arrive as Messages") via `refresh_todos_task`. Implements roadmap Phase 4's exact scope:
  "Agenda derivation + 7-day To Do aggregation, click-to-navigate" — Agenda via `derive_agenda` +
  `ui/agenda.rs`, the aggregation via `core::todos` + `App::todo_groups` + `refresh_todos_task`,
  and click-to-navigate via `Message::OpenDateAndLine`, used by both new widgets.
- **Type consistency (against the real committed/verified code):** `TodoGroup { date: String,
  todos: Vec<TodoItem> }` and `TodoItem { text: String, done: bool, line_index: usize }` from
  `core::todos` are used identically in `app.rs`, `ui/todo_list.rs`. `AgendaItem { time, name,
  heading_line_index, started, ended }` from `core::agenda` is used identically in
  `ui/agenda.rs` (`started` is ported and stored for parity with the web's `AgendaItem`, even
  though — like the web's `Agenda.svelte`, which also never reads `item.started` — no Phase 4 UI
  surfaces it; `ended` alone drives the done/struck-through state, matching `Agenda.svelte`'s
  `class:done={!!item.ended}`). `Message::OpenDateAndLine(String, usize)` is matched exactly once
  in `update()` and constructed identically (owned `String` date, `usize` line) from both
  `ui/agenda.rs` and `ui/todo_list.rs`. `SectionKind::{Todo, Meetings, Notes, Other}` from
  `core::doc::scan` is matched exhaustively everywhere it's used (`scan_document`'s blocks-or-not
  branch, `derive_agenda`, `extract_todos`).
- **Placeholder scan:** no `todo!()`/TODO/"handle later" in any shipped code; every step shows the
  exact code to write, not a description of it.
- **Deliberate divergences (noted in-plan):** (1) `window_dates`'s TS default parameter (`days =
  7`) becomes a required second argument in Rust — there is exactly one call site (`app.rs`), which
  always passes `7`, so this loses nothing. (2) To Do row text is rendered as plain (struck-through
  when done) text rather than through `render_inline`, matching the web's `TodoList.svelte`, which
  also renders `todo.text` as a plain `{todo.text}` interpolation, not through its HTML-emitting
  inline renderer — Agenda's meeting *names* are likewise plain text in the web (`{item.name}`), so
  `ui/agenda.rs` matches that too. Per-section chevron collapse and the Ctrl/Cmd-click-to-new-tab
  interaction are both explicitly deferred (see "Deferred on purpose") rather than silently dropped.
- **Reachability:** `OpenDateAndLine`/`TodosRefreshed` are both reachable now — the former via
  mouse (Agenda rows, To Do rows), the latter automatically after every navigation, at startup, and
  after every same-tab save. No dead code waiting on a later phase.
