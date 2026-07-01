//! Editor tabs: an ordered, de-duplicated set of open dates with one active index.
//! Port of `web/src/lib/tabs.ts`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsState {
    /// Date strings (`YYYY-MM-DD`), de-duplicated.
    pub tabs: Vec<String>,
    pub active_index: usize,
}

pub fn init_tabs(today: &str) -> TabsState {
    TabsState {
        tabs: vec![today.to_string()],
        active_index: 0,
    }
}

/// The active tab's date.
pub fn active_date(state: &TabsState) -> &str {
    &state.tabs[state.active_index]
}

/// Retarget the active tab to `date` in place; if `date` is already open, focus that tab.
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

/// Open `date` in a new tab (appended right) and focus it; focus an existing tab if present.
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

/// Close the tab at `index`; guarantees >=1 tab by falling back to `today`.
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
