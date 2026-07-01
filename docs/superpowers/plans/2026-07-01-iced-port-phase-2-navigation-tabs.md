# Phase 2 — Navigation & Tabs — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **This is a port.** Behavioral truth for the pure pieces is `web/src/lib/tabs.ts` (+ `tabs.test.ts`),
> `web/src/lib/dates.ts` (+ `dates.test.ts`), and the effect/navigation logic in
> `web/src/lib/appState.svelte.ts` (`runEffect`, `goToDate`, `loadActive`) and
> `web/src/lib/editor/keymap.ts`. Each task gives the full Rust plus ported tests.
>
> **Iced API caution:** method names target iced `0.13.x`. Where flagged **[verify]**, confirm the
> exact signature in the pinned version and adjust — the *intent* is the contract.

**Goal:** Make the editor navigate between days and tabs. NORMAL-mode keys `[` / `]` / `Ctrl-t` and `gt` / `gT` emit navigation `AppEffect`s; the Iced app translates every `AppEffect` into a `Task` that flushes the current buffer, retargets the tab set, and loads the new note. The window title and a minimal tab strip reflect the active date, and the yank register is shared across navigation.

**Architecture:** Two new pure `slugline-core` modules (`tabs`, and `add_days` added to `dates`), a small keymap extension that emits effects, and the `AppEffect → Task` wiring in the Iced `app` — a direct translation of the web's `runEffect`. Flush-before-navigate is modeled as one composed async `Task` so navigation always observes a fully-flushed buffer (`update` cannot `await`).

**Tech Stack:** Rust, `chrono` (already a `slugline-core` dep) for calendar-day math; Iced `0.13.x` (`Task::perform`, `button`, `row`, `container`).

---

## Prerequisites

