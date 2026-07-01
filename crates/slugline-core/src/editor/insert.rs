use super::state::{Cursor, EditorState, Mode, clamp_cursor, push_undo};

fn char_len(s: &str) -> usize {
    s.chars().count()
}
fn split_at_col(text: &str, col: usize) -> (String, String) {
    let chars: Vec<char> = text.chars().collect();
    let col = col.min(chars.len());
    (chars[..col].iter().collect(), chars[col..].iter().collect())
}

// Mode entry: each pushes ONE undo snapshot for the whole insert session.
pub fn enter_insert(s: &EditorState) -> EditorState {
    let mut ns = push_undo(s);
    ns.mode = Mode::Insert;
    ns
}
pub fn enter_insert_after(s: &EditorState) -> EditorState {
    let mut ns = push_undo(s);
    let len = char_len(
        ns.lines
            .get(ns.cursor.line)
            .map(String::as_str)
            .unwrap_or(""),
    );
    ns.mode = Mode::Insert;
    ns.cursor.col = (ns.cursor.col + if len > 0 { 1 } else { 0 }).min(len);
    ns
}
pub fn enter_insert_line_end(s: &EditorState) -> EditorState {
    let mut ns = push_undo(s);
    let len = char_len(
        ns.lines
            .get(ns.cursor.line)
            .map(String::as_str)
            .unwrap_or(""),
    );
    ns.mode = Mode::Insert;
    ns.cursor.col = len;
    ns
}
pub fn open_below(s: &EditorState) -> EditorState {
    let mut ns = push_undo(s);
    ns.lines.insert(ns.cursor.line + 1, String::new());
    ns.mode = Mode::Insert;
    ns.cursor = Cursor {
        line: ns.cursor.line + 1,
        col: 0,
    };
    ns
}
pub fn open_above(s: &EditorState) -> EditorState {
    let mut ns = push_undo(s);
    let line = ns.cursor.line;
    ns.lines.insert(line, String::new());
    ns.mode = Mode::Insert;
    ns.cursor = Cursor { line, col: 0 };
    ns
}
pub fn exit_insert(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    ns.mode = Mode::Normal;
    clamp_cursor(&ns)
}

