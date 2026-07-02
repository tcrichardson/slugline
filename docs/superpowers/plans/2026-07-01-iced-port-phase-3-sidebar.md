# Phase 3 — Sidebar (Calendar) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **This is a port.** Behavioral truth for the pure calendar math is `web/src/lib/dates.ts`
> (+ `dates.test.ts`, for `monthGrid`/`yearMonth`) and `web/src/lib/components/Calendar.svelte` +
> `Sidebar.svelte` (for the has-note dots, click-to-open, and month navigation). The
> `prevMonth`/`nextMonth` arithmetic lives inline in `web/src/lib/appState.svelte.ts` and has no
> existing TS test — this plan writes a fresh one, since it's new logic being ported, not existing
> logic being re-verified.
>
> **Iced API caution:** method/type names target iced `0.13.x` (`pane_grid`, `button::Style`).
> Every non-trivial API call in this plan was verified against the actual pinned `iced_widget
> 0.13.4` source and compiled + tested in a scratch branch before being written down — but if a
> signature has drifted in your checkout, confirm it and adjust; the *intent* is the contract.

**Goal:** Add a resizable, collapsible sidebar containing a calendar: month grid with has-note
dots, click-to-open navigation, and prev/next month buttons — built with Iced's `pane_grid` so the
sidebar | main split is drag-resizable, and the whole sidebar can collapse to a slim rail.

**Architecture:** Two new pure `slugline-core::dates` items (`MonthCell`, `YearMonth`,
`month_grid`, `year_month`) with ported tests; two new UI-only files (`ui/calendar.rs`,
`ui/sidebar.rs`) with no framework-agnostic logic beyond a tiny pure `month_label` helper; and an
`app.rs` rewrite that adds a `pane_grid::State<PaneKind>` to the Model, a handful of new
`Message` variants (`OpenDate`, `NotesListed`, `PrevMonth`, `NextMonth`, `PaneResized`,
`ToggleSidebar`), and a `view()` that switches between the two-pane `pane_grid` (expanded) and a
plain row with a collapse rail (collapsed).

**Tech Stack:** Rust, `chrono` (already a `slugline-core` dep) for calendar-grid math; Iced
`0.13.x` `pane_grid` widget for the resizable/collapsible split.

---

## Prerequisites

- **Phases 0, 1a, 1b, 1c, 2 are complete and committed on `iced-port`, and `cargo test --workspace`
  is green** (98 `slugline-core` tests + `cli`/`keys`/`app` tests in `slugline`, as of
  `527decf "chore: fmt + clippy clean for phase 2"`). Phase 3 builds directly on the Phase 2 `App`
  (`crates/slugline/src/app.rs`: `TabsState`, `Message::{Navigated,SwitchTab,CloseTab}`,
  `navigate()`), `core::store::NotesStore::list_dates()` (already ported in Phase 0 — reused
  as-is), and `core::dates::{today_iso, add_days}`.

## Scope

**In this phase:**
- `core::dates`: `MonthCell { date, in_month }`, `YearMonth { year, month }`,
  `month_grid(year, month) -> Vec<Vec<MonthCell>>`, `year_month(date) -> YearMonth` — direct ports
  of `monthGrid`/`yearMonth` from `web/src/lib/dates.ts`, with their `dates.test.ts` cases ported.
- A calendar widget (`ui/calendar.rs`): month header with `‹`/`›` buttons, a day-of-week row, and a
  6×7 grid of day cells. Days with a note file on disk get a dot; today gets an outline; the active
  tab's date is filled solid. Clicking a day retargets the active tab to that date (`goToDate`).
- A sidebar widget (`ui/sidebar.rs`) wrapping the calendar with a header bar that has a collapse
  button. Additional sections (agenda, todos) are stacked into this same column in **Phase 4** —
  this phase only has the calendar section.
- The shell's `pane_grid` (design Section 4, "Layout via `pane_grid`"): a resizable sidebar | main
  split, drag-to-resize via `PaneResized`, and a whole-sidebar collapse toggle that swaps the
  `pane_grid` for a plain row with a slim rail.
- `notes_with_files` refresh (`core::store::list_dates()` via a `Task`) after every successful
  navigation and once at startup (`App::boot()`), feeding the calendar's has-note dots.