- **Phases 0, 1a, 1b, 1c are complete and committed on `iced-port`, and `cargo test --workspace` is green.** Phase 2 builds directly on the Phase 1c `App` (`crates/slugline/src/app.rs`), `core::editor` (`AppEffect`/`KeyInput`/`KeyResult`/`handle_key`), `core::store::NotesStore`, and `core::dates::today_iso`. If Phase 1c is still uncommitted working changes, commit it first (its own plan's Task 5 gate) before starting here.

## Scope

**In this phase (reachable by keyboard now):**
- `[` → `PrevDay`, `]` → `NextDay`, `Ctrl-t` → `Today` (day navigation, retargets the active tab in place).
- `gt` → `TabNext`, `gT` → `TabPrev` (cycle tabs).
- The full `AppEffect → Task` machinery for **all** navigation variants (`Goto`, `Today`, `Tab`, `Close`, `PrevDay`, `NextDay`, `TabNext`, `TabPrev`) plus `Save`, wired once here so later phases only need to *emit* them.
- Shared yank register across navigation; flush-before-navigate; window title + a minimal tab strip reflect the active date; mouse `SwitchTab` / `CloseTab`.

**Deferred on purpose:**
- `:` command mode + palette (which is what makes `:goto` / `:tab` / `:close` / `:w` reachable) → **Phase 5**. Their effect *handlers* are built here; only the `:` entry point is missing until Phase 5. Because of this, **creating a second tab from the keyboard is not possible until Phase 5** — Phase 2 exercises multi-tab logic via unit tests and mouse clicks.
- `Theme` effect handling and toast/error surface → **Phase 6** (`run_effect` returns `Task::none()` for `Theme` this phase).
- Calendar month reset on navigate (`yearMonth`), calendar/agenda/todos refresh → **Phases 3–4**.

---

## File Structure (files added/changed in Phase 2)

```
crates/slugline-core/
  src/
    lib.rs                         # + pub mod tabs;
    dates.rs                       # + add_days()
    tabs.rs                        # NEW: port of web/src/lib/tabs.ts
    editor/keymap.rs               # + emit PrevDay/NextDay/Today/TabNext/TabPrev effects

crates/slugline/
  Cargo.toml                       # + tempfile dev-dependency (for App reducer tests)
  src/
    app.rs                         # REWRITE: tabs in Model, AppEffect→Task, flush-before-navigate
    ui/mod.rs                      # + pub mod tab_strip;
    ui/tab_strip.rs                # NEW: minimal clickable tab strip
```

---

### Task 1: `add_days` in `core::dates`

**Files:**
- Modify: `crates/slugline-core/src/dates.rs`

- [ ] **Step 1: Write the failing test** — append to the `tests` module in `crates/slugline-core/src/dates.rs`:

```rust
    #[test]
    fn adds_days_across_month_and_year_boundaries() {
        assert_eq!(add_days("2026-12-31", 1), "2027-01-01");
        assert_eq!(add_days("2026-03-01", -1), "2026-02-28");
    }

    #[test]
    fn add_days_zero_is_identity_and_bad_input_is_unchanged() {
        assert_eq!(add_days("2026-06-23", 0), "2026-06-23");
        assert_eq!(add_days("not-a-date", 5), "not-a-date");
    }
```

- [ ] **Step 2: Run test to verify it fails** — `cargo test -p slugline-core dates::`
Expected: FAIL to compile (`add_days` undefined).

- [ ] **Step 3: Implement `add_days`** — add to the top of `crates/slugline-core/src/dates.rs`, and extend the import:

```rust
use chrono::{Days, Local, NaiveDate};

/// Today's date in the local timezone, formatted `YYYY-MM-DD`.
pub fn today_iso() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// Add `n` days (may be negative) to an ISO `YYYY-MM-DD` date, returning a new ISO date.
/// On any parse/overflow failure the input is returned unchanged (callers always pass
/// validated dates, so this is a safety net rather than an expected path).
pub fn add_days(date: &str, n: i64) -> String {
    let Ok(base) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
        return date.to_string();
    };
    let shifted = if n >= 0 {
        base.checked_add_days(Days::new(n as u64))
    } else {
        base.checked_sub_days(Days::new(n.unsigned_abs()))
    };
    shifted.map_or_else(|| date.to_string(), |d| d.format("%Y-%m-%d").to_string())
}
```

(Replace the existing `use chrono::Local;` line with the `use chrono::{Days, Local, NaiveDate};` line above, and leave the existing `today_iso` as-is — it is shown here only for the merged import context.)

- [ ] **Step 4: Run test to verify it passes** — `cargo test -p slugline-core dates::`
Expected: PASS (existing `today_iso_is_a_valid_yyyy_mm_dd` + the 2 new tests).

- [ ] **Step 5: Commit**

```bash
git add crates/slugline-core/src/dates.rs
git commit -m "feat(core): add add_days() for day navigation"
```

---

### Task 2: Port `tabs` into `core`

**Files:**
- Create: `crates/slugline-core/src/tabs.rs`
- Modify: `crates/slugline-core/src/lib.rs` (add `pub mod tabs;`)

- [ ] **Step 1: Declare the module** — add `pub mod tabs;` to `crates/slugline-core/src/lib.rs` (keep the list alphabetical):

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

- [ ] **Step 2: Write the failing test** — create `crates/slugline-core/src/tabs.rs` with the type + stubs + full ported tests:

```rust
//! Editor tabs: an ordered, de-duplicated set of open dates with one active index.
//! Port of `web/src/lib/tabs.ts`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsState {
    /// Date strings (`YYYY-MM-DD`), de-duplicated.
    pub tabs: Vec<String>,
    pub active_index: usize,
}

pub fn init_tabs(today: &str) -> TabsState {
    todo!()
}

/// The active tab's date.
pub fn active_date(state: &TabsState) -> &str {
    todo!()
}

/// Retarget the active tab to `date` in place; if `date` is already open, focus that tab.
pub fn retarget(state: &TabsState, date: &str) -> TabsState {
    todo!()
}

/// Open `date` in a new tab (appended right) and focus it; focus an existing tab if present.
pub fn open_new_tab(state: &TabsState, date: &str) -> TabsState {
    todo!()
}

/// Close the tab at `index`; guarantees >=1 tab by falling back to `today`.
pub fn close_tab(state: &TabsState, index: usize, today: &str) -> TabsState {
    todo!()
}

pub fn next_tab(state: &TabsState) -> TabsState {
    todo!()
}

pub fn prev_tab(state: &TabsState) -> TabsState {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_with_today_as_only_tab() {
        let s = init_tabs("2026-06-23");
        assert_eq!(s.tabs, vec!["2026-06-23".to_string()]);
        assert_eq!(active_date(&s), "2026-06-23");
    }

    #[test]
    fn retargets_the_active_tab_in_place() {
        let s = retarget(&init_tabs("2026-06-23"), "2026-06-22");
        assert_eq!(s.tabs, vec!["2026-06-22".to_string()]);
        assert_eq!(s.active_index, 0);
    }

    #[test]
    fn focuses_existing_tab_instead_of_duplicating_on_retarget() {
        let mut s = open_new_tab(&init_tabs("2026-06-23"), "2026-06-22"); // [23, 22] active 1
        s = retarget(&s, "2026-06-23"); // 23 already open at index 0
        assert_eq!(
            s.tabs,
            vec!["2026-06-23".to_string(), "2026-06-22".to_string()]
        );
        assert_eq!(s.active_index, 0);
    }

    #[test]
    fn opens_new_tabs_appended_right_and_focuses_them() {
        let s = open_new_tab(&init_tabs("2026-06-23"), "2026-06-24");
        assert_eq!(
            s.tabs,
            vec!["2026-06-23".to_string(), "2026-06-24".to_string()]
        );
        assert_eq!(s.active_index, 1);
    }

    #[test]
    fn closes_a_tab_and_always_keeps_at_least_one() {
        let mut s = open_new_tab(&init_tabs("2026-06-23"), "2026-06-24"); // [23, 24] active 1
        s = close_tab(&s, 1, "2026-06-25");
        assert_eq!(s.tabs, vec!["2026-06-23".to_string()]);
        assert_eq!(s.active_index, 0);
        s = close_tab(&s, 0, "2026-06-25"); // removing the last falls back to today
        assert_eq!(s.tabs, vec!["2026-06-25".to_string()]);
        assert_eq!(s.active_index, 0);
    }

    #[test]
    fn cycles_tabs_with_wrap_around() {
        let mut s = open_new_tab(&init_tabs("a"), "b"); // [a, b] active 1
        s = next_tab(&s);
        assert_eq!(s.active_index, 0);
        s = prev_tab(&s);
        assert_eq!(s.active_index, 1);
    }
}
```

- [ ] **Step 3: Run test to verify it fails** — `cargo test -p slugline-core tabs::`
Expected: FAIL (`todo!()` panics).

- [ ] **Step 4: Implement the functions** (replace each `todo!()`):

```rust
pub fn init_tabs(today: &str) -> TabsState {
    TabsState {
        tabs: vec![today.to_string()],
        active_index: 0,
    }
}

pub fn active_date(state: &TabsState) -> &str {
    &state.tabs[state.active_index]
}

pub fn retarget(state: &TabsState, date: &str) -> TabsState {
    if let Some(existing) = state.tabs.iter().position(|d| d == date) {
        return TabsState {
            tabs: state.tabs.clone(),
            active_index: existing,
        };
    }
    let mut tabs = state.tabs.clone();
    tabs[state.active_index] = date.to_string();
    TabsState {
        tabs,
        active_index: state.active_index,
    }
}

pub fn open_new_tab(state: &TabsState, date: &str) -> TabsState {
    if let Some(existing) = state.tabs.iter().position(|d| d == date) {
        return TabsState {
            tabs: state.tabs.clone(),
            active_index: existing,
        };
    }
    let mut tabs = state.tabs.clone();
    tabs.push(date.to_string());
    let active_index = tabs.len() - 1;
    TabsState { tabs, active_index }
}

pub fn close_tab(state: &TabsState, index: usize, today: &str) -> TabsState {
    if index >= state.tabs.len() {
        return state.clone();
    }
    let mut tabs = state.tabs.clone();
    tabs.remove(index);
    if tabs.is_empty() {
        return TabsState {
            tabs: vec![today.to_string()],
            active_index: 0,
        };
    }
    let mut active_index = state.active_index;
    if index < active_index {
        active_index -= 1;
    } else if index == active_index {
        active_index = active_index.min(tabs.len() - 1);
    }
    TabsState { tabs, active_index }
}

pub fn next_tab(state: &TabsState) -> TabsState {
    TabsState {
        tabs: state.tabs.clone(),
        active_index: (state.active_index + 1) % state.tabs.len(),
    }
}

pub fn prev_tab(state: &TabsState) -> TabsState {
    let n = state.tabs.len();
    TabsState {
        tabs: state.tabs.clone(),
        active_index: (state.active_index + n - 1) % n,
    }
}
```

Note: the TS `closeTab` also guards `index < 0`; that case is impossible with `usize`, so only `index >= len` is checked. `prev_tab` uses `(active_index + n - 1) % n` to avoid `usize` underflow (the TS `(i - 1 + n) % n`).

- [ ] **Step 5: Run test to verify it passes** — `cargo test -p slugline-core tabs::`
Expected: PASS (6 tests — a 1:1 port of `tabs.test.ts`).

- [ ] **Step 6: Commit**

```bash
git add crates/slugline-core/src/tabs.rs crates/slugline-core/src/lib.rs
git commit -m "feat(core): port tabs state (init/retarget/open/close/cycle)"
```

---

### Task 3: Emit navigation effects from the keymap

**Files:**
- Modify: `crates/slugline-core/src/editor/keymap.rs`

- [ ] **Step 1: Add the failing tests** — replace the four `// TODO(phase 2/5): ...` comment lines near the end of the `tests` module (currently around lines 223–225) with these tests (leave the command-mode `:` TODO for Phase 5):

```rust
    // Ported from web/src/lib/editor/keymap.test.ts — navigation effects.

    #[test]
    fn gt_emits_tab_next_effect() {
        let s = create_editor_state(vec!["a".into()], vec![]);
        let s = handle_key(&s, &key("g")).state;
        assert_eq!(handle_key(&s, &key("t")).effect, Some(AppEffect::TabNext));
    }

    #[test]
    fn shift_gt_emits_tab_prev_effect() {
        let s = create_editor_state(vec!["a".into()], vec![]);
        let s = handle_key(&s, &key("g")).state;
        assert_eq!(handle_key(&s, &key("T")).effect, Some(AppEffect::TabPrev));
    }

    #[test]
    fn bracket_keys_emit_day_navigation() {
        let s = create_editor_state(vec!["a".into()], vec![]);
        assert_eq!(handle_key(&s, &key("[")).effect, Some(AppEffect::PrevDay));
        assert_eq!(handle_key(&s, &key("]")).effect, Some(AppEffect::NextDay));
    }

    #[test]
    fn ctrl_t_emits_today_effect() {
        let s = create_editor_state(vec!["a".into()], vec![]);
        let r = handle_key(
            &s,
            &KeyInput {
                key: "t".into(),
                ctrl: true,
                meta: false,
                shift: false,
            },
        );
        assert_eq!(r.effect, Some(AppEffect::Today));
    }

    // TODO(phase 5): command mode (`:`) tests — deferred to Phase 5
```

- [ ] **Step 2: Run tests to verify they fail** — `cargo test -p slugline-core editor::keymap`
Expected: FAIL — the four new tests get `None` (effects not emitted yet).

- [ ] **Step 3: Add the effect helper** — in `crates/slugline-core/src/editor/keymap.rs`, just below the existing `state_only` fn:

```rust
fn state_effect(state: EditorState, effect: AppEffect) -> KeyResult {
    KeyResult {
        state,
        effect: Some(effect),
    }
}
```

- [ ] **Step 4: Emit `gt` / `gT`** — in `handle_normal`, extend the `Pending::G` arm. Replace:

```rust
        Pending::G => {
            let s2 = with_pending(s, Pending::None);
            return match k.key.as_str() {
                "g" => state_only(motions::first_line(&s2)),
                // "t"/"T" (gt/gT) emit TabNext/TabPrev — deferred to Phase 2.
                _ => handle_normal(&s2, k),
            };
        }
```

with:

```rust
        Pending::G => {
            let s2 = with_pending(s, Pending::None);
            return match k.key.as_str() {
                "g" => state_only(motions::first_line(&s2)),
                "t" => state_effect(s2, AppEffect::TabNext),
                "T" => state_effect(s2, AppEffect::TabPrev),
                _ => handle_normal(&s2, k),
            };
        }
```

- [ ] **Step 5: Emit `[` / `]` / `Ctrl-t`** — still in `handle_normal`, insert the following block immediately **before** the `let st = match k.key.as_str() {` line (i.e., after the `Pending` match and before the main key match):

```rust
    // Navigation effects (Phase 2): reachable in NORMAL mode only.
    match k.key.as_str() {
        "[" => return state_effect(s.clone(), AppEffect::PrevDay),
        "]" => return state_effect(s.clone(), AppEffect::NextDay),
        _ => {}
    }
    if k.ctrl && (k.key == "t" || k.key == "T") {
        return state_effect(s.clone(), AppEffect::Today);
    }

```

Then update the stale comment inside the main match — replace:

```rust
        // ":" / "[" / "]" / Ctrl-t are deferred (Phase 2/5).
```

with:

```rust
        // ":" command mode is deferred to Phase 5.
```

- [ ] **Step 6: Run tests to verify they pass** — `cargo test -p slugline-core editor::keymap`
Expected: PASS (all prior editing tests + the 4 new navigation-effect tests). `gt`/`gT` still require the two-keystroke `g` prefix; `[`/`]`/`Ctrl-t` only fire in NORMAL mode (in INSERT, `[`/`]` insert literals and `Ctrl-t` is ignored), matching `keymap.ts`.

- [ ] **Step 7: Commit**

```bash
git add crates/slugline-core/src/editor/keymap.rs
git commit -m "feat(core): emit navigation effects ([ ] gt gT Ctrl-t)"
```

---

### Task 4: Wire `AppEffect → Task` navigation into the app

**Files:**
- Modify: `crates/slugline/Cargo.toml` (add `tempfile` dev-dependency)
- Rewrite: `crates/slugline/src/app.rs`

- [ ] **Step 1: Add the test dependency** — in `crates/slugline/Cargo.toml`, under `[dev-dependencies]`, add:

```toml
tempfile = "3.27.0"
```

so the section reads:

```toml
[dev-dependencies]
dirs = "6.0.0"
smol_str = "0.2"
tempfile = "3.27.0"
```

- [ ] **Step 2: Rewrite `crates/slugline/src/app.rs`** — replace the entire file with the following. This adds `tabs`/`shared_register`/`loading` to the Model, routes `handle_key`'s effect through `run_effect`, and models flush-before-navigate as one composed `Task`. The pure `plan_tabs` helper is unit-tested; `view`/navigation IO are not (rendering + async side effects, per the design's testing boundary).

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

