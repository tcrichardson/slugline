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

    /// Refresh the calendar's has-note dots from disk.
    fn list_notes_task(&self) -> Task<Message> {
        let store = self.store.clone();
        Task::perform(
            async move { store.list_dates().unwrap_or_default() },
            Message::NotesListed,
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
}
