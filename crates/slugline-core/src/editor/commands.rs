use crate::doc::{
    Block, CommandName, Context, Line, Section, SectionKind, ValidationResult, classify_line,
    nearest_heading_level, resolve_context, scan_document, validate_command,
};

use super::keymap::AppEffect;
use super::state::{Cursor, EditorState, clamp_cursor, push_undo};

/// Append an H3 (or any heading text) at the end of a section's content.
pub fn append_block(lines: &[String], section: &Section, heading: &str) -> (Vec<String>, usize) {
    let idx = section.end_line + 1;
    let mut out = lines.to_vec();
    out.splice(idx..idx, [heading.to_string(), String::new()]);
    (out, idx)
}

/// Append a single line after the last non-blank line of a section (or right after its
/// heading).
pub fn append_line_to_section(
    lines: &[String],
    section: &Section,
    text: &str,
) -> (Vec<String>, usize) {
    let mut insert_at = section.start_line + 1;
    for i in (section.start_line + 1)..=section.end_line {
        if !lines.get(i).map(|l| l.trim()).unwrap_or("").is_empty() {
            insert_at = i + 1;
        }
    }
    let mut out = lines.to_vec();
    out.insert(insert_at, text.to_string());
    (out, insert_at)
}

/// Insert or update a `meta:key value` line within a block's meta region.
pub fn upsert_meta(
    lines: &[String],
    block: &Block,
    key: &str,
    value: &str,
) -> (Vec<String>, usize) {
    let meta_line = format!("meta:{key} {value}");
    let mut out = lines.to_vec();
    if let Some(existing) = block.meta.iter().find(|m| m.key == key) {
        let line_index = existing.line_index;
        out[line_index] = meta_line;
        return (out, line_index);
    }
    let insert_at = block.meta_end_line + 1; // meta_end_line == heading when no meta yet
    out.insert(insert_at, meta_line);
    (out, insert_at)
}

/// Append a new value to an existing `meta:key` line, or create it if absent. Values are
/// joined with `", "`.
pub fn append_meta(
    lines: &[String],
    block: &Block,
    key: &str,
    new_value: &str,
) -> (Vec<String>, usize) {
    let trimmed = new_value.trim();
    if let Some(existing) = block.meta.iter().find(|m| m.key == key) {
        let existing_value = existing.value.trim();
        if !existing_value.is_empty() {
            let combined = format!("{existing_value}, {trimmed}");
            return upsert_meta(lines, block, key, &combined);
        }
    }
    upsert_meta(lines, block, key, trimmed)
}

/// The three standard top-level sections a note may need created on demand, in the
/// canonical order they appear when all three are present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardSection {
    Todo,
    Meetings,
    Notes,
}

const STANDARD_SECTION_ORDER: [StandardSection; 3] = [
    StandardSection::Todo,
    StandardSection::Meetings,
    StandardSection::Notes,
];

impl StandardSection {
    fn heading(self) -> &'static str {
        match self {
            StandardSection::Todo => "## To Do",
            StandardSection::Meetings => "## Meetings",
            StandardSection::Notes => "## Notes",
        }
    }

    fn matches(self, kind: SectionKind) -> bool {
        matches!(
            (self, kind),
            (StandardSection::Todo, SectionKind::Todo)
                | (StandardSection::Meetings, SectionKind::Meetings)
                | (StandardSection::Notes, SectionKind::Notes)
        )
    }
}

/// Ensure a standard section exists; if missing, insert it in canonical order.
pub fn ensure_section(lines: &[String], kind: StandardSection) -> (Vec<String>, Section) {
    let model = scan_document(lines);
    if let Some(found) = model.sections.iter().find(|s| kind.matches(s.kind)) {
        return (lines.to_vec(), found.clone());
    }

    let order_idx = STANDARD_SECTION_ORDER
        .iter()
        .position(|&k| k == kind)
        .expect("kind is a StandardSection variant");
    let mut insert_at = lines.len();
    let mut placed = false;
    for prev_kind in STANDARD_SECTION_ORDER[..order_idx].iter().rev() {
        if let Some(prev) = model.sections.iter().find(|s| prev_kind.matches(s.kind)) {
            insert_at = prev.end_line + 1;
            placed = true;
            break;
        }
    }
    if !placed {
        insert_at = model.title_line_index.map_or(0, |i| i + 1);
    }

    let mut out = lines.to_vec();
    out.splice(
        insert_at..insert_at,
        [String::new(), kind.heading().to_string(), String::new()],
    );
    let rescanned = scan_document(&out);
    let section = rescanned
        .sections
        .iter()
        .find(|s| kind.matches(s.kind))
        .expect("just inserted this section")
        .clone();
    (out, section)
}