// In-session edits: NO undo push (the session snapshot was taken on entry).
pub fn insert_text(s: &EditorState, ch: &str) -> EditorState {
    let text = s.lines.get(s.cursor.line).cloned().unwrap_or_default();
    let (before, after) = split_at_col(&text, s.cursor.col);
    let mut ns = s.clone();
    ns.lines[s.cursor.line] = format!("{before}{ch}{after}");
    ns.cursor.col += char_len(ch);
    ns
}
pub fn insert_tab(s: &EditorState) -> EditorState {
    insert_text(s, "  ")
}
pub fn insert_newline(s: &EditorState) -> EditorState {
    let text = s.lines.get(s.cursor.line).cloned().unwrap_or_default();
    let (before, after) = split_at_col(&text, s.cursor.col);
    let mut ns = s.clone();
    ns.lines
        .splice(s.cursor.line..=s.cursor.line, [before, after]);
    ns.cursor = Cursor {
        line: s.cursor.line + 1,
        col: 0,
    };
    ns
}
pub fn backspace(s: &EditorState) -> EditorState {
    let (line, col) = (s.cursor.line, s.cursor.col);
    if col > 0 {
        let chars: Vec<char> = s.lines[line].chars().collect();
        let mut new_line: String = chars[..col - 1].iter().collect();
        new_line.extend(chars[col..].iter());
        let mut ns = s.clone();
        ns.lines[line] = new_line;
        ns.cursor = Cursor { line, col: col - 1 };
        return ns;
    }
    if line > 0 {
        let prev = s.lines[line - 1].clone();
        let prev_len = char_len(&prev);
        let merged = format!("{prev}{}", s.lines[line]);
        let mut ns = s.clone();
        ns.lines.splice(line - 1..=line, [merged]);
        ns.cursor = Cursor {
            line: line - 1,
            col: prev_len,
        };
        return ns;
    }
    s.clone()
}
pub fn delete_word_before(s: &EditorState) -> EditorState {
    let chars: Vec<char> = s
        .lines
        .get(s.cursor.line)
        .cloned()
        .unwrap_or_default()
        .chars()
        .collect();
    let col = s.cursor.col.min(chars.len());
    let mut i = col;
    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }
    while i > 0 && !chars[i - 1].is_whitespace() {
        i -= 1;
    }
    let mut new_line: String = chars[..i].iter().collect();
    new_line.extend(chars[col..].iter());
    let mut ns = s.clone();
    ns.lines[s.cursor.line] = new_line;
    ns.cursor.col = i;
    ns
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::state::create_editor_state;

    fn ins(lines: &[&str], line: usize, col: usize) -> EditorState {
        let mut s = create_editor_state(lines.iter().map(|s| s.to_string()).collect(), vec![]);
        s.cursor = Cursor { line, col };
        s.mode = Mode::Insert;
        s
    }

    #[test]
    fn enter_insert_pushes_one_undo() {
        let s = create_editor_state(vec!["abc".into()], vec![]);
        let r = enter_insert(&s);
        assert_eq!(r.mode, Mode::Insert);
        assert_eq!(r.undo.len(), 1);
    }

    #[test]
    fn insert_text_and_newline() {
        let s = ins(&["ac"], 0, 1);
        let r = insert_text(&s, "b");
        assert_eq!(r.lines, vec!["abc".to_string()]);
        assert_eq!(r.cursor.col, 2);

        let r2 = insert_newline(&r);
        assert_eq!(r2.lines, vec!["ab".to_string(), "c".to_string()]);
        assert_eq!(r2.cursor, Cursor { line: 1, col: 0 });
    }

    #[test]
    fn backspace_merges_lines_at_col0() {
        let s = ins(&["ab", "cd"], 1, 0);
        let r = backspace(&s);
        assert_eq!(r.lines, vec!["abcd".to_string()]);
        assert_eq!(r.cursor, Cursor { line: 0, col: 2 });
    }

    #[test]
    fn open_below_inserts_blank_and_enters_insert() {
        let s = create_editor_state(vec!["x".into()], vec![]);
        let r = open_below(&s);
        assert_eq!(r.lines, vec!["x".to_string(), String::new()]);
        assert_eq!(r.mode, Mode::Insert);
        assert_eq!(r.cursor, Cursor { line: 1, col: 0 });
    }

    // Ported from web/src/lib/editor/insert.test.ts

    #[test]
    fn enter_insert_after_moves_one_past_cursor() {
        let mut s = create_editor_state(vec!["abc".into()], vec![]);
        s.cursor = Cursor { line: 0, col: 1 };
        let r = enter_insert_after(&s);
        assert_eq!(r.cursor.col, 2);
        assert_eq!(r.mode, Mode::Insert);
    }

    #[test]
    fn exit_insert_clamps_cursor_back_to_normal_bounds() {
        let s = ins(&["abc"], 0, 3);
        let r = exit_insert(&s);
        assert_eq!(r.mode, Mode::Normal);
        assert_eq!(r.cursor.col, 2);
    }

    #[test]
    fn delete_word_before_removes_previous_word() {
        let s = ins(&["foo bar"], 0, 7);
        let r = delete_word_before(&s);
        assert_eq!(r.lines, vec!["foo ".to_string()]);
    }

    // Additional edge-case coverage

    #[test]
    fn enter_insert_after_on_empty_line_stays_at_col0() {
        let s = create_editor_state(vec!["".into()], vec![]);
        let r = enter_insert_after(&s);
        assert_eq!(r.cursor.col, 0);
        assert_eq!(r.mode, Mode::Insert);
    }

    #[test]
    fn enter_insert_line_end_goes_to_end() {
        let s = create_editor_state(vec!["abc".into()], vec![]);
        let r = enter_insert_line_end(&s);
        assert_eq!(r.cursor.col, 3);
        assert_eq!(r.mode, Mode::Insert);
    }

    #[test]
    fn open_above_at_line0_inserts_before() {
        let s = create_editor_state(vec!["x".into()], vec![]);
        let r = open_above(&s);
        assert_eq!(r.lines, vec![String::new(), "x".to_string()]);
        assert_eq!(r.mode, Mode::Insert);
        assert_eq!(r.cursor, Cursor { line: 0, col: 0 });
    }

    #[test]
    fn delete_word_before_deletes_leading_whitespace_and_word() {
        let s = ins(&["foo  bar"], 0, 8);
        let r = delete_word_before(&s);
        assert_eq!(r.lines, vec!["foo  ".to_string()]);
    }

    #[test]
    fn backspace_at_origin_is_noop() {
        let s = ins(&["ab"], 0, 0);
        let r = backspace(&s);
        assert_eq!(r.lines, vec!["ab".to_string()]);
        assert_eq!(r.cursor, Cursor { line: 0, col: 0 });
    }
}