#[derive(Debug, Clone)]
pub enum Message {
    Key(KeyInput),
    Tick,
    /// A debounced save finished. `date` is the note it targeted, so late saves for a
    /// since-navigated-away date don't corrupt the current buffer's bookkeeping.
    Saved {
        date: String,
        res: Result<String, String>,
    },
    /// A flush-then-load finished: commit the new tab set + loaded body atomically.
    Navigated {
        tabs: TabsState,
        body: Result<String, String>,
    },
    SwitchTab(usize),
    CloseTab(usize),
    CloseRequested(window::Id),
}

/// Pure: compute the new tab set for a navigation effect. `active`/`today` are injected so
/// this is deterministic and unit-testable. Returns `None` for non-navigation effects
/// (`Save`, `Theme`), which the app handles separately.
fn plan_tabs(tabs: &TabsState, active: &str, today: &str, effect: &AppEffect) -> Option<TabsState> {
    match effect {
        AppEffect::Goto(date) => Some(retarget(tabs, date)),
        AppEffect::Today => Some(retarget(tabs, today)),
        AppEffect::PrevDay => Some(retarget(tabs, &add_days(active, -1))),
        AppEffect::NextDay => Some(retarget(tabs, &add_days(active, 1))),
        AppEffect::Tab(date) => Some(open_new_tab(tabs, date)),
        AppEffect::Close => Some(close_tab(tabs, tabs.active_index, today)),
        AppEffect::TabNext => Some(next_tab(tabs)),
        AppEffect::TabPrev => Some(prev_tab(tabs)),
        AppEffect::Save | AppEffect::Theme(_) => None,
    }
}