/// Index just past the enclosing heading's content (before the next same/shallower
/// heading, or EOF). Never panics on an empty document.
pub fn end_of_enclosing_section(lines: &[String], cursor_line: usize, level: u8) -> usize {
    if lines.is_empty() {
        return 0;
    }
    let clamped = cursor_line.min(lines.len() - 1);

    let mut start = None;
    for i in (0..=clamped).rev() {
        if let Line::Heading { level: lvl, .. } = classify_line(&lines[i])
            && lvl == level
        {
            start = Some(i);
            break;
        }
    }
    let start = start.unwrap_or(cursor_line);

    for (i, l) in lines.iter().enumerate().skip(start + 1) {
        if let Line::Heading { level: lvl, .. } = classify_line(l)
            && lvl <= level
        {
            return i;
        }
    }
    lines.len()
}

/// What `nowHHMM` was for `:start`/`:end` when a command ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCtx {
    pub now_hhmm: String,
}

/// The outcome of running a validated `:` command.
pub struct CommandResult {
    pub state: EditorState,
    pub effect: Option<AppEffect>,
}

fn add_block(state: &EditorState, kind: StandardSection, name: &str) -> EditorState {
    let ns = push_undo(state);
    let (lines, section) = ensure_section(&ns.lines, kind);
    let (lines, heading_index) = append_block(&lines, &section, &format!("### {name}"));
    let mut ns = ns;
    ns.lines = lines;
    ns.cursor = Cursor {
        line: heading_index,
        col: 0,
    };
    ns.message = String::new();
    clamp_cursor(&ns)
}

fn add_todo(state: &EditorState, text: &str) -> EditorState {
    let ns = push_undo(state);
    let model = scan_document(&ns.lines);
    let suffix = match resolve_context(&model, ns.cursor.line) {
        Context::Meeting { block, .. } => format!(" _({})_", block.name),
        _ => String::new(),
    };
    let (lines, section) = ensure_section(&ns.lines, StandardSection::Todo);
    let (lines, _) = append_line_to_section(&lines, &section, &format!("- [ ] {text}{suffix}"));
    let mut ns = ns;
    ns.lines = lines;
    ns.message = String::new(); // cursor stays
    ns
}

fn add_subsection(state: &EditorState, name: &str) -> EditorState {
    let Some(level) = nearest_heading_level(&state.lines, state.cursor.line) else {
        let mut ns = state.clone();
        ns.message = "No enclosing heading".to_string();
        return ns;
    };
    if level >= 6 {
        let mut ns = state.clone();
        ns.message = "Max heading depth".to_string();
        return ns;
    }
    let ns = push_undo(state);
    let heading = format!("{} {name}", "#".repeat((level + 1) as usize));
    let insert_at = end_of_enclosing_section(&ns.lines, ns.cursor.line, level);
    let mut ns = ns;
    ns.lines.insert(insert_at, String::new());
    ns.lines.insert(insert_at, heading);
    ns.cursor = Cursor {
        line: insert_at,
        col: 0,
    };
    ns.message = String::new();
    clamp_cursor(&ns)
}

/// Which kind of block `:scheduled`/`:purpose`/`:start`/`:end` (meeting) or `:topic`
/// (note) requires the cursor to be inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequiredContext {
    Meeting,
    Note,
}

fn set_meta(state: &EditorState, required: RequiredContext, key: &str, value: &str) -> EditorState {
    let model = scan_document(&state.lines);
    let block = match (required, resolve_context(&model, state.cursor.line)) {
        (RequiredContext::Meeting, Context::Meeting { block, .. }) => block,
        (RequiredContext::Note, Context::Note { block, .. }) => block,
        _ => {
            let mut ns = state.clone();
            ns.message = match required {
                RequiredContext::Meeting => "Not in a meeting".to_string(),
                RequiredContext::Note => "Not in a note".to_string(),
            };
            return ns;
        }
    };
    let ns = push_undo(state);
    let (lines, _) = upsert_meta(&ns.lines, &block, key, value);
    let mut ns = ns;
    ns.lines = lines;
    ns.message = String::new();
    ns
}

