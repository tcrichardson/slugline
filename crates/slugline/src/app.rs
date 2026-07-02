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
    /// A calendar day cell was clicked: retarget the active tab to that date.
    OpenDate(String),
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
    /// The `update_theme` persistence write for a `:theme` switch finished. `target` is
    /// the theme that was applied optimistically; `prev` is what to roll back to on
    /// failure.
    ThemePersisted {
        target: String,
        prev: String,
        res: Result<(), String>,
    },
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

    /// The initial `Task` to run once the window opens (called from `main`).
    pub fn boot(&self) -> Task<Message> {
        self.list_notes_task()
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

    /// Refresh the calendar's has-note dots from disk.
    fn list_notes_task(&self) -> Task<Message> {
        let store = self.store.clone();
        Task::perform(
            async move { store.list_dates().unwrap_or_default() },
            Message::NotesListed,
        )
    }

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
                let ctx = CommandCtx {
                    now_hhmm: now_hhmm(),
                };
                let result = handle_key(&self.editor, &input, &ctx);
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
                            return self.refresh_todos_task();
                        }
                    }
                    Err(e) => {
                        self.set_error(format!("Save failed \u{2014} edits kept, will retry: {e}"));
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
                        self.set_error(format!("Failed to load note {date}: {e}"));
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
            Message::OpenPalette => {
                self.editor.command = Some(String::new());
                self.editor.message = String::new();
                Task::none()
            }
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
                        &self.palette,
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

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::on_key_press(|key, mods| {
                let k = key_string(&key, &mods)?;
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
        let config_path = dir.path().join("config.toml");
        let app = App::new(store, date.to_string(), UiConfig::default(), config_path);
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

    #[test]
    fn switch_tab_to_same_index_is_a_noop() {
        let (_dir, mut app) = temp_app("2026-06-23");
        let before = app.tabs.clone();
        let _ = app.update(Message::SwitchTab(0));
        assert_eq!(app.tabs, before);
        assert!(!app.loading); // no navigation was kicked off
    }

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
}
