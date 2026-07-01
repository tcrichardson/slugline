use std::sync::LazyLock;

use regex::Regex;

/// A classified line. Mirrors `web/src/lib/doc/types.ts` `ClassifiedLine` as a sum type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Blank,
    Heading { level: u8, text: String },
    Task { done: bool, text: String },
    Meta { key: String, text: String },
    List { ordered: bool, number: Option<u32>, depth: usize, text: String },
    Blockquote { text: String },
    Paragraph { text: String },
}

static HEADING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(#{1,6})\s+(.*)$").unwrap());
static TASK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^- \[([ xX])\]\s?(.*)$").unwrap());
static META: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^meta:(\S+)(?: (.*))?$").unwrap());
static UL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\s*)[-*+]\s+(.*)$").unwrap());
static OL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\s*)(\d+)\.\s+(.*)$").unwrap());
static BLOCKQUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^>\s?(.*)$").unwrap());

pub fn classify_line(raw: &str) -> Line {
    if raw.trim().is_empty() {
        return Line::Blank;
    }
    if let Some(c) = HEADING.captures(raw) {
        return Line::Heading { level: c[1].len() as u8, text: c[2].trim().to_string() };
    }
    if let Some(c) = TASK.captures(raw) {
        return Line::Task { done: c[1].eq_ignore_ascii_case("x"), text: c[2].to_string() };
    }
    if let Some(c) = META.captures(raw) {
        return Line::Meta {
            key: c[1].to_string(),
            text: c.get(2).map_or("", |m| m.as_str()).trim().to_string(),
        };
    }
    if let Some(c) = BLOCKQUOTE.captures(raw) {
        return Line::Blockquote { text: c[1].to_string() };
    }
    if let Some(c) = UL.captures(raw) {
        return Line::List { ordered: false, number: None, depth: c[1].len() / 2, text: c[2].to_string() };
    }
    if let Some(c) = OL.captures(raw) {
        return Line::List { ordered: true, number: c[2].parse().ok(), depth: c[1].len() / 2, text: c[3].to_string() };
    }
    Line::Paragraph { text: raw.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_and_heading() {
        assert_eq!(classify_line("   "), Line::Blank);
        assert_eq!(
            classify_line("## Morning"),
            Line::Heading { level: 2, text: "Morning".into() }
        );
    }

    #[test]
    fn tasks_and_meta() {
        assert_eq!(
            classify_line("- [x] done it"),
            Line::Task { done: true, text: "done it".into() }
        );
        assert_eq!(
            classify_line("- [ ] todo"),
            Line::Task { done: false, text: "todo".into() }
        );
        assert_eq!(
            classify_line("meta:scheduled 09:00"),
            Line::Meta { key: "scheduled".into(), text: "09:00".into() }
        );
    }

    #[test]
    fn lists_blockquote_paragraph() {
        assert_eq!(
            classify_line("  - nested"),
            Line::List { ordered: false, number: None, depth: 1, text: "nested".into() }
        );
        assert_eq!(
            classify_line("3. third"),
            Line::List { ordered: true, number: Some(3), depth: 0, text: "third".into() }
        );
        assert_eq!(classify_line("> quote"), Line::Blockquote { text: "quote".into() });
        assert_eq!(classify_line("just text"), Line::Paragraph { text: "just text".into() });
    }

    #[test]
    fn meta_empty_value() {
        assert_eq!(
            classify_line("meta:purpose"),
            Line::Meta { key: "purpose".into(), text: "".into() }
        );
    }

    #[test]
    fn alternate_bullets() {
        assert_eq!(
            classify_line("* star bullet"),
            Line::List { ordered: false, number: None, depth: 0, text: "star bullet".into() }
        );
        assert_eq!(
            classify_line("+ plus bullet"),
            Line::List { ordered: false, number: None, depth: 0, text: "plus bullet".into() }
        );
    }

    #[test]
    fn ordered_list_depth() {
        assert_eq!(
            classify_line("  1. sub numbered"),
            Line::List { ordered: true, number: Some(1), depth: 1, text: "sub numbered".into() }
        );
        assert_eq!(
            classify_line("    3. deep numbered"),
            Line::List { ordered: true, number: Some(3), depth: 2, text: "deep numbered".into() }
        );
    }

    #[test]
    fn blockquote_without_space() {
        assert_eq!(
            classify_line(">no space"),
            Line::Blockquote { text: "no space".into() }
        );
    }

    #[test]
    fn bare_blockquote() {
        assert_eq!(
            classify_line(">"),
            Line::Blockquote { text: "".into() }
        );
    }

    #[test]
    fn uppercase_x_task() {
        assert_eq!(
            classify_line("- [X] Send invoice"),
            Line::Task { done: true, text: "Send invoice".into() }
        );
    }
}