impl App {
    pub fn new(store: NotesStore, date: String) -> Self {
        let (content, error) = match store.read_or_create(&date) {
            Ok(c) => (c, None),
            Err(e) => (String::new(), Some(format!("Failed to load note: {e}"))),
        };
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

    fn active_date(&self) -> String {
        active_date(&self.tabs).to_string()
    }

    pub fn title(&self) -> String {
        format!("Slugline \u{2014} {}", self.active_date())
    }

    /// The buffer as file content, with a guaranteed trailing newline.
    fn content(&self) -> String {
        let body = self.editor.lines.join("\n");
        if body.ends_with('\n') {
            body
        } else {
            format!("{body}\n")
        }
    }

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

    /// Flush the current buffer (if dirty) to its date, then load `new_tabs`' active date.
    /// One composed `Task` so navigation observes a fully-flushed buffer, mirroring the web's
    /// `await flush(); retarget(); loadActive()`.
    fn navigate(&mut self, new_tabs: TabsState) -> Task<Message> {
        let old_date = self.active_date();
        let old_content = self.content();
        let dirty = old_content != self.last_saved;
        let new_date = active_date(&new_tabs).to_string();
        let store = self.store.clone();
        self.loading = true;
        Task::perform(
            async move {
                if dirty {
                    // Best-effort flush; matches the web, which continues even if the write fails.
                    let _ = store.write(&old_date, &old_content);
                }
                let body = store.read_or_create(&new_date).map_err(|e| e.to_string());
                (new_tabs, body)
            },
            |(tabs, body)| Message::Navigated { tabs, body },
        )
    }

    /// Spawn an atomic save of the current buffer to the active date.
    fn spawn_save(&mut self) -> Task<Message> {
        if self.saving {
            return Task::none();
        }
        let content = self.content();
        if content == self.last_saved {
            self.dirty_since = None;
            return Task::none();
        }
        self.saving = true;
        let store = self.store.clone();
        let date = self.active_date();
        let to_save = content;
        Task::perform(
            async move {
                let res = store
                    .write(&date, &to_save)
                    .map(|_| to_save)
                    .map_err(|e| e.to_string());
                (date, res)
            },
            |(date, res)| Message::Saved { date, res },
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Key(input) => {
                let before = self.editor.lines.clone();
                let result = handle_key(&self.editor, &input);
                self.editor = result.state;
                self.shared_register = self.editor.register.clone();
                if self.editor.lines != before {
                    self.dirty_since = Some(Instant::now());
                }
                match result.effect {
                    Some(effect) => self.run_effect(effect),
                    None => Task::none(),
                }
            }
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
                    }
                    Err(e) => {
                        let date = self.active_date();
                        self.error = Some(format!("Failed to load note {date}: {e}"));
                    }
                }
                Task::none()
            }
            Message::SwitchTab(index) => {
                if index >= self.tabs.tabs.len() || index == self.tabs.active_index {
                    return Task::none();
                }
                let new_tabs = TabsState {
                    tabs: self.tabs.tabs.clone(),
                    active_index: index,
                };
                self.navigate(new_tabs)
            }
            Message::CloseTab(index) => {
                let new_tabs = close_tab(&self.tabs, index, &today_iso());
                self.navigate(new_tabs)
            }
            Message::CloseRequested(id) => {
                // Final synchronous flush so nothing is lost on quit.
                let content = self.content();
                if content != self.last_saved {
                    let _ = self.store.write(&self.active_date(), &content);
                }
                window::close(id)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![tab_strip::view(&self.tabs), editor_pane::view(&self.editor)].into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
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
            time::every(Duration::from_millis(250)).map(|_| Message::Tick),
            window::close_requests().map(Message::CloseRequested),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slugline_core::tabs::init_tabs;

    fn temp_app(date: &str) -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        let store = NotesStore::new(dir.path().to_path_buf());
        let app = App::new(store, date.to_string());
        (dir, app)
    }