fn people(base: &EditorState, arg: &str) -> EditorState {
    let model = scan_document(&base.lines);
    let block = match resolve_context(&model, base.cursor.line) {
        Context::Meeting { block, .. } | Context::Note { block, .. } => block,
        _ => {
            let mut ns = base.clone();
            ns.message = "Not in a meeting or note".to_string();
            return ns;
        }
    };
    let ns = push_undo(base);
    let (lines, _) = append_meta(&ns.lines, &block, "people", arg);
    let mut ns = ns;
    ns.lines = lines;
    ns.message = String::new();
    ns
}

/// Run the command currently typed into `state.command` (colon excluded). Always clears
/// `command` back to `None` — on success *and* on a validation error, matching the web's
/// "close the palette either way, leave the error in `message`" behavior. Port of
/// `web/src/lib/editor/commands.ts` `runCommand`.
pub fn run_command(state: &EditorState, ctx: &CommandCtx) -> CommandResult {
    let mut base = state.clone();
    base.command = None;
    let typed = state.command.as_deref().unwrap_or("");

    let (command, arg) = match validate_command(typed) {
        ValidationResult::Err { error } => {
            base.message = error;
            return CommandResult {
                state: base,
                effect: None,
            };
        }
        ValidationResult::Ok { command, arg } => (command, arg),
    };

    match command {
        CommandName::Goto => {
            base.message = String::new();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Goto(arg)),
            }
        }
        CommandName::Today => {
            base.message = String::new();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Today),
            }
        }
        CommandName::Tab => {
            base.message = String::new();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Tab(arg)),
            }
        }
        CommandName::Close => {
            base.message = String::new();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Close),
            }
        }
        CommandName::W => {
            base.message = "Written".to_string();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Save),
            }
        }
        CommandName::Theme => {
            base.message = String::new();
            CommandResult {
                state: base,
                effect: Some(AppEffect::Theme(arg)),
            }
        }
        CommandName::Meeting => CommandResult {
            state: add_block(&base, StandardSection::Meetings, &arg),
            effect: None,
        },
        CommandName::Note => CommandResult {
            state: add_block(&base, StandardSection::Notes, &arg),
            effect: None,
        },
        CommandName::Todo => CommandResult {
            state: add_todo(&base, &arg),
            effect: None,
        },
        CommandName::Section => CommandResult {
            state: add_subsection(&base, &arg),
            effect: None,
        },
        CommandName::Scheduled => CommandResult {
            state: set_meta(&base, RequiredContext::Meeting, "scheduled", &arg),
            effect: None,
        },
        CommandName::Purpose => CommandResult {
            state: set_meta(&base, RequiredContext::Meeting, "purpose", &arg),
            effect: None,
        },
        CommandName::Start => CommandResult {
            state: set_meta(&base, RequiredContext::Meeting, "started", &ctx.now_hhmm),
            effect: None,
        },
        CommandName::End => CommandResult {
            state: set_meta(&base, RequiredContext::Meeting, "ended", &ctx.now_hhmm),
            effect: None,
        },
        CommandName::Topic => CommandResult {
            state: set_meta(&base, RequiredContext::Note, "topic", &arg),
            effect: None,
        },
        CommandName::People => CommandResult {
            state: people(&base, &arg),
            effect: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::state::create_editor_state;

    const TEMPLATE: [&str; 8] = [
        "# 2026-06-23-TUE",
        "",
        "## To Do",
        "",
        "## Meetings",
        "",
        "## Notes",
        "",
    ];

    fn lines(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn append_block_adds_an_h3_at_the_end_of_a_section() {
        let tpl = lines(&TEMPLATE);
        let meetings = scan_document(&tpl)
            .sections
            .into_iter()
            .find(|s| s.kind == SectionKind::Meetings)
            .unwrap();
        let (out, heading_index) = append_block(&tpl, &meetings, "### Sync");
        assert_eq!(out[heading_index], "### Sync");
    }

    #[test]
    fn append_line_to_section_adds_after_the_last_non_blank_line() {
        let tpl = lines(&TEMPLATE);
        let todo = scan_document(&tpl)
            .sections
            .into_iter()
            .find(|s| s.kind == SectionKind::Todo)
            .unwrap();
        let (out, index) = append_line_to_section(&tpl, &todo, "- [ ] Buy milk");
        assert_eq!(out[index], "- [ ] Buy milk");
        let meetings_at = out.iter().position(|l| l == "## Meetings").unwrap();
        assert!(meetings_at > index);
    }

    #[test]
    fn upsert_meta_inserts_then_updates_in_place() {
        let mut ls = lines(&["## Meetings", "### Sync", ""]);
        let mut block = scan_document(&ls).sections[0].blocks[0].clone();
        let (out, _) = upsert_meta(&ls, &block, "scheduled", "14:30");
        ls = out;
        assert!(ls.contains(&"meta:scheduled 14:30".to_string()));
        block = scan_document(&ls).sections[0].blocks[0].clone();
        let (out, _) = upsert_meta(&ls, &block, "scheduled", "15:00");
        ls = out;
        assert_eq!(
            ls.iter()
                .filter(|l| l.starts_with("meta:scheduled"))
                .count(),
            1
        );
        assert!(ls.contains(&"meta:scheduled 15:00".to_string()));
    }

    #[test]
    fn ensure_section_recreates_a_missing_section_in_canonical_order() {
        let (out, section) = ensure_section(
            &lines(&["# T", "", "## Notes", ""]),
            StandardSection::Meetings,
        );
        let kinds: Vec<SectionKind> = scan_document(&out)
            .sections
            .iter()
            .map(|s| s.kind)
            .collect();
        let meetings_idx = kinds
            .iter()
            .position(|k| *k == SectionKind::Meetings)
            .unwrap();
        let notes_idx = kinds.iter().position(|k| *k == SectionKind::Notes).unwrap();
        assert!(meetings_idx < notes_idx);
        assert_eq!(section.kind, SectionKind::Meetings);
    }

    #[test]
    fn end_of_enclosing_section_finds_the_next_same_or_shallower_heading() {
        assert_eq!(
            end_of_enclosing_section(&lines(&["### A", "body", "### B"]), 1, 3),
            2
        );
    }

    #[test]
    fn append_meta_inserts_meta_people_when_no_prior_value_exists() {
        let ls = lines(&["## Meetings", "### Sync", ""]);
        let block = scan_document(&ls).sections[0].blocks[0].clone();
        let (out, _) = append_meta(&ls, &block, "people", "Alice");
        assert!(out.contains(&"meta:people Alice".to_string()));
    }

    #[test]
    fn append_meta_appends_comma_separated_to_an_existing_value() {
        let ls = lines(&["## Meetings", "### Sync", "meta:people Alice", ""]);
        let block = scan_document(&ls).sections[0].blocks[0].clone();
        let (out, _) = append_meta(&ls, &block, "people", "Bob");
        assert!(out.contains(&"meta:people Alice, Bob".to_string()));
        assert_eq!(
            out.iter().filter(|l| l.starts_with("meta:people")).count(),
            1
        );
    }

    #[test]
    fn append_meta_trims_whitespace_from_the_new_value_before_appending() {
        let ls = lines(&["## Meetings", "### Sync", "meta:people Alice", ""]);
        let block = scan_document(&ls).sections[0].blocks[0].clone();
        let (out, _) = append_meta(&ls, &block, "people", "  Bob  ");
        assert!(out.contains(&"meta:people Alice, Bob".to_string()));
    }

    fn with_cmd(raw: &[&str], cmd: &str, line: usize) -> EditorState {
        let mut s = create_editor_state(lines(raw), Vec::new());
        s.command = Some(cmd.to_string());
        s.cursor = Cursor { line, col: 0 };
        s
    }

    fn ctx() -> CommandCtx {
        CommandCtx {
            now_hhmm: "09:30".to_string(),
        }
    }

    #[test]
    fn reports_unknown_commands_and_clears_the_command_line() {
        let r = run_command(&with_cmd(&TEMPLATE, "meetng x", 0), &ctx());
        assert_eq!(r.state.command, None);
        assert!(r.state.message.contains("Unknown command"));
    }

    #[test]
    fn meeting_adds_a_heading_and_moves_the_cursor_to_it() {
        let r = run_command(&with_cmd(&TEMPLATE, "meeting Daily Standup", 0), &ctx());
        assert_eq!(r.state.lines[r.state.cursor.line], "### Daily Standup");
    }

    #[test]
    fn todo_appends_to_to_do_and_keeps_the_cursor() {
        let r = run_command(&with_cmd(&TEMPLATE, "todo Buy milk", 0), &ctx());
        assert!(r.state.lines.contains(&"- [ ] Buy milk".to_string()));
        assert_eq!(r.state.cursor.line, 0);
    }

    #[test]
    fn todo_inside_a_meeting_tags_the_meeting_name() {
        let raw = [
            "# T",
            "",
            "## To Do",
            "",
            "## Meetings",
            "### Sync",
            "",
            "## Notes",
            "",
        ];
        let r = run_command(&with_cmd(&raw, "todo Prep", 5), &ctx());
        assert!(r.state.lines.iter().any(|l| l == "- [ ] Prep _(Sync)_"));
    }

    #[test]
    fn scheduled_errors_when_not_in_a_meeting() {
        let r = run_command(&with_cmd(&TEMPLATE, "scheduled 14:30", 2), &ctx());
        assert_eq!(r.state.message, "Not in a meeting");
    }

    #[test]
    fn start_records_the_current_time_on_the_enclosing_meeting() {
        let raw = ["# T", "", "## Meetings", "### Sync", "", "## Notes", ""];
        let r = run_command(&with_cmd(&raw, "start", 3), &ctx());
        assert!(r.state.lines.contains(&"meta:started 09:30".to_string()));
    }

    #[test]
    fn section_nests_one_level_deeper_than_the_enclosing_heading() {
        let raw = ["## Meetings", "### Sync", "body", "## Notes", ""];
        let r = run_command(&with_cmd(&raw, "section Risks", 2), &ctx());
        assert!(r.state.lines.iter().any(|l| l == "#### Risks"));
    }

    #[test]
    fn goto_emits_an_effect_without_mutating_the_buffer() {
        let r = run_command(&with_cmd(&TEMPLATE, "goto 2026-06-01", 0), &ctx());
        assert_eq!(r.effect, Some(AppEffect::Goto("2026-06-01".to_string())));
        assert_eq!(r.state.lines, lines(&TEMPLATE));
    }

    #[test]
    fn people_sets_meta_people_in_a_meeting_block() {
        let raw = ["# T", "", "## Meetings", "### Sync", "", "## Notes", ""];
        let r = run_command(&with_cmd(&raw, "people Alice", 4), &ctx());
        assert!(r.state.lines.contains(&"meta:people Alice".to_string()));
        assert_eq!(r.state.message, "");
    }

    #[test]
    fn people_appends_to_existing_meta_people_in_a_meeting_block() {
        let raw = [
            "# T",
            "",
            "## Meetings",
            "### Sync",
            "meta:people Alice",
            "",
            "## Notes",
            "",
        ];
        let r = run_command(&with_cmd(&raw, "people Bob", 5), &ctx());
        assert!(
            r.state
                .lines
                .contains(&"meta:people Alice, Bob".to_string())
        );
    }

    #[test]
    fn people_sets_meta_people_in_a_note_block() {
        let raw = [
            "# T",
            "",
            "## Meetings",
            "",
            "## Notes",
            "### Retro",
            "",
            "",
        ];
        let r = run_command(&with_cmd(&raw, "people Alice", 6), &ctx());
        assert!(r.state.lines.contains(&"meta:people Alice".to_string()));
        assert_eq!(r.state.message, "");
    }

    #[test]
    fn people_errors_when_not_in_a_meeting_or_note_block() {
        // TEMPLATE cursor line 2 = "## To Do" section heading, not inside any block.
        let r = run_command(&with_cmd(&TEMPLATE, "people Alice", 2), &ctx());
        assert!(r.state.message.contains("meeting or note"));
    }

    #[test]
    fn p_shortcut_works_end_to_end_through_run_command() {
        let raw = ["# T", "", "## Meetings", "### Sync", "", "## Notes", ""];
        let r = run_command(&with_cmd(&raw, "p Alice", 4), &ctx());
        assert!(r.state.lines.contains(&"meta:people Alice".to_string()));
    }
}
