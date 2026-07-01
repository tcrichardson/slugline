#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub lines: Vec<String>,
    pub cursor: Cursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pending {
    None,
    G,
    D,
    Y,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorState {
    pub lines: Vec<String>,
    pub cursor: Cursor,
    pub mode: Mode,
    pub register: Vec<String>,
    pub pending: Pending,
    pub command: Option<String>,
    pub message: String,
    pub undo: Vec<Snapshot>,
    pub redo: Vec<Snapshot>,
}

pub fn create_editor_state(lines: Vec<String>, register: Vec<String>) -> EditorState {
    EditorState {
        lines: if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        },
        cursor: Cursor { line: 0, col: 0 },
        mode: Mode::Normal,
        register,
        pending: Pending::None,
        command: None,
        message: String::new(),
        undo: Vec::new(),
        redo: Vec::new(),
    }
}

pub fn snapshot(s: &EditorState) -> Snapshot {
    Snapshot {
        lines: s.lines.clone(),
        cursor: s.cursor,
    }
}

/// Snapshot the pre-mutation state and clear redo. Call BEFORE applying a mutation.
pub fn push_undo(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    ns.undo.push(snapshot(s));
    ns.redo.clear();
    ns
}

pub fn undo(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    match ns.undo.pop() {
        None => {
            ns.message = "Already at oldest change".into();
            ns
        }
        Some(prev) => {
            ns.redo.push(snapshot(s));
            ns.lines = prev.lines;
            ns.cursor = prev.cursor;
            ns.message = String::new();
            ns
        }
    }
}

pub fn redo(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    match ns.redo.pop() {
        None => {
            ns.message = "Already at newest change".into();
            ns
        }
        Some(next) => {
            ns.undo.push(snapshot(s));
            ns.lines = next.lines;
            ns.cursor = next.cursor;
            ns.message = String::new();
            ns
        }
    }
}

/// Clamp cursor to valid bounds for the current mode. Columns are char indices.
pub fn clamp_cursor(s: &EditorState) -> EditorState {
    let mut ns = s.clone();
    let line = ns.cursor.line.min(ns.lines.len().saturating_sub(1));
    let len = ns.lines[line].chars().count();
    let max_col = match ns.mode {
        Mode::Insert => len,
        Mode::Normal => len.saturating_sub(1),
    };
    ns.cursor = Cursor {
        line,
        col: ns.cursor.col.min(max_col),
    };
    ns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_never_has_zero_lines() {
        let s = create_editor_state(vec![], vec![]);
        assert_eq!(s.lines, vec![String::new()]);
        assert_eq!(s.cursor, Cursor { line: 0, col: 0 });
        assert_eq!(s.mode, Mode::Normal);
    }

    #[test]
    fn undo_redo_round_trip() {
        let mut s = create_editor_state(vec!["a".into()], vec![]);
        s = push_undo(&s); // snapshot "a"
        s.lines = vec!["b".into()];
        let undone = undo(&s);
        assert_eq!(undone.lines, vec!["a".to_string()]);
        let redone = redo(&undone);
        assert_eq!(redone.lines, vec!["b".to_string()]);
    }

    #[test]
    fn clamp_respects_mode() {
        let base = create_editor_state(vec!["abc".into()], vec![]);
        let mut n = base.clone();
        n.cursor = Cursor { line: 0, col: 99 };
        assert_eq!(clamp_cursor(&n).cursor.col, 2); // normal: len-1
        let mut ins = base.clone();
        ins.mode = Mode::Insert;
        ins.cursor = Cursor { line: 0, col: 99 };
        assert_eq!(clamp_cursor(&ins).cursor.col, 3); // insert: len
    }

    // Ported from web/src/lib/editor/state.test.ts

    #[test]
    fn preserves_provided_register() {
        let s = create_editor_state(vec!["x".into()], vec!["line".into()]);
        assert_eq!(s.register, vec!["line".to_string()]);
    }

    #[test]
    fn undo_redo_round_trip_two_lines() {
        let mut s = create_editor_state(vec!["a".into(), "b".into()], vec![]);
        s = push_undo(&s);
        s.lines = vec!["a".into(), "b".into(), "c".into()];
        s = undo(&s);
        assert_eq!(s.lines, vec!["a".to_string(), "b".to_string()]);
        s = redo(&s);
        assert_eq!(
            s.lines,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn exhausted_undo_shows_message() {
        let s = create_editor_state(vec!["a".into()], vec![]);
        let r = undo(&s);
        assert_eq!(r.message, "Already at oldest change");
    }

    #[test]
    fn exhausted_redo_shows_message() {
        let s = create_editor_state(vec!["a".into()], vec![]);
        let r = redo(&s);
        assert_eq!(r.message, "Already at newest change");
    }

    #[test]
    fn snapshot_isolation() {
        let mut s = create_editor_state(vec!["a".into()], vec![]);
        s = push_undo(&s);
        s.lines = vec!["b".into()];
        // Mutate after snapshot
        s.lines[0].push_str("c");
        let undone = undo(&s);
        // Undo should restore "a", not "bc"
        assert_eq!(undone.lines, vec!["a".to_string()]);
    }
}