    #[test]
    fn plan_tabs_prev_and_next_day_retarget_active() {
        let tabs = init_tabs("2026-06-23");
        let prev = plan_tabs(&tabs, "2026-06-23", "2026-06-23", &AppEffect::PrevDay).unwrap();
        assert_eq!(prev.tabs, vec!["2026-06-22".to_string()]);
        let next = plan_tabs(&tabs, "2026-06-23", "2026-06-23", &AppEffect::NextDay).unwrap();
        assert_eq!(next.tabs, vec!["2026-06-24".to_string()]);
    }

    #[test]
    fn plan_tabs_today_retargets_to_today() {
        let tabs = init_tabs("2026-06-20");
        let r = plan_tabs(&tabs, "2026-06-20", "2026-06-23", &AppEffect::Today).unwrap();
        assert_eq!(r.tabs, vec!["2026-06-23".to_string()]);
        assert_eq!(r.active_index, 0);
    }

    #[test]
    fn plan_tabs_tab_opens_new_then_tabprev_cycles() {
        let tabs = init_tabs("2026-06-23");
        let opened = plan_tabs(
            &tabs,
            "2026-06-23",
            "2026-06-23",
            &AppEffect::Tab("2026-06-24".into()),
        )
        .unwrap();
        assert_eq!(
            opened.tabs,
            vec!["2026-06-23".to_string(), "2026-06-24".to_string()]
        );
        assert_eq!(opened.active_index, 1);
        let prev = plan_tabs(&opened, "2026-06-24", "2026-06-23", &AppEffect::TabPrev).unwrap();
        assert_eq!(prev.active_index, 0);
    }

