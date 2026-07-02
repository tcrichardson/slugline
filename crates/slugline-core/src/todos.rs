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
