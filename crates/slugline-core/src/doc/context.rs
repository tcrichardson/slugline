use super::scan::{Block, DocModel, Section, SectionKind};

/// What the cursor's current line is "inside", for commands that write relative to it
/// (`:scheduled`, `:todo`'s meeting-tag suffix, `:section`'s nesting level, ...). Port of
/// `web/src/lib/doc/context.ts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Context {
    None,
    Todo { section: Section },
    Meeting { block: Block, section: Section },
    Note { block: Block, section: Section },
    Other { section: Section },
}

/// Which top-level section (and, for Meetings/Notes, which H3 block) `line_index` falls
/// inside. Never panics: a `line_index` outside every section yields `Context::None`.
pub fn resolve_context(model: &DocModel, line_index: usize) -> Context {
    let Some(section) = model
        .sections
        .iter()
        .find(|s| line_index >= s.start_line && line_index <= s.end_line)
    else {
        return Context::None;
    };

    match section.kind {
        SectionKind::Todo => Context::Todo {
            section: section.clone(),
        },
        SectionKind::Meetings | SectionKind::Notes => {
            let block = section
                .blocks
                .iter()
                .find(|b| line_index >= b.start_line && line_index <= b.end_line);
            match (section.kind, block) {
                (SectionKind::Meetings, Some(b)) => Context::Meeting {
                    block: b.clone(),
                    section: section.clone(),
                },
                (SectionKind::Notes, Some(b)) => Context::Note {
                    block: b.clone(),
                    section: section.clone(),
                },
                _ => Context::Other {
                    section: section.clone(),
                },
            }
        }
        SectionKind::Other => Context::Other {
            section: section.clone(),
        },
    }
}

/// The level (1-6) of the nearest heading at or above `line_index`, or `None` if there
/// is no heading above it. Used by `:section` to nest one level deeper than whatever
/// heading encloses the cursor.
pub fn nearest_heading_level(lines: &[String], line_index: usize) -> Option<u8> {
    if lines.is_empty() {
        return None;
    }
    let start = line_index.min(lines.len() - 1);
    for i in (0..=start).rev() {
        if let super::classify::Line::Heading { level, .. } =
            super::classify::classify_line(&lines[i])
        {
            return Some(level);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::scan_document;

    fn fixture_lines(name: &str) -> Vec<String> {
        let path = format!("{}/../../fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"))
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn returns_the_meeting_when_the_cursor_is_inside_an_h3_under_meetings() {
        let lines = fixture_lines("full-day.md");
        let model = scan_document(&lines);
        let meetings = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Meetings)
            .unwrap();
        let sync = &meetings.blocks[0];
        match resolve_context(&model, sync.heading_line_index + 1) {
            Context::Meeting { block, .. } => assert_eq!(block.name, "Weekly Sync"),
            other => panic!("expected Context::Meeting, got {other:?}"),
        }
    }

    #[test]
    fn returns_the_note_when_the_cursor_is_inside_an_h3_under_notes() {
        let lines = fixture_lines("full-day.md");
        let model = scan_document(&lines);
        let notes = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Notes)
            .unwrap();
        let arch = &notes.blocks[0];
        match resolve_context(&model, arch.heading_line_index + 1) {
            Context::Note { .. } => {}
            other => panic!("expected Context::Note, got {other:?}"),
        }
    }

    #[test]
    fn returns_todo_when_the_cursor_is_inside_the_to_do_section() {
        let lines = fixture_lines("full-day.md");
        let model = scan_document(&lines);
        let todo = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Todo)
            .unwrap();
        match resolve_context(&model, todo.heading_line_index + 1) {
            Context::Todo { .. } => {}
            other => panic!("expected Context::Todo, got {other:?}"),
        }
    }

    #[test]
    fn returns_none_when_the_cursor_is_on_the_title_line() {
        let lines = fixture_lines("full-day.md");
        let model = scan_document(&lines);
        assert_eq!(resolve_context(&model, 0), Context::None);
    }

    #[test]
    fn nearest_heading_level_finds_the_level_of_the_nearest_enclosing_heading() {
        let lines = fixture_lines("subsections.md");
        let idx = lines.iter().position(|l| l == "Cut scope.").unwrap();
        // Inside the "Mitigations" (H5) area -> nearest heading level is 5.
        assert_eq!(nearest_heading_level(&lines, idx), Some(5));
    }

    #[test]
    fn nearest_heading_level_returns_none_above_any_heading() {
        let lines = vec![String::new(), "no heading yet".to_string()];
        assert_eq!(nearest_heading_level(&lines, 1), None);
    }
}