    #[test]
    fn plan_tabs_close_falls_back_to_today() {
        let tabs = init_tabs("2026-06-23");
        let r = plan_tabs(&tabs, "2026-06-23", "2026-06-25", &AppEffect::Close).unwrap();
        assert_eq!(r.tabs, vec!["2026-06-25".to_string()]);
    }

    #[test]
    fn plan_tabs_returns_none_for_non_navigation() {
        let tabs = init_tabs("2026-06-23");
        assert!(plan_tabs(&tabs, "2026-06-23", "2026-06-23", &AppEffect::Save).is_none());
        assert!(
            plan_tabs(
                &tabs,
                "2026-06-23",
                "2026-06-23",
                &AppEffect::Theme("dark".into())
            )
            .is_none()
        );
    }

    #[test]
    fn navigated_swaps_tabs_editor_and_carries_register() {
        let (_dir, mut app) = temp_app("2026-06-23");
        app.shared_register = vec!["yanked".to_string()];
        let tabs = init_tabs("2026-06-24");
        let _ = app.update(Message::Navigated {
            tabs: tabs.clone(),
            body: Ok("# hello\n".to_string()),
        });
        assert_eq!(app.tabs, tabs);
        assert_eq!(app.active_date(), "2026-06-24");
        assert_eq!(app.editor.lines, vec!["# hello".to_string()]);
        assert_eq!(app.last_saved, "# hello\n");
        assert_eq!(app.editor.register, vec!["yanked".to_string()]);
        assert!(!app.loading);
    }

