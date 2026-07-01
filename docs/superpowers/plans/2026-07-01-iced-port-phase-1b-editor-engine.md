# Phase 1b — Editor Engine Port (`state`/`motions`/`edits`/`insert`/`keymap`) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **This is a port.** The authoritative behavioral spec for each module is the corresponding
> TypeScript file **and its `*.test.ts`** under `web/src/lib/editor/`. Each task gives the full Rust
> implementation plus representative ported tests, and names the exact `.test.ts` file to translate
> in full — a concrete instruction against an existing artifact, not a placeholder.

**Goal:** Port the pure, DOM-free editor state machine into `slugline-core`: `EditorState` with undo/redo, motions, edits, insert-mode operations, and the `handle_key` dispatcher — fully unit-tested and headless.

**Architecture:** An `editor` module of pure functions `fn(&EditorState, ...) -> EditorState`, mirroring the TS reducers. `handle_key(&EditorState, &KeyInput) -> KeyResult` is the single entry point. Columns are Unicode-scalar (`char`) indices.

**Tech Stack:** Rust, `regex` (task-checkbox toggle; already a `slugline-core` dep from Phase 1a).

**Scope (intentional deferral):** ports all *editing* keys — motions (`h j k l w b e 0 $ gg G`), edits (`x dd yy p P t u Ctrl-r`), and INSERT mode. It **defers**: (a) `:` command mode + dispatch → **Phase 5**; (b) the `AppEffect`-emitting keys `[ ] gt gT Ctrl-t` → **Phase 2**. `AppEffect`/`KeyResult` are defined now so later phases add handlers, not signatures.

---

## File Structure (files added/changed in Phase 1b)

```
crates/slugline-core/
  src/
    lib.rs                         # + pub mod editor;
    editor/
      mod.rs                       # module decls + re-exports
      state.rs                     # port of editor/state.ts
      motions.rs                   # port of editor/motions.ts
      edits.rs                     # port of editor/edits.ts
      insert.rs                    # port of editor/insert.ts
      keymap.rs                    # port of editor/keymap.ts (editing subset) + key/effect types
```

---

### Task 1: Editor state (undo/redo/clamp)

**Files:**
- Modify: `crates/slugline-core/src/lib.rs` (add `pub mod editor;`)
- Create: `crates/slugline-core/src/editor/mod.rs`, `crates/slugline-core/src/editor/state.rs`

- [ ] **Step 1: Declare the module** — add `pub mod editor;` to `crates/slugline-core/src/lib.rs`; create `crates/slugline-core/src/editor/mod.rs`:

```rust
pub mod edits;
pub mod insert;
pub mod keymap;
pub mod motions;
pub mod state;

pub use keymap::{handle_key, AppEffect, KeyInput, KeyResult};
pub use state::{clamp_cursor, create_editor_state, Cursor, EditorState, Mode, Pending};
```

- [ ] **Step 2: Write the failing test** — `crates/slugline-core/src/editor/state.rs`:

```rust
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
    todo!()
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
}
```

- [ ] **Step 3: Implement the state functions** (replace `todo!()`, add the rest below it):

```rust
pub fn create_editor_state(lines: Vec<String>, register: Vec<String>) -> EditorState {
    EditorState {
        lines: if lines.is_empty() { vec![String::new()] } else { lines },
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
    Snapshot { lines: s.lines.clone(), cursor: s.cursor }
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
        None => { ns.message = "Already at oldest change".into(); ns }
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
        None => { ns.message = "Already at newest change".into(); ns }
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
    ns.cursor = Cursor { line, col: ns.cursor.col.min(max_col) };
    ns
}
```

- [ ] **Step 4: Run test to verify it passes** — `cargo test -p slugline-core editor::state`
Expected: PASS (3 tests).

- [ ] **Step 5: Port remaining cases** from `web/src/lib/editor/state.test.ts` (undo/redo stack depth, `message` text on exhausted undo/redo, snapshot isolation from later mutation). Run until green.

- [ ] **Step 6: Commit**

```bash
git add crates/slugline-core/
git commit -m "feat(core): port editor state (undo/redo/clamp)"
```

---

### Task 2: Motions

**Files:** Create `crates/slugline-core/src/editor/motions.rs`

- [ ] **Step 1: Write the failing test**:

```rust
use super::state::{clamp_cursor, Cursor, EditorState, Mode};

// implementations added in Step 3

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
}
```

- [ ] **Step 2: Run test to verify it fails** — `cargo test -p slugline-core editor::motions`
Expected: FAIL to compile (functions undefined).

- [ ] **Step 3: Implement motions** (prepend above the test module):

