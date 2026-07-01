use super::state::{EditorState, Mode, clamp_cursor};

fn line_chars(s: &EditorState, line: usize) -> Vec<char> {
    s.lines
        .get(line)
        .map(|t| t.chars().collect())
        .unwrap_or_default()
}

pub fn move_left(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    ns.cursor.col = ns.cursor.col.saturating_sub(1);
    clamp_cursor(&ns)
}
pub fn move_right(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    ns.cursor.col += 1;
    clamp_cursor(&ns)
}
pub fn move_up(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    ns.cursor.line = ns.cursor.line.saturating_sub(1);
    clamp_cursor(&ns)
}
pub fn move_down(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    ns.cursor.line = (ns.cursor.line + 1).min(ns.lines.len().saturating_sub(1));
    clamp_cursor(&ns)
}
pub fn line_start(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    ns.cursor.col = 0;
    ns
}
pub fn line_end(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    let len = line_chars(s, s.cursor.line).len();
    ns.cursor.col = match s.mode {
        Mode::Insert => len,
        Mode::Normal => len.saturating_sub(1),
    };
    ns
}
pub fn first_line(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    ns.cursor.line = 0;
    clamp_cursor(&ns)
}
pub fn last_line(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    ns.cursor.line = s.lines.len().saturating_sub(1);
    clamp_cursor(&ns)
}

// vim "word": a run of word chars OR a run of punctuation, separated by whitespace.
fn class_of(ch: Option<char>) -> u8 {
    match ch {
        None => 0,
        Some(c) if c.is_whitespace() => 0,
        Some(c) if c.is_alphanumeric() || c == '_' => 1,
        Some(_) => 2,
    }
}

pub fn word_forward(s: &EditorState) -> EditorState {
    let text = line_chars(s, s.cursor.line);
    let n = text.len();
    let mut i = s.cursor.col;
    if i >= n {
        return s.clone();
    }
    let start = class_of(text.get(i).copied());
    if start != 0 {
        while i < n && class_of(text.get(i).copied()) == start {
            i += 1;
        }
    }
    while i < n && class_of(text.get(i).copied()) == 0 {
        i += 1;
    }
    let mut ns = s.clone();
    ns.cursor.col = i.min(n.saturating_sub(1));
    ns
}

pub fn word_backward(s: &EditorState) -> EditorState {
    let text = line_chars(s, s.cursor.line);
    let mut i = s.cursor.col;
    if i == 0 {
        return s.clone();
    }
    i -= 1;
    while i > 0 && class_of(text.get(i).copied()) == 0 {
        i -= 1;
    }
    let cl = class_of(text.get(i).copied());
    while i > 0 && class_of(text.get(i - 1).copied()) == cl && cl != 0 {
        i -= 1;
    }
    let mut ns = s.clone();
    ns.cursor.col = i;
    ns
}

pub fn word_end(s: &EditorState) -> EditorState {
    let text = line_chars(s, s.cursor.line);
    let n = text.len();
    let mut i = s.cursor.col;
    if n == 0 || i >= n - 1 {
        return s.clone();
    }
    i += 1;
    while i < n && class_of(text.get(i).copied()) == 0 {
        i += 1;
    }
    let cl = class_of(text.get(i).copied());
    while i < n - 1 && class_of(text.get(i + 1).copied()) == cl && cl != 0 {
        i += 1;
    }
    let mut ns = s.clone();
    ns.cursor.col = i.min(n - 1);
    ns
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::state::{Cursor, create_editor_state};

    fn at(lines: &[&str], line: usize, col: usize) -> EditorState {
        let mut s = create_editor_state(lines.iter().map(|s| s.to_string()).collect(), vec![]);
        s.cursor = Cursor { line, col };
        s
    }

    #[test]
    fn hjkl_clamps() {
        let s = at(&["abc", "de"], 0, 0);
        assert_eq!(move_left(&s).cursor.col, 0);
        assert_eq!(move_right(&s).cursor.col, 1);
        assert_eq!(move_down(&s).cursor, Cursor { line: 1, col: 0 });
    }

    #[test]
    fn word_forward_skips_to_next_word() {
        let s = at(&["foo bar"], 0, 0);
        assert_eq!(word_forward(&s).cursor.col, 4);
    }

    #[test]
    fn line_end_respects_mode() {
        let mut s = at(&["abc"], 0, 1);
        assert_eq!(line_start(&s).cursor.col, 0);
        assert_eq!(line_end(&s).cursor.col, 2); // normal: len-1
        s.mode = Mode::Insert;
        assert_eq!(line_end(&s).cursor.col, 3); // insert: len
    }

    // Ported from web/src/lib/editor/motions.test.ts

    #[test]
    fn move_right_stops_at_last_char_in_normal_mode() {
        let s = at(&["ab"], 0, 1);
        assert_eq!(move_right(&s).cursor.col, 1);
    }

    #[test]
    fn move_down_stays_within_bounds() {
        let s = at(&["a", "b"], 1, 0);
        assert_eq!(move_down(&s).cursor.line, 1);
    }

    #[test]
    fn last_line_jumps_to_final_line() {
        let s = at(&["a", "b", "c"], 0, 0);
        assert_eq!(last_line(&s).cursor.line, 2);
    }

    #[test]
    fn word_backward_jumps_to_previous_word_start() {
        let s = at(&["foo bar"], 0, 4);
        assert_eq!(word_backward(&s).cursor.col, 0);
    }

    #[test]
    fn word_end_jumps_to_end_of_next_word() {
        let s = at(&["foo bar"], 0, 0);
        assert_eq!(word_end(&s).cursor.col, 2);
    }

    // Additional edge-case coverage

    #[test]
    fn word_forward_across_punctuation_runs() {
        let s = at(&["foo!bar"], 0, 0);
        assert_eq!(word_forward(&s).cursor.col, 3); // end of "foo"
        let s = at(&["foo!bar"], 0, 3);
        assert_eq!(word_forward(&s).cursor.col, 4); // start of "bar"
    }

    #[test]
    fn word_backward_across_punctuation_runs() {
        let s = at(&["foo!bar"], 0, 4);
        assert_eq!(word_backward(&s).cursor.col, 3); // start of "!"
        let s = at(&["foo!bar"], 0, 3);
        assert_eq!(word_backward(&s).cursor.col, 0); // start of "foo"
    }

    #[test]
    fn word_end_across_punctuation_runs() {
        let s = at(&["foo!bar"], 0, 0);
        assert_eq!(word_end(&s).cursor.col, 2); // end of "foo"
        let s = at(&["foo!bar"], 0, 2);
        assert_eq!(word_end(&s).cursor.col, 3); // end of "!"
        let s = at(&["foo!bar"], 0, 3);
        assert_eq!(word_end(&s).cursor.col, 6); // end of "bar"
    }

    #[test]
    fn word_motions_at_line_boundaries() {
        let s = at(&[""], 0, 0);
        assert_eq!(word_forward(&s).cursor.col, 0);
        assert_eq!(word_backward(&s).cursor.col, 0);
        assert_eq!(word_end(&s).cursor.col, 0);
    }

    #[test]
    fn first_line_clamps_col_across_lines() {
        let s = at(&["ab", "x"], 1, 0);
        // gg from line 1 should go to line 0 and clamp col to available length
        assert_eq!(first_line(&s).cursor, Cursor { line: 0, col: 0 });
    }

    #[test]
    fn last_line_clamps_col_across_lines() {
        let mut s = at(&["x", "abc"], 0, 2);
        s.mode = Mode::Normal;
        assert_eq!(last_line(&s).cursor, Cursor { line: 1, col: 2 });
    }
}