    #[test]
    fn switch_tab_to_same_index_is_a_noop() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let before = app.tabs.clone();
        let _ = app.update(Message::SwitchTab(0));
        assert_eq!(app.tabs, before);
        assert!(!app.loading); // no navigation was kicked off
    }
}
```

- [ ] **Step 3: Build** — `cargo build -p slugline`
Expected: fails only because `crate::ui::tab_strip` does not exist yet (added in Task 5). All other code compiles. If any Iced call name differs in the pinned version (`Task::perform`, `window::close`, `keyboard::on_key_press`, `mods.logo()`), fix per **[verify]** — the structure is the contract.

- [ ] **Step 4: Run the app reducer tests** — `cargo test -p slugline app::`
Expected: the 7 `app::tests` compile and PASS (they don't touch `tab_strip`). If `cargo test` won't build because `view` references the missing `tab_strip`, proceed to Task 5 first, then run this. Order the commit after Task 5 so the crate builds.

---

### Task 5: Minimal tab strip UI

**Files:**
- Create: `crates/slugline/src/ui/tab_strip.rs`
- Modify: `crates/slugline/src/ui/mod.rs` (add `pub mod tab_strip;`)

- [ ] **Step 1: Declare the module** — `crates/slugline/src/ui/mod.rs`:

```rust
pub mod editor_pane;
pub mod palette;
pub mod tab_strip;
```

- [ ] **Step 2: Implement the strip** — `crates/slugline/src/ui/tab_strip.rs`. A horizontal row of tab buttons; the active tab is marked with `\u{25b8}` (▸). Clicking a tab emits `SwitchTab`; the `\u{00d7}` (×) button emits `CloseTab`. There is no unit test here (rendering side effects); it is exercised by the Task 6 manual smoke.

```rust
use iced::widget::{button, container, row, text};
use iced::{Element, Length};

use slugline_core::tabs::TabsState;

use crate::app::Message;

/// A simple horizontal strip of tab buttons reflecting the open dates and the active one.
/// Styling is intentionally default (theming/polish lands in Phase 6).
pub fn view(tabs: &TabsState) -> Element<'_, Message> {
    let mut strip = row![].spacing(6).padding([6, 8]).width(Length::Fill);
    for (i, date) in tabs.tabs.iter().enumerate() {
        let marker = if i == tabs.active_index {
            "\u{25b8} "
        } else {
            ""
        };
        let label = button(text(format!("{marker}{date}")).size(13))
            .on_press(Message::SwitchTab(i))
            .padding([4, 10]);
        let close = button(text("\u{00d7}").size(13))
            .on_press(Message::CloseTab(i))
            .padding([4, 8]);
        strip = strip.push(label).push(close);
    }
    container(strip).width(Length::Fill).into()
}
```

**[verify]**: `button`, `.on_press`, `.padding([..])`, `row!`, `container`, `text(..).size(..)` are iced `0.13.x` spellings already used in `editor_pane.rs`. If `button`'s builder differs in the pinned version, adjust; the contract is "one clickable label + close button per tab, active tab marked."

- [ ] **Step 3: Build the whole app** — `cargo build -p slugline`
Expected: compiles (the `view` from Task 4 now resolves `tab_strip::view`).

- [ ] **Step 4: Run the app tests** — `cargo test -p slugline`
Expected: PASS (cli, keys, and the 7 `app::tests`).

- [ ] **Step 5: Commit Tasks 4 + 5 together** (the crate only builds with both):

```bash
git add crates/slugline/Cargo.toml crates/slugline/src/app.rs crates/slugline/src/ui/
git commit -m "feat(app): navigation & tabs — AppEffect->Task, flush-before-navigate, tab strip"
```

---

### Task 6: Workspace hygiene gate + manual smoke

**Files:** none (verification only)

- [ ] **Step 1: Full workspace test** — `cargo test --workspace`
Expected: green (core: date/dates/store/config/doc/editor/**tabs**; app: cli/keys/**app**).

- [ ] **Step 2: Format + clippy** — `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean. Fix and re-run if needed.