- The calendar's own displayed month (`calendar: YearMonth` in the Model) resets to the active
  tab's month on every successful navigation, and can be paged independently via `PrevMonth`/
  `NextMonth` without moving the active tab (mirrors `app.calendar` in `appState.svelte.ts`).

**Deferred on purpose:**
- **Ctrl/Cmd-click a calendar day to open it in a new tab** (the web's `openInNewTab` path in
  `Calendar.svelte`'s `onCellClick`). Detecting held modifiers on a plain click in Iced 0.13
  requires tracking `keyboard::Event::ModifiersChanged` in a separate subscription+field, which is
  real added complexity for a secondary interaction the roadmap doesn't call out for this phase
  ("click-to-open", singular). Plain click (`goToDate`) covers the primary interaction; the tab
  strip built in Phase 2 already supports switching/closing multiple tabs once they exist, so this
  is a pure enhancement to pick up later, not a blocker.
- **Per-section chevron collapse** (design Section 4's `TogglePanel(Panel)`). With only one sidebar
  section (calendar) this phase, a generic `Panel` enum would be a speculative abstraction — it
  earns its keep once Agenda/Todos land in **Phase 4** and there's more than one section to
  toggle independently. This phase only builds the whole-sidebar collapse.
- Calendar/agenda/todos interplay, and the 7-day To Do aggregation — **Phase 4**.
- Pane-grid drag-and-drop / multi-split / maximize — out of scope entirely; this is a single fixed
  two-pane split, resizable but not re-arrangeable, matching the design's "resizable/collapsible"
  (not "freely re-tileable") sidebar.

---

## File Structure (files added/changed in Phase 3)

```
crates/slugline-core/
  src/
    dates.rs                       # + MonthCell, YearMonth, month_grid(), year_month()

crates/slugline/
  src/
    app.rs                         # REWRITE: pane_grid Model/Message/update/view, calendar state
    main.rs                        # + App::boot() Task wired into run_with
    ui/mod.rs                      # + pub mod calendar; pub mod sidebar;
    ui/palette.rs                  # + ACCENT color token
    ui/calendar.rs                 # NEW: month grid widget + month_label() (tested)
    ui/sidebar.rs                  # NEW: sidebar header + collapse rail
```

---

### Task 1: Port `month_grid`/`year_month` into `core::dates`

**Files:**
- Modify: `crates/slugline-core/src/dates.rs`

- [ ] **Step 1: Write the failing tests** — append to the `tests` module in
`crates/slugline-core/src/dates.rs` (after the existing `add_days_zero_is_identity_and_bad_input_is_unchanged` test):

```rust
    #[test]
    fn builds_a_6x7_month_grid_with_first_of_month_and_out_of_month_days() {
        let g = month_grid(2026, 6);
        assert_eq!(g.len(), 6);
        assert_eq!(g[0].len(), 7);
        let flat: Vec<&MonthCell> = g.iter().flatten().collect();
        assert!(
            flat.iter()
                .find(|c| c.date == "2026-06-01")
                .unwrap()
                .in_month
        );
        assert!(flat.iter().any(|c| !c.in_month));
    }

    #[test]
    fn extracts_year_and_month() {
        assert_eq!(
            year_month("2026-06-23"),
            YearMonth {
                year: 2026,
                month: 6
            }
        );
    }
```

- [ ] **Step 2: Run tests to verify they fail** — `cargo test -p slugline-core dates::`
Expected: FAIL to compile (`month_grid`/`year_month`/`MonthCell`/`YearMonth` undefined).

- [ ] **Step 3: Extend the import and add the types + functions** — in
`crates/slugline-core/src/dates.rs`, replace the top import line:

```rust
use chrono::{Days, Local, NaiveDate};
```

with:

```rust
use chrono::{Datelike, Days, Local, NaiveDate};
```

then insert the following between the end of `add_days` and the `#[cfg(test)]` line:

```rust
/// A single day cell in a month grid: its ISO date and whether it belongs to the
/// requested month (vs. a leading/trailing day borrowed from an adjacent month).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthCell {
    pub date: String,
    pub in_month: bool,
}

/// A calendar year/month pair (`month` is 1-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YearMonth {
    pub year: i32,
    pub month: u32,
}

/// A 6x7 grid (weeks start Sunday) covering `month` (1-12) of `year`.
pub fn month_grid(year: i32, month: u32) -> Vec<Vec<MonthCell>> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let offset = first.weekday().num_days_from_sunday() as u64;
    let mut cursor = first - Days::new(offset);

    let mut weeks = Vec::with_capacity(6);
    for _ in 0..6 {
        let mut row = Vec::with_capacity(7);
        for _ in 0..7 {
            row.push(MonthCell {
                date: cursor.format("%Y-%m-%d").to_string(),
                in_month: cursor.month() == month,
            });
            cursor = cursor + Days::new(1);
        }
        weeks.push(row);
    }
    weeks
}

/// Extract the year/month of an ISO `YYYY-MM-DD` date.
pub fn year_month(date: &str) -> YearMonth {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| YearMonth {
            year: d.year(),
            month: d.month(),
        })
        .unwrap_or(YearMonth {
            year: 1970,
            month: 1,
        })
}
```

- [ ] **Step 4: Run tests to verify they pass** — `cargo test -p slugline-core dates::`
Expected: PASS (5 tests: the 3 existing `add_days`/`today_iso` tests plus the 2 new ones).

- [ ] **Step 5: Commit**

```bash
git add crates/slugline-core/src/dates.rs
git commit -m "feat(core): port monthGrid/yearMonth for the calendar"
```

---

### Task 2: Rewrite `app.rs` — calendar state, navigation, and `pane_grid` Model

**Files:**
- Modify: `crates/slugline/src/app.rs`

This task adds everything except the two new UI files (`ui/calendar.rs`, `ui/sidebar.rs`), which
don't exist until Task 4. `cargo build -p slugline` will fail after this task — that's expected
and resolved by Task 4 (mirrors Phase 2 Task 4/5's split for the same reason).

- [ ] **Step 1: Update imports and add `PaneKind` + the `INITIAL_SIDEBAR_RATIO` constant** — in
`crates/slugline/src/app.rs`, replace the top of the file:

```rust
use std::time::{Duration, Instant};

use iced::widget::column;
use iced::{Element, Subscription, Task, keyboard, time, window};

use slugline_core::dates::{add_days, today_iso};
use slugline_core::editor::{AppEffect, EditorState, KeyInput, create_editor_state, handle_key};
use slugline_core::store::NotesStore;
use slugline_core::tabs::{
    TabsState, active_date, close_tab, init_tabs, next_tab, open_new_tab, prev_tab, retarget,
};

use crate::keys::key_string;
use crate::ui::{editor_pane, tab_strip};

const SAVE_DEBOUNCE: Duration = Duration::from_millis(750);

pub struct App {
    store: NotesStore,
    tabs: TabsState,
    editor: EditorState,
    /// Yank register carried across tabs/navigation (mirrors the web `sharedRegister`).
    shared_register: Vec<String>,
    last_saved: String,
    dirty_since: Option<Instant>,
    saving: bool,
    /// True while a navigation load is in flight; suppresses autosave ticks so a
    /// mid-navigation buffer is never written to the wrong date.
    loading: bool,
    #[allow(dead_code)]
    error: Option<String>,
}
```

with:

```rust
use std::time::{Duration, Instant};

use iced::widget::{column, pane_grid, row};
use iced::{Element, Length, Subscription, Task, keyboard, time, window};

use slugline_core::dates::{YearMonth, add_days, today_iso, year_month};
use slugline_core::editor::{AppEffect, EditorState, KeyInput, create_editor_state, handle_key};
use slugline_core::store::NotesStore;
use slugline_core::tabs::{
    TabsState, active_date, close_tab, init_tabs, next_tab, open_new_tab, prev_tab, retarget,
};

use crate::keys::key_string;
use crate::ui::{editor_pane, sidebar, tab_strip};

const SAVE_DEBOUNCE: Duration = Duration::from_millis(750);
/// The sidebar's share of the window width when the app starts.
const INITIAL_SIDEBAR_RATIO: f32 = 0.22;

/// The two top-level panes of the shell's `pane_grid` (design Section 4: "Layout via
/// `pane_grid`"). There is only ever this one fixed split — no user-driven splitting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneKind {
    Sidebar,
    Main,
}

pub struct App {
    store: NotesStore,
    tabs: TabsState,
    editor: EditorState,
    /// Yank register carried across tabs/navigation (mirrors the web `sharedRegister`).
    shared_register: Vec<String>,
    last_saved: String,
    dirty_since: Option<Instant>,
    saving: bool,
    /// True while a navigation load is in flight; suppresses autosave ticks so a
    /// mid-navigation buffer is never written to the wrong date.
    loading: bool,
    /// The sidebar calendar's displayed month (independent of the active tab's day).
    calendar: YearMonth,
    /// Dates (`YYYY-MM-DD`) with a note file on disk, for the calendar's has-note dots.
    notes_with_files: Vec<String>,
    /// The sidebar | main split.
    panes: pane_grid::State<PaneKind>,
    /// True when the whole sidebar is collapsed to a slim rail.
    sidebar_collapsed: bool,
    #[allow(dead_code)]
    error: Option<String>,
}
```

- [ ] **Step 2: Add the new `Message` variants and the `shift_month` pure helper** — replace:

```rust
    SwitchTab(usize),
    CloseTab(usize),
    CloseRequested(window::Id),
}
```

with:

```rust
    SwitchTab(usize),
    CloseTab(usize),
    CloseRequested(window::Id),
    /// A calendar day cell was clicked: retarget the active tab to that date.
    OpenDate(String),
    /// The store's list of dated note files finished loading (has-note dots).
    NotesListed(Vec<String>),
    PrevMonth,
    NextMonth,
    PaneResized(pane_grid::ResizeEvent),
    ToggleSidebar,
}

/// Pure: shift a calendar month by `delta` months (may be negative), rolling over years.
/// Ports the web's inline `prevMonth`/`nextMonth` (`appState.svelte.ts`); there is no existing
/// TS test for this arithmetic, so the tests below are new rather than ported.
fn shift_month(ym: YearMonth, delta: i32) -> YearMonth {
    let total = (ym.month as i32 - 1) + delta;
    YearMonth {
        year: ym.year + total.div_euclid(12),
        month: (total.rem_euclid(12) + 1) as u32,
    }
}
```

(`plan_tabs`, directly below, is unchanged.)

- [ ] **Step 3: Initialize the new Model fields in `App::new`, and add `boot()`** — replace:

```rust
        let editor = create_editor_state(content.lines().map(str::to_string).collect(), Vec::new());
        Self {
            store,
            tabs: init_tabs(&date),
            editor,
            shared_register: Vec::new(),
            last_saved: content,
            dirty_since: None,
            saving: false,
            loading: false,
            error,
        }
    }
```

with:

```rust
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
            panes,
            sidebar_collapsed: false,
            error,
        }
    }

    /// The initial `Task` to run once the window opens (called from `main`).
    pub fn boot(&self) -> Task<Message> {
        self.list_notes_task()
    }
```

- [ ] **Step 4: Add `list_notes_task`** — insert this method immediately above
`fn spawn_save`:

```rust
    /// Refresh the calendar's has-note dots from disk.
    fn list_notes_task(&self) -> Task<Message> {
        let store = self.store.clone();
        Task::perform(
            async move { store.list_dates().unwrap_or_default() },
            Message::NotesListed,
        )
    }

    /// Spawn an atomic save of the current buffer to the active date.
```

(the doc comment + signature line for `spawn_save` already exist below it — this step only adds
the new method above it, so after this step the file has both methods back to back.)

- [ ] **Step 5: Update `Navigated`, and add the new `update()` arms** — replace:

```rust
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
                    }
                    Err(e) => {
                        let date = self.active_date();
                        self.error = Some(format!("Failed to load note {date}: {e}"));
                    }
                }
                Task::none()
            }
            Message::SwitchTab(index) => {
```

with:

```rust
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
            Message::PrevMonth => {
                self.calendar = shift_month(self.calendar, -1);
                Task::none()
            }
            Message::NextMonth => {
                self.calendar = shift_month(self.calendar, 1);
                Task::none()
            }
            Message::PaneResized(event) => {
                self.panes.resize(event.split, event.ratio);
                Task::none()
            }
            Message::ToggleSidebar => {
                self.sidebar_collapsed = !self.sidebar_collapsed;
                Task::none()
            }
            Message::SwitchTab(index) => {
```

Note the behavioral change from Phase 2: on a successful `Navigated`, the calendar's displayed
month now resets to the new active date (mirrors `this.calendar = yearMonth(date)` inside the web's
`loadActive`), and a notes-list refresh is kicked off (mirrors `refreshNotesList()`, also inside
`loadActive`) — both only on the `Ok` branch, matching the web, which throws before reaching either
line on a load failure. `OpenDate` on the already-active date is a deliberate no-op divergence from
the web's `goToDate` (which always flushes+reloads even for the current day) — the same divergence
Phase 2 already made for `SwitchTab`, for the same reason (avoid needlessly resetting editor/cursor
state when nothing actually changed).

- [ ] **Step 6: Rewrite `view()`** — replace:

```rust
    pub fn view(&self) -> Element<'_, Message> {
        column![tab_strip::view(&self.tabs), editor_pane::view(&self.editor)].into()
    }
```

with:

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

    fn main_pane(&self) -> Element<'_, Message> {
        column![tab_strip::view(&self.tabs), editor_pane::view(&self.editor)].into()
    }
```

(`subscription()` below is unchanged.)

- [ ] **Step 7: Add the new tests** — in the `#[cfg(test)] mod tests` block, insert the following
between `navigated_swaps_tabs_editor_and_carries_register` and `switch_tab_to_same_index_is_a_noop`:

```rust
    #[test]
    fn navigated_resets_the_calendar_month_to_the_new_date() {
        let (_dir, mut app) = temp_app("2026-06-23");
        app.calendar = YearMonth {
            year: 2020,
            month: 1,
        }; // pretend the user had browsed the calendar elsewhere
        let tabs = init_tabs("2026-09-05");
        let _ = app.update(Message::Navigated {
            tabs,
            body: Ok("# hi\n".to_string()),
        });
        assert_eq!(
            app.calendar,
            YearMonth {
                year: 2026,
                month: 9
            }
        );
    }

    #[test]
    fn navigated_error_leaves_the_calendar_month_untouched() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let tabs = init_tabs("2026-09-05");
        let _ = app.update(Message::Navigated {
            tabs,
            body: Err("boom".to_string()),
        });
        assert_eq!(
            app.calendar,
            YearMonth {
                year: 2026,
                month: 6
            }
        );
    }

    #[test]
    fn open_date_to_the_active_date_is_a_noop() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let _ = app.update(Message::OpenDate("2026-06-23".to_string()));
        assert!(!app.loading);
    }

    #[test]
    fn open_date_to_a_new_date_retargets_the_active_tab() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let _ = app.update(Message::OpenDate("2026-06-24".to_string()));
        assert!(app.loading); // navigation kicked off; Navigated finishes it
    }

    #[test]
    fn notes_listed_replaces_the_calendar_dot_set() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let _ = app.update(Message::NotesListed(vec![
            "2026-06-20".to_string(),
            "2026-06-23".to_string(),
        ]));
        assert_eq!(
            app.notes_with_files,
            vec!["2026-06-20".to_string(), "2026-06-23".to_string()]
        );
    }

    #[test]
    fn prev_and_next_month_messages_shift_the_calendar() {
        let (_dir, mut app) = temp_app("2026-06-23");
        assert_eq!(
            app.calendar,
            YearMonth {
                year: 2026,
                month: 6
            }
        );
        let _ = app.update(Message::PrevMonth);
        assert_eq!(
            app.calendar,
            YearMonth {
                year: 2026,
                month: 5
            }
        );
        let _ = app.update(Message::NextMonth);
        let _ = app.update(Message::NextMonth);
        assert_eq!(
            app.calendar,
            YearMonth {
                year: 2026,
                month: 7
            }
        );
    }

    #[test]
    fn shift_month_rolls_over_year_boundaries() {
        assert_eq!(
            shift_month(
                YearMonth {
                    year: 2026,
                    month: 1
                },
                -1
            ),
            YearMonth {
                year: 2025,
                month: 12
            }
        );
        assert_eq!(
            shift_month(
                YearMonth {
                    year: 2026,
                    month: 12
                },
                1
            ),
            YearMonth {
                year: 2027,
                month: 1
            }
        );
    }

    #[test]
    fn toggle_sidebar_flips_the_collapsed_flag() {
        let (_dir, mut app) = temp_app("2026-06-23");
        assert!(!app.sidebar_collapsed);
        let _ = app.update(Message::ToggleSidebar);
        assert!(app.sidebar_collapsed);
        let _ = app.update(Message::ToggleSidebar);
        assert!(!app.sidebar_collapsed);
    }
```

- [ ] **Step 8: Attempt a build** — `cargo build -p slugline`
Expected: fails only because `crate::ui::sidebar` (and its `crate::ui::calendar` dependency) do
not exist yet — Task 4 adds them. This mirrors Phase 2's Task 4/5 split.

(Do not commit yet — Task 4 finishes the crate so it builds and tests pass, then Tasks 2–4 commit
together, since the crate is only buildable as a whole. This exact "build fails until the next
task" checkpoint is itself the verification that Step 8 is correct.)

---

### Task 3: Add the `ACCENT` color token

**Files:**
- Modify: `crates/slugline/src/ui/palette.rs`

- [ ] **Step 1: Add the token** — in `crates/slugline/src/ui/palette.rs`, insert after
`BLOCKQUOTE_BORDER`:

```rust
pub const BLOCKQUOTE_BORDER: Color = hex(0x3b82f6);
pub const ACCENT: Color = hex(0x2f6df6);
```

(Same hex as `LINK` — this matches the web's `--accent: #2f6df6` in `web/src/app.css`, which is
coincidentally identical to the link color today; giving it its own named constant, rather than
reusing `LINK`, keeps the calendar's styling independent so Phase 6's per-theme palettes can vary
them separately.)

- [ ] **Step 2: No build/test yet** — `palette` is a leaf module with no tests of its own; this
compiles as part of Task 4's build.

---

### Task 4: Build `ui/calendar.rs` and `ui/sidebar.rs`

**Files:**
- Create: `crates/slugline/src/ui/calendar.rs`
- Create: `crates/slugline/src/ui/sidebar.rs`
- Modify: `crates/slugline/src/ui/mod.rs`

- [ ] **Step 1: Write the failing test** — create `crates/slugline/src/ui/calendar.rs` with the
widget plus its one pure, tested helper:

```rust
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use slugline_core::dates::{MonthCell, YearMonth, month_grid};

use crate::app::Message;
use crate::ui::palette;

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
            .push(container(text(d).size(11).color(palette::MUTED)).center_x(Length::Fixed(CELL)));
    }

    let mut grid = column![dow_row].spacing(2);
    for week in month_grid(calendar.year, calendar.month) {
        let mut wk = row![].spacing(2);
        for cell in &week {
            let has_note = notes_with_files.iter().any(|d| d == &cell.date);
            wk = wk.push(day_cell(cell, today, active, has_note));
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

    let dot = text(if has_note { "\u{2022}" } else { " " }).size(9);
    let label = column![text(day_num).size(12), dot].align_x(Alignment::Center);

    button(label)
        .width(Length::Fixed(CELL))
        .height(Length::Fixed(CELL))
        .padding(0.0)
        .on_press(Message::OpenDate(cell.date.clone()))
        .style(move |_theme, status| {
            let background = if is_selected {
                Some(palette::ACCENT.into())
            } else if status == button::Status::Hovered {
                Some(palette::EDIT_BAR_BG.into())
            } else {
                None
            };
            let text_color = if is_selected {
                palette::BG
            } else if !in_month {
                palette::MUTED
            } else {
                palette::FG
            };
            button::Style {
                background,
                text_color,
                border: iced::Border {
                    color: if is_today {
                        palette::ACCENT
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

`day_cell` takes `cell: &MonthCell` (not `&'a MonthCell`) deliberately: every piece of `cell` data
that ends up in the returned `Element` is copied or cloned out (`is_today`/`is_selected`/`in_month`
are `bool`s computed up front; `day_num` is `.to_string()`-owned; the button's `on_press` argument
is `cell.date.clone()`), so the widget never actually borrows `cell` — giving it `'a` would
incorrectly tie the returned `Element<'a, _>`'s lifetime to the caller's loop-local `week: Vec<MonthCell>`,
which does not live long enough to satisfy `view`'s own `'a`.

- [ ] **Step 2: Create the sidebar wrapper** — create `crates/slugline/src/ui/sidebar.rs`:

```rust
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use slugline_core::dates::YearMonth;

use crate::app::Message;
use crate::ui::calendar;

/// The sidebar pane: a collapse header followed by the calendar section.
/// Additional sections (agenda, todos) land in Phase 4, stacked below the
/// calendar in this same column. Port of `web/src/lib/components/Sidebar.svelte`.
pub fn view<'a>(
    calendar_month: YearMonth,
    today: &str,
    active: &str,
    notes_with_files: &[String],
) -> Element<'a, Message> {
    let header = row![
        container(text("Slugline").size(13)).width(Length::Fill),
        button(text("\u{ab}").size(13)) // «
            .on_press(Message::ToggleSidebar)
            .padding([2, 8]),
    ]
    .align_y(Alignment::Center)
    .padding([8, 10]);

    column![
        header,
        calendar::view(calendar_month, today, active, notes_with_files),
    ]
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

- [ ] **Step 3: Declare both modules** — `crates/slugline/src/ui/mod.rs`:

```rust
pub mod calendar;
pub mod editor_pane;
pub mod palette;
pub mod sidebar;
pub mod tab_strip;
```

- [ ] **Step 4: Build the whole app** — `cargo build -p slugline`
Expected: compiles clean (`app.rs`'s `view()` now resolves `sidebar::view`/`sidebar::collapsed_rail`).

- [ ] **Step 5: Run the tests to verify they pass** — `cargo test -p slugline`
Expected: PASS — `cli`/`keys` tests, `ui::calendar::tests::formats_month_and_year`, and all
`app::tests` including the Task 2 additions (19 tests total in the `slugline` binary crate as of
this phase).

- [ ] **Step 6: Commit Tasks 2–4 together** (the crate only builds with all three):

```bash
git add crates/slugline/src/app.rs crates/slugline/src/main.rs crates/slugline/src/ui/
git commit -m "feat(app): sidebar calendar in a resizable/collapsible pane_grid"
```

(`main.rs` is included here even though it's edited in Task 5 below — Task 5 is a tiny, independent
one-line-of-substance change; committing it separately after Task 4 would leave an intermediate
commit where `App::boot()` exists but is unused, which is harmless but noisier than necessary. If
you're executing strictly task-by-task with a commit gate after every task, it's fine to split this
into two commits instead — just do Task 5 first and swap the commit order.)

---

### Task 5: Wire `App::boot()` into `main.rs`

**Files:**
- Modify: `crates/slugline/src/main.rs`

- [ ] **Step 1: Run the initial notes-list `Task` on startup** — replace:

```rust
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run_with(move || (App::new(store.clone(), date.clone()), Task::none()))
}
```

with:

```rust
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run_with(move || {
            let app = App::new(store.clone(), date.clone());
            let boot = app.boot();
            (app, boot)
        })
}
```

- [ ] **Step 2: Remove the now-unused import** — `main.rs` no longer calls `Task::none()`
directly, so delete this line near the top of the file:

```rust
use iced::Task;
```

- [ ] **Step 3: Build** — `cargo build -p slugline`
Expected: compiles with zero warnings (confirms the `iced::Task` import removal was correct and
nothing else referenced it).

- [ ] **Step 4: Commit** (or fold into Task 4's commit — see the note at the end of Task 4):

```bash
git add crates/slugline/src/main.rs
git commit -m "feat(app): run the initial notes-list task on startup"
```

---

### Task 6: Workspace hygiene gate + manual smoke

**Files:** none (verification only)

- [ ] **Step 1: Full workspace test** — `cargo test --workspace`
Expected: green — `slugline-core` (102 tests: everything from Phase 2 plus the 2 new `dates::`
tests) and `slugline` (20 tests: everything from Phase 2 plus 8 new `app::tests` and 1 new
`ui::calendar::tests`).

- [ ] **Step 2: Format + clippy** — `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean. Fix and re-run if needed.

- [ ] **Step 3: Manual smoke — calendar navigation, month paging, resize, collapse**

Run: `cargo run -p slugline -- --notes-dir ./dev-notes`

Verify all of:
1. The window opens split into a left sidebar (header "Slugline" + a calendar showing the current
   month, today outlined, today's cell filled since it's also the active tab) and the editor on
   the right.
2. Click a different day in the calendar (e.g. a day earlier this month with no note yet): the
   editor loads that day's (materialized) note, the tab strip's single tab now shows that date, and
   the calendar's filled cell moves to the clicked day. Click back to today.
3. Has-note dots: after step 2, re-open the app (or navigate away and back) — the day you clicked
   now shows a dot (it has a note file on disk), while other still-unopened days don't.
4. Click `‹`/`›` to page the calendar to a different month; the active (filled) day disappears from
   view since it's a different month, but clicking `‹`/`›` back to the current month brings it back
   filled — confirming `calendar` state is independent of the active tab.
5. Navigating with keyboard `[`/`]`/`Ctrl-t` (from Phase 2) still works, and after any such
   navigation the calendar auto-pages back to that date's month (e.g. press `]` repeatedly across a
   month boundary and confirm the calendar header's month label changes to match).
6. Drag the sidebar/editor divider left and right — the split resizes. Resize to a very narrow or
   very wide sidebar; it clamps sanely rather than disappearing or overrunning the window (iced's
   built-in `0.1..0.9` ratio clamp).
7. Click the sidebar's `«` button — the sidebar collapses to a slim rail with a single `»` button.
   Click `»` — the sidebar reappears at its previous width. Repeat resize-then-collapse-then-expand
   once to confirm the ratio survives a collapse/expand cycle (it does: collapsing doesn't touch
   `panes`, it only swaps which `view()` branch renders).
8. Everything from Phase 2's smoke test still works unmodified: day/tab navigation, shared register,
   flush-before-navigate, autosave, flush-on-exit.

- [ ] **Step 4: Commit any fixups**

```bash
git add -A
git commit -m "chore: fmt + clippy clean for phase 3" || echo "nothing to commit"
```

---

## Self-Review (performed while writing this plan)

- **Verification method:** every non-trivial piece of this plan (the `pane_grid` construction and
  resize wiring, the `button::Style`/`Status` closure shape, `container::center_x`,
  `Padding`'s array-literal conversions, `NaiveDate`'s `Days` operators, the day-cell lifetime
  issue) was implemented in a disposable scratch branch off the real `iced-port` tip, compiled,
  tested (`cargo test --workspace`: 100 + 20 tests green), formatted, linted
  (`cargo clippy --workspace --all-targets -- -D warnings`: clean), and smoke-run
  (`cargo run -p slugline`, confirmed the window opens and materializes today's note with no
  panics) before being copied into this document. The scratch branch was then discarded — none of
  this work is committed on `iced-port` yet; that's what executing this plan does.
- **Spec coverage:** implements design Section 4's "Layout via `pane_grid`" (resizable sidebar |
  main split via `on_resize`/`ResizeEvent`) and "the whole sidebar can collapse too" (the
  `sidebar_collapsed` branch in `view()`), plus roadmap Phase 3's exact scope: "Sidebar: calendar
  (has-note dots, click-to-open, month nav) inside a resizable/collapsible `pane_grid`." Matches
  design Section 5's "Mouse actions (calendar day, ...) emit `OpenDate`" (the Message is named
  exactly `OpenDate` as the design's Section 2 sketch names it) and Section 3/5's has-note-dot data
  source (`core::store::list_dates()`, already ported in Phase 0, reused rather than re-invented).
- **Type consistency (against the real committed/verified code):** `YearMonth { year: i32, month:
  u32 }` and `MonthCell { date: String, in_month: bool }` from `core::dates` are used identically
  in `app.rs`, `ui/calendar.rs`, and `ui/sidebar.rs`. `Message::{OpenDate, NotesListed, PrevMonth,
  NextMonth, PaneResized, ToggleSidebar}` are each matched exactly once in `update()` and referenced
  with matching argument shapes from `ui/calendar.rs`/`ui/sidebar.rs`. `PaneKind::{Sidebar, Main}`
  is exhaustively matched in `view()`. `NotesStore::list_dates() -> io::Result<Vec<String>>`
  (Phase 0) is called exactly as it already exists, with `.unwrap_or_default()` for the `Task`
  boundary (a `Vec::new()` fallback on a listing error just means no dots that refresh, not a
  crash — consistent with the design's total-function philosophy for `core`).
- **Placeholder scan:** no `todo!()`/TODO/"handle later" in any shipped code; every step shows the
  exact code to write, not a description of it.
- **Deliberate divergences (noted in-plan):** (1) `OpenDate` on the already-active date is a no-op,
  diverging from the web's unconditional `goToDate` — same rationale and precedent as Phase 2's
  `SwitchTab` no-op-on-same-index. (2) Ctrl/Cmd-click-to-new-tab from the calendar and per-section
  chevron collapse are both explicitly deferred (see "Deferred on purpose") rather than silently
  dropped, each with a concrete reason and target phase.
- **Reachability:** `PrevMonth`/`NextMonth`/`OpenDate`/`ToggleSidebar`/`PaneResized` are all
  reachable now via mouse (calendar buttons, day cells, sidebar header button, pane-grid divider
  drag) — no dead code waiting on a later phase, unlike some of Phase 2's keyboard-only effects
  which waited on Phase 5's command mode.
