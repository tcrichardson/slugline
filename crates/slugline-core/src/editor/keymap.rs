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

fn state_only(state: EditorState) -> KeyResult {
    KeyResult {
        state,
        effect: None,
    }
}

fn state_effect(state: EditorState, effect: AppEffect) -> KeyResult {
    KeyResult {
        state,
        effect: Some(effect),
    }
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
                "t" => state_effect(s2, AppEffect::TabNext),
                "T" => state_effect(s2, AppEffect::TabPrev),
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

    // Navigation effects (Phase 2): reachable in NORMAL mode only.
    match k.key.as_str() {
        "[" => return state_effect(s.clone(), AppEffect::PrevDay),
        "]" => return state_effect(s.clone(), AppEffect::NextDay),
        _ => {}
    }
    if k.ctrl && (k.key == "t" || k.key == "T") {
        return state_effect(s.clone(), AppEffect::Today);
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
        // ":" command mode is deferred to Phase 5.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::state::{Cursor, create_editor_state};

    fn key(k: &str) -> KeyInput {
        KeyInput {
            key: k.into(),
            ctrl: false,
            meta: false,
            shift: false,
        }
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
        let r = handle_key(
            &s,
            &KeyInput {
                key: "r".into(),
                ctrl: true,
                meta: false,
                shift: false,
            },
        )
        .state;
        assert_eq!(r.lines, vec![String::new()]);
    }

    // Ported from web/src/lib/editor/keymap.test.ts (editing subset only)

    #[test]
    fn j_moves_down_in_normal_mode() {
        let s = create_editor_state(vec!["a".into(), "b".into()], vec![]);
        let r = handle_key(&s, &key("j")).state;
        assert_eq!(r.cursor.line, 1);
    }

    #[test]
    fn escape_exits_insert_mode() {
        let s = create_editor_state(vec!["".into()], vec![]);
        let s = handle_key(&s, &key("i")).state;
        assert_eq!(s.mode, Mode::Insert);
        let s = handle_key(&s, &key("x")).state;
        assert_eq!(s.lines, vec!["x".to_string()]);
        let s = handle_key(&s, &key("Escape")).state;
        assert_eq!(s.mode, Mode::Normal);
    }

    #[test]
    fn gg_jumps_to_first_line() {
        let mut s = create_editor_state(vec!["a".into(), "b".into(), "c".into()], vec![]);
        s.cursor = Cursor { line: 2, col: 0 };
        let s = handle_key(&s, &key("g")).state;
        let s = handle_key(&s, &key("g")).state;
        assert_eq!(s.cursor.line, 0);
    }

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

    // Review flag: keymap.ts:137-138 maps Escape in NORMAL mode to enterInsert(...).
    // This looks like a latent bug and is deliberately NOT replicated here.
}