- [ ] **Step 3: Manual smoke — day navigation, flush, shared register**

Run: `cargo run -p slugline -- --notes-dir ./dev-notes`
Verify all of:
1. Window opens titled `Slugline — <today>`; a tab strip at the top shows one tab (`▸ <today>`).
2. In NORMAL mode press `]` → title changes to `<today+1>`, that note materializes/loads, the tab strip shows `▸ <today+1>`. Press `[` twice → `<today-1>`. Press `Ctrl-t` → back to `<today>`.
3. Shared register across navigation: on today, put the cursor on a line and press `yy`, then `]`, then `p` → the yanked line is pasted into tomorrow's note (register survived navigation).
4. Flush-before-navigate: press `o`, type `flush check`, `Escape`, then **immediately** press `]` (well under ~750ms). Press `[` to return to today and confirm `flush check` is present (the edit was flushed before navigating). Also inspect `./dev-notes/<today>.md` on disk — it contains `flush check`.
5. Autosave still works: type more, wait ~1s, inspect the active date's file on disk — it reflects the edit.
6. Close the window; reopen — last edits are present (flush-on-exit still works).

(Multi-tab creation via `:tab` and switching between multiple tabs by mouse arrives with command mode in **Phase 5**; the tab strip renders a single tab until then.)

- [ ] **Step 4: Commit any fixups**

```bash
git add -A
git commit -m "chore: fmt + clippy clean for phase 2" || echo "nothing to commit"
```

---

## Self-Review (performed while writing this plan)

- **Spec coverage:** Implements design Section 2's `AppEffect → Task` translation (the "existing `AppEffect` variants each become a `Task`… a direct translation of `runEffect`") and Section 5's flush-before-navigate (the composed `Task::perform(async { flush_if_dirty(cur); read_note(next) }, NoteLoaded)`), the shared register (`create_editor_state(lines, shared_register)` on load, `shared_register = editor.register` after each key), and Section 4's window title + simple tab-strip `row` of buttons driving `SwitchTab`/`CloseTab`. Matches roadmap Phase 2: "`[ ]` `:goto` `:today`, `gt/gT` `:tab` `:close`, shared register, flush-before-navigate, `AppEffect`→`Task` wiring, window title reflects active date."
- **Type consistency (against the real committed code):** `TabsState { tabs: Vec<String>, active_index: usize }` and free fns `init_tabs/active_date/retarget/open_new_tab/close_tab/next_tab/prev_tab` are used identically in `app.rs`. `AppEffect` variants match `core::editor::keymap` exactly (`Goto/Today/Tab/Close/Save/PrevDay/NextDay/TabNext/TabPrev/Theme`). `NotesStore::{read_or_create, write}`, `create_editor_state(lines, register)`, `today_iso()`, `add_days(&str, i64)`, `handle_key(&EditorState, &KeyInput) -> KeyResult { state, effect }`, and `editor_pane::view` / `keys::key_string` / `keyboard::on_key_press` all match the current tree.
- **Placeholder scan:** the only `todo!()`s are intentional red-phase stubs in `tabs.rs`, replaced in Task 2. No TODO/"handle later" in shipped code (the sole remaining `// TODO(phase 5)` marks the deliberately-deferred `:` command-mode tests).
- **Reachability / no dead code:** every `AppEffect` variant is matched in `run_effect`/`plan_tabs`, so nothing is dead even though `Goto`/`Tab`/`Close`/`Save` are only *emitted* once command mode lands in Phase 5. `plan_tabs`, `Navigated`, and `SwitchTab` are covered by `app::tests`; `tabs`/`add_days`/keymap effects by ported core tests.
- **Deliberate divergences (noted in-plan):** (1) navigation flush is best-effort and swallows write errors to continue — this matches the web's `await flush()` (which catches internally); flagged as a shared latent behavior, not introduced here. (2) `Saved` now carries its target `date` to prevent a late save from corrupting a since-navigated buffer — a small correctness hardening over Phase 1c, required now that navigation exists. (3) `SwitchTab` is a no-op for the current/out-of-range index (the web trusts the caller); safer and observably identical.
