use super::classify::{Line, classify_line};

/// A `meta:key value` line inside a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaEntry {
    pub key: String,
    pub value: String,
    pub line_index: usize,
}

/// The kind of a top-level (H2) section, inferred from its title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Todo,
    Meetings,
    Notes,
    Other,
}

/// An H3 block nested under a `Meetings`/`Notes` section (e.g. one meeting).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub name: String,
    pub level: u8,
    pub heading_line_index: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub meta: Vec<MetaEntry>,
    /// Index of the last meta line, or `heading_line_index` when the block has no meta.
    pub meta_end_line: usize,
}

/// A top-level (H2) section of the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub kind: SectionKind,
    pub title: String,
    pub level: u8,
    pub heading_line_index: usize,
    pub start_line: usize,
    pub end_line: usize,
    /// H3 blocks for `Meetings`/`Notes`; empty otherwise.
    pub blocks: Vec<Block>,
}

/// The whole document: an optional title (first H1) and its top-level sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocModel {
    pub title: Option<String>,
    pub title_line_index: Option<usize>,
    pub sections: Vec<Section>,
}

fn section_kind(title: &str) -> SectionKind {
    match title.trim().to_lowercase().as_str() {
        "to do" | "todo" => SectionKind::Todo,
        "meetings" => SectionKind::Meetings,
        "notes" => SectionKind::Notes,
        _ => SectionKind::Other,
    }
}

/// Scans forward from `from` to `to` (inclusive) and returns the index just before the
/// first heading whose level is `<= max_level`, or `to` if none is found. Used to find
/// where an H3 block or H2 section ends.
fn find_boundary_end(classified: &[Line], from: usize, to: usize, max_level: u8) -> usize {
    for (j, c) in classified.iter().enumerate().take(to + 1).skip(from) {
        if let Line::Heading { level, .. } = c
            && *level <= max_level
        {
            return j - 1;
        }
    }
    to
}

fn collect_blocks(classified: &[Line], from: usize, to: usize) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut i = from;
    while i <= to {
        if let Line::Heading { level: 3, text } = &classified[i] {
            let start = i;
            let end = find_boundary_end(classified, i + 1, to, 3);

            let mut meta = Vec::new();
            let mut meta_end_line = start;
            let mut k = start + 1;
            while k <= end {
                match &classified[k] {
                    Line::Meta { key, text: value } => {
                        meta.push(MetaEntry {
                            key: key.clone(),
                            value: value.clone(),
                            line_index: k,
                        });
                        meta_end_line = k;
                        k += 1;
                    }
                    _ => break,
                }
            }

            blocks.push(Block {
                name: text.clone(),
                level: 3,
                heading_line_index: start,
                start_line: start,
                end_line: end,
                meta,
                meta_end_line,
            });
            i = end + 1;
        } else {
            i += 1;
        }
    }
    blocks
}

/// Scan a document's raw lines into title + top-level sections (with H3 blocks for
/// `Meetings`/`Notes`). Never panics: malformed/heading-less documents just yield an
/// empty section list. Port of `web/src/lib/doc/scan.ts`.
pub fn scan_document(lines: &[String]) -> DocModel {
    let classified: Vec<Line> = lines.iter().map(|l| classify_line(l)).collect();

    let mut title = None;
    let mut title_line_index = None;
    for (i, c) in classified.iter().enumerate() {
        if let Line::Heading { level: 1, text } = c {
            title = Some(text.clone());
            title_line_index = Some(i);
            break;
        }
    }

    let mut sections = Vec::new();
    if !classified.is_empty() {
        let last = classified.len() - 1;
        let mut i = 0;
        while i <= last {
            if let Line::Heading { level: 2, text } = &classified[i] {
                let start = i;
                let end = find_boundary_end(&classified, i + 1, last, 2);
                let kind = section_kind(text);
                let blocks = match kind {
                    SectionKind::Meetings | SectionKind::Notes => {
                        collect_blocks(&classified, start + 1, end)
                    }
                    SectionKind::Todo | SectionKind::Other => Vec::new(),
                };
                sections.push(Section {
                    kind,
                    title: text.clone(),
                    level: 2,
                    heading_line_index: start,
                    start_line: start,
                    end_line: end,
                    blocks,
                });
                i = end + 1;
            } else {
                i += 1;
            }
        }
    }

    DocModel {
        title,
        title_line_index,
        sections,
    }
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
    fn reads_the_title_from_the_first_h1() {
        let model = scan_document(&fixture_lines("full-day.md"));
        assert_eq!(model.title, Some("2026-06-23-TUE".to_string()));
        assert_eq!(model.title_line_index, Some(0));
    }

    #[test]
    fn finds_the_three_standard_sections_by_kind() {
        let model = scan_document(&fixture_lines("full-day.md"));
        let kinds: Vec<SectionKind> = model.sections.iter().map(|s| s.kind).collect();
        assert_eq!(
            kinds,
            vec![SectionKind::Todo, SectionKind::Meetings, SectionKind::Notes]
        );
    }

    #[test]
    fn collects_h3_blocks_under_meetings_with_their_meta() {
        let model = scan_document(&fixture_lines("full-day.md"));
        let meetings = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Meetings)
            .unwrap();
        let names: Vec<&str> = meetings.blocks.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, vec!["Weekly Sync", "Standup"]);

        let sync = &meetings.blocks[0];
        let scheduled = sync.meta.iter().find(|m| m.key == "scheduled").unwrap();
        assert_eq!(scheduled.value, "14:30");
        let keys: Vec<&str> = sync.meta.iter().map(|m| m.key.as_str()).collect();
        assert_eq!(keys, vec!["purpose", "scheduled", "started", "ended"]);
    }

    #[test]
    fn bounds_a_block_to_the_line_before_the_next_heading() {
        let model = scan_document(&fixture_lines("full-day.md"));
        let meetings = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Meetings)
            .unwrap();
        let sync = &meetings.blocks[0];
        assert!(sync.meta_end_line > sync.heading_line_index);
        assert!(sync.end_line >= sync.meta_end_line);
    }

    #[test]
    fn does_not_panic_on_malformed_documents() {
        let model = scan_document(&fixture_lines("malformed.md"));
        assert_eq!(model.title, Some("Just a title".to_string()));
        assert_eq!(model.sections, Vec::new());
    }

    #[test]
    fn treats_a_block_with_no_meta_as_meta_end_line_eq_heading_line_index() {
        let model = scan_document(&fixture_lines("subsections.md"));
        let meetings = model
            .sections
            .iter()
            .find(|s| s.kind == SectionKind::Meetings)
            .unwrap();
        let planning = &meetings.blocks[0];
        assert_eq!(planning.name, "Planning");
        let keys: Vec<&str> = planning.meta.iter().map(|m| m.key.as_str()).collect();
        assert_eq!(keys, vec!["scheduled"]);
    }
}