```rust
fn line_chars(s: &EditorState, line: usize) -> Vec<char> {
    s.lines.get(line).map(|t| t.chars().collect()).unwrap_or_default()
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
```

- [ ] **Step 4: Run test to verify it passes** — `cargo test -p slugline-core editor::motions`
Expected: PASS (3 tests).

- [ ] **Step 5: Port remaining cases** from `web/src/lib/editor/motions.test.ts` (punctuation-run boundaries for `w`/`b`/`e`, behavior at line start/end, `gg`/`G` column clamping across lines). Run until green.

- [ ] **Step 6: Commit**

```bash
git add crates/slugline-core/src/editor/motions.rs
git commit -m "feat(core): port editor motions"
```

---

### Task 3: Edits

**Files:** Create `crates/slugline-core/src/editor/edits.rs`

- [ ] **Step 1: Write the failing test**:

```rust
use std::sync::LazyLock;

use regex::Regex;

use super::state::{clamp_cursor, push_undo, Cursor, EditorState};

// implementations added in Step 3

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
        assert_eq!(r.lines, vec!["a".to_string(), "X".to_string(), "b".to_string()]);
        assert_eq!(r.cursor, Cursor { line: 1, col: 0 });
    }

    #[test]
    fn toggle_todo_flips_checkbox() {
        let s = at(&["- [ ] task"], 0, 0);
        let r = toggle_todo(&s);
        assert_eq!(r.lines, vec!["- [x] task".to_string()]);
        assert_eq!(toggle_todo(&r).lines, vec!["- [ ] task".to_string()]);
    }
}
```

- [ ] **Step 2: Run test to verify it fails** — `cargo test -p slugline-core editor::edits`
Expected: FAIL to compile.

- [ ] **Step 3: Implement edits** (prepend above the tests):

```rust
static TASK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*- \[)([ xX])(\] )").unwrap());

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
```

- [ ] **Step 4: Run test to verify it passes** — `cargo test -p slugline-core editor::edits`
Expected: PASS (4 tests).

- [ ] **Step 5: Port remaining cases** from `web/src/lib/editor/edits.test.ts` (`P` position, `yy` sets message, `x` on empty line is a no-op, `dd` on the last line clamps the cursor up, `t` is a no-op on non-task lines and preserves indentation). Run until green.

- [ ] **Step 6: Commit**

```bash
git add crates/slugline-core/src/editor/edits.rs
git commit -m "feat(core): port editor edits (x/dd/yy/p/P/t)"
```

---

### Task 4: Insert-mode operations

**Files:** Create `crates/slugline-core/src/editor/insert.rs`

- [ ] **Step 1: Write the failing test**:

```rust
use super::state::{clamp_cursor, push_undo, Cursor, EditorState, Mode};

// implementations added in Step 3

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
}
```

- [ ] **Step 2: Run test to verify it fails** — `cargo test -p slugline-core editor::insert`
Expected: FAIL to compile.

- [ ] **Step 3: Implement insert ops** (prepend above the tests):

```rust
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
    let len = char_len(ns.lines.get(ns.cursor.line).map(String::as_str).unwrap_or(""));
    ns.mode = Mode::Insert;
    ns.cursor.col = (ns.cursor.col + if len > 0 { 1 } else { 0 }).min(len);
    ns
}
pub fn enter_insert_line_end(s: &EditorState) -> EditorState {
    let mut ns = push_undo(s);
    let len = char_len(ns.lines.get(ns.cursor.line).map(String::as_str).unwrap_or(""));
    ns.mode = Mode::Insert;
    ns.cursor.col = len;
    ns
}
pub fn open_below(s: &EditorState) -> EditorState {
    let mut ns = push_undo(s);
    ns.lines.insert(ns.cursor.line + 1, String::new());
    ns.mode = Mode::Insert;
    ns.cursor = Cursor { line: ns.cursor.line + 1, col: 0 };
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
    ns.lines.splice(s.cursor.line..=s.cursor.line, [before, after]);
    ns.cursor = Cursor { line: s.cursor.line + 1, col: 0 };
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
        ns.cursor = Cursor { line: line - 1, col: prev_len };
        return ns;
    }
    s.clone()
}
pub fn delete_word_before(s: &EditorState) -> EditorState {
    let chars: Vec<char> = s.lines.get(s.cursor.line).cloned().unwrap_or_default().chars().collect();
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
```

- [ ] **Step 4: Run test to verify it passes** — `cargo test -p slugline-core editor::insert`
Expected: PASS (4 tests).

