use std::sync::LazyLock;

use regex::Regex;

use super::state::{Cursor, EditorState, clamp_cursor, push_undo};

static TASK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\s*- \[)([ xX])(\] )").unwrap());

pub fn delete_char(s: &EditorState) -> EditorState {
    let text = s.lines.get(s.cursor.line).cloned().unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return s.clone();
    }
    let mut ns = push_undo(s);
    let col = s.cursor.col.min(chars.len());
    let mut new_line: String = chars[..col].iter().collect();
    if col < chars.len() {
        new_line.extend(chars[col + 1..].iter());
    }
    ns.lines[s.cursor.line] = new_line;
    clamp_cursor(&ns)
}

pub fn delete_line(s: &EditorState) -> EditorState {
    let mut ns = push_undo(s);
    let removed = ns.lines.remove(s.cursor.line);
    if ns.lines.is_empty() {
        ns.lines.push(String::new());
    }
    ns.register = vec![removed];
    let line = s.cursor.line.min(ns.lines.len() - 1);
    ns.cursor = Cursor { line, col: 0 };
    clamp_cursor(&ns)
}

pub fn yank_line(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    ns.register = vec![s.lines.get(s.cursor.line).cloned().unwrap_or_default()];
    ns.message = "1 line yanked".into();
    ns
}

pub fn paste_below(s: &EditorState) -> EditorState {
    if s.register.is_empty() {
        return s.clone();
    }
    let mut ns = push_undo(s);
    let at = s.cursor.line + 1;
    for (k, line) in s.register.iter().enumerate() {
        ns.lines.insert(at + k, line.clone());
    }
    ns.cursor = Cursor { line: at, col: 0 };
    clamp_cursor(&ns)
}

pub fn paste_above(s: &EditorState) -> EditorState {
    if s.register.is_empty() {
        return s.clone();
    }
    let mut ns = push_undo(s);
    let at = s.cursor.line;
    for (k, line) in s.register.iter().enumerate() {
        ns.lines.insert(at + k, line.clone());
    }
    ns.cursor = Cursor { line: at, col: 0 };
    clamp_cursor(&ns)
}

pub fn toggle_todo(s: &EditorState) -> EditorState {
    let text = s.lines.get(s.cursor.line).cloned().unwrap_or_default();
    let Some(caps) = TASK_RE.captures(&text) else {
        return s.clone();
    };
    let mut ns = push_undo(s);
    let next = if &caps[2] == " " { "x" } else { " " };
    let replaced = TASK_RE.replace(&text, format!("${{1}}{next}${{3}}").as_str());
    ns.lines[s.cursor.line] = replaced.into_owned();
    ns
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::state::create_editor_state;

    fn at(lines: &[&str], line: usize, col: usize) -> EditorState {
        let mut s = create_editor_state(lines.iter().map(|s| s.to_string()).collect(), vec![]);
        s.cursor = Cursor { line, col };
        s
    }

    #[test]
    fn delete_char_removes_under_cursor_and_pushes_undo() {
        let s = at(&["abc"], 0, 1);
        let r = delete_char(&s);
        assert_eq!(r.lines, vec!["ac".to_string()]);
        assert_eq!(r.undo.len(), 1);
    }

    #[test]
    fn dd_yanks_and_leaves_at_least_one_line() {
        let s = at(&["only"], 0, 0);
        let r = delete_line(&s);
        assert_eq!(r.lines, vec![String::new()]);
        assert_eq!(r.register, vec!["only".to_string()]);
    }

    #[test]
    fn paste_below_inserts_register() {
        let mut s = at(&["a", "b"], 0, 0);
        s.register = vec!["X".into()];
        let r = paste_below(&s);
        assert_eq!(
            r.lines,
            vec!["a".to_string(), "X".to_string(), "b".to_string()]
        );
        assert_eq!(r.cursor, Cursor { line: 1, col: 0 });
    }

    #[test]
    fn toggle_todo_flips_checkbox() {
        let s = at(&["- [ ] task"], 0, 0);
        let r = toggle_todo(&s);
        assert_eq!(r.lines, vec!["- [x] task".to_string()]);
        assert_eq!(toggle_todo(&r).lines, vec!["- [ ] task".to_string()]);
    }

    // Ported from web/src/lib/editor/edits.test.ts

    #[test]
    fn delete_char_on_empty_line_is_noop() {
        let s = at(&[""], 0, 0);
        let r = delete_char(&s);
        assert_eq!(r.lines, vec!["".to_string()]);
    }

    #[test]
    fn dd_on_last_line_clamps_cursor_up() {
        let s = at(&["a", "b"], 0, 0);
        let r = delete_line(&s);
        assert_eq!(r.lines, vec!["b".to_string()]);
        assert_eq!(r.cursor, Cursor { line: 0, col: 0 });
    }

    #[test]
    fn yy_yanks_and_sets_message() {
        let s = at(&["a", "b"], 0, 0);
        let r = yank_line(&s);
        assert_eq!(r.register, vec!["a".to_string()]);
        assert_eq!(r.message, "1 line yanked");
    }

    #[test]
    fn yy_then_p_pastes_below() {
        let s = at(&["a", "b"], 0, 0);
        let mut r = yank_line(&s);
        r.cursor = Cursor { line: 1, col: 0 };
        let r = paste_below(&r);
        assert_eq!(
            r.lines,
            vec!["a".to_string(), "b".to_string(), "a".to_string()]
        );
    }

    #[test]
    fn p_pastes_above() {
        let s = at(&["a", "b"], 1, 0);
        let mut r = yank_line(&s);
        r.cursor = Cursor { line: 0, col: 0 };
        let r = paste_above(&r);
        assert_eq!(
            r.lines,
            vec!["b".to_string(), "a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn toggle_todo_noop_on_non_task_lines() {
        let s = at(&["plain"], 0, 0);
        let r = toggle_todo(&s);
        assert_eq!(r.lines, vec!["plain".to_string()]);
    }

    #[test]
    fn toggle_todo_preserves_indentation() {
        let s = at(&["  - [ ] task"], 0, 0);
        let r = toggle_todo(&s);
        assert_eq!(r.lines, vec!["  - [x] task".to_string()]);
    }
}