- [ ] **Step 5: Port remaining cases** from `web/src/lib/editor/insert.test.ts` (`a` on empty line stays at col 0, `A` goes to end, `O` at line 0, `Ctrl-W` deletes leading whitespace + word, backspace at (0,0) is a no-op). Run until green.

- [ ] **Step 6: Commit**

```bash
git add crates/slugline-core/src/editor/insert.rs
git commit -m "feat(core): port insert-mode operations"
```

---

### Task 5: Key types + keymap dispatcher

**Files:** Create `crates/slugline-core/src/editor/keymap.rs`

- [ ] **Step 1: Write the failing test**:

```rust
use super::state::{EditorState, Mode, Pending};
use super::{edits, insert, motions, state};

/// A normalized key event. `key` follows the DOM `KeyboardEvent.key` convention
/// ("h", "A", "Enter", "Escape", "ArrowLeft", " ", ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInput {
    pub key: String,
    pub ctrl: bool,
    pub meta: bool,
    pub shift: bool,
}

/// Side effects the app performs (navigation, save, theme). Handlers land in Phase 2/5;
/// the variants exist now so signatures are stable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEffect {
    Goto(String),
    Today,
    Tab(String),
    Close,
    Save,
    PrevDay,
    NextDay,
    TabNext,
    TabPrev,
    Theme(String),
}

pub struct KeyResult {
    pub state: EditorState,
    pub effect: Option<AppEffect>,
}

pub fn handle_key(state: &EditorState, key: &KeyInput) -> KeyResult {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::state::create_editor_state;

    fn key(k: &str) -> KeyInput {
        KeyInput { key: k.into(), ctrl: false, meta: false, shift: false }
    }

    #[test]
    fn i_enters_insert_then_typing_inserts() {
        let s = create_editor_state(vec!["ac".into()], vec![]);
        let s = handle_key(&s, &key("i")).state;
        assert_eq!(s.mode, Mode::Insert);
        let s = handle_key(&s, &key("x")).state;
        assert_eq!(s.lines, vec!["xac".to_string()]);
    }

    #[test]
    fn dd_requires_two_keystrokes() {
        let s = create_editor_state(vec!["one".into(), "two".into()], vec![]);
        let s = handle_key(&s, &key("d")).state;
        assert_eq!(s.pending, Pending::D);
        let s = handle_key(&s, &key("d")).state;
        assert_eq!(s.lines, vec!["two".to_string()]);
    }

    #[test]
    fn ctrl_r_redoes() {
        let mut s = create_editor_state(vec!["a".into()], vec![]);
        s = handle_key(&s, &key("x")).state; // delete 'a' (pushes undo)
        s = handle_key(&s, &key("u")).state; // undo
        assert_eq!(s.lines, vec!["a".to_string()]);
        let r = handle_key(&s, &KeyInput { key: "r".into(), ctrl: true, meta: false, shift: false }).state;
        assert_eq!(r.lines, vec![String::new()]);
    }
}
```

- [ ] **Step 2: Run test to verify it fails** — `cargo test -p slugline-core editor::keymap`
Expected: FAIL (`todo!()`).

- [ ] **Step 3: Implement `handle_key`** (replace `todo!()`, add helpers below). Deferred keys (`:`, `[`, `]`, `gt`, `gT`, `Ctrl-t`) fall through to no-ops this phase:

```rust
fn state_only(state: EditorState) -> KeyResult {
    KeyResult { state, effect: None }
}

pub fn handle_key(state: &EditorState, key: &KeyInput) -> KeyResult {
    // Phase 1: command mode (`:`) is never entered; it is added in Phase 5.
    if state.mode == Mode::Insert {
        return state_only(handle_insert(state, key));
    }
    handle_normal(state, key)
}

fn handle_insert(s: &EditorState, k: &KeyInput) -> EditorState {
    match k.key.as_str() {
        "Escape" => insert::exit_insert(s),
        "Backspace" => insert::backspace(s),
        "Enter" => insert::insert_newline(s),
        "Tab" => insert::insert_tab(s),
        "ArrowLeft" => motions::move_left(s),
        "ArrowRight" => motions::move_right(s),
        "ArrowUp" => motions::move_up(s),
        "ArrowDown" => motions::move_down(s),
        _ => {
            if k.ctrl && (k.key == "w" || k.key == "W") {
                insert::delete_word_before(s)
            } else if k.key.chars().count() == 1 && !k.ctrl && !k.meta {
                insert::insert_text(s, &k.key)
            } else {
                s.clone()
            }
        }
    }
}

fn with_pending(s: &EditorState, p: Pending) -> EditorState {
    let mut ns = s.clone();
    ns.pending = p;
    ns
}

fn handle_normal(s: &EditorState, k: &KeyInput) -> KeyResult {
    match s.pending {
        Pending::G => {
            let s2 = with_pending(s, Pending::None);
            return match k.key.as_str() {
                "g" => state_only(motions::first_line(&s2)),
                // "t"/"T" (gt/gT) emit TabNext/TabPrev — deferred to Phase 2.
                _ => handle_normal(&s2, k),
            };
        }
        Pending::D => {
            let s2 = with_pending(s, Pending::None);
            return match k.key.as_str() {
                "d" => state_only(edits::delete_line(&s2)),
                _ => handle_normal(&s2, k),
            };
        }
        Pending::Y => {
            let s2 = with_pending(s, Pending::None);
            return match k.key.as_str() {
                "y" => state_only(edits::yank_line(&s2)),
                _ => handle_normal(&s2, k),
            };
        }
        Pending::None => {}
    }

    let st = match k.key.as_str() {
        "h" | "ArrowLeft" => motions::move_left(s),
        "l" | "ArrowRight" => motions::move_right(s),
        "j" | "ArrowDown" => motions::move_down(s),
        "k" | "ArrowUp" => motions::move_up(s),
        "w" => motions::word_forward(s),
        "b" => motions::word_backward(s),
        "e" => motions::word_end(s),
        "0" => motions::line_start(s),
        "$" => motions::line_end(s),
        "G" => motions::last_line(s),
        "g" => with_pending(s, Pending::G),
        "d" => with_pending(s, Pending::D),
        "y" => with_pending(s, Pending::Y),
        "x" => edits::delete_char(s),
        "p" => edits::paste_below(s),
        "P" => edits::paste_above(s),
        "t" => edits::toggle_todo(s),
        "u" => state::undo(s),
        "i" => insert::enter_insert(s),
        "a" => insert::enter_insert_after(s),
        "A" => insert::enter_insert_line_end(s),
        "o" => insert::open_below(s),
        "O" => insert::open_above(s),
        "Enter" => motions::move_down(s),
        // ":" / "[" / "]" / Ctrl-t are deferred (Phase 2/5).
        _ => {
            if k.ctrl && (k.key == "r" || k.key == "R") {
                state::redo(s)
            } else {
                s.clone()
            }
        }
    };
    state_only(st)
}
```

- [ ] **Step 4: Run test to verify it passes** — `cargo test -p slugline-core editor::keymap`
Expected: PASS (3 tests).

- [ ] **Step 5: Port the editing-subset cases** from `web/src/lib/editor/keymap.test.ts` — port every case that exercises NORMAL editing/motions and INSERT mode. **Skip and leave a `// TODO(phase 2/5)` comment** for cases covering `:` command mode and the `[`/`]`/`gt`/`gT`/`Ctrl-t` effects. **Flag for review:** `keymap.ts:137-138` maps `Escape` in NORMAL mode to `enterInsert(...)`, which enters INSERT — this looks like a latent bug. Do **not** replicate it (this plan omits normal-mode `Escape`). If `keymap.test.ts` asserts the old behavior, do not silently match it: surface it to the maintainer and confirm the intended behavior before adding a test. Run until green.

- [ ] **Step 6: Full core test + hygiene**

Run: `cargo test -p slugline-core && cargo fmt --all -- --check && cargo clippy -p slugline-core -- -D warnings`
Expected: all green.

- [ ] **Step 7: Commit**

```bash
git add crates/slugline-core/src/editor/keymap.rs
git commit -m "feat(core): port keymap dispatcher (editing subset) + key/effect types"
```

---

## Self-Review (performed while writing this plan)

- **Spec coverage:** Implements the `editor` half of design Sections 2–3 — the pure state machine the
  Iced `update` will call. `AppEffect`/`KeyResult`/`KeyInput` are defined here so Phases 2 and 5 add
  handlers without changing signatures.
- **Type consistency:** `EditorState`/`Cursor`/`Mode`/`Pending` defined once in `state.rs`;
  `motions`/`edits`/`insert`/`keymap` all import them; `mod.rs` re-exports `handle_key`,
  `create_editor_state`, `clamp_cursor`, and the key/effect types used by Phase 1c.
- **Placeholder scan:** the only `todo!()`s are intentional red-phase stubs replaced in-task.
- **Judgment flagged:** normal-mode `Escape` (suspected upstream bug at `keymap.ts:137`) is
  deliberately omitted with an explicit review step — not silently ported.
- **Known divergence (intentional):** char-index columns vs. the TS UTF-16 indices (identical for
  BMP text; differs only for emoji, which is out of scope per the design).
