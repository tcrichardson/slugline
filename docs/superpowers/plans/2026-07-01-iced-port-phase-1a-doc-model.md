# Phase 1a — Document Model Port (`classify` + `render_inline`) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **This is a port.** The authoritative behavioral spec for each module is the corresponding
> TypeScript file **and its `*.test.ts`** under `web/src/lib/doc/`. Each task gives the full Rust
> implementation plus representative ported tests, and names the exact `.test.ts` file to translate
> in full. "Port the remaining cases from `web/src/lib/doc/classify.test.ts`" is a concrete
> instruction against an existing artifact — not a placeholder.

**Goal:** Port the two pure functions the editor pane needs to render pretty lines — `classify_line` (line → structured `Line`) and `render_inline` (inline markdown → `Vec<Span>`) — into `slugline-core`, fully unit-tested and headless.

**Architecture:** A new `doc` module in `slugline-core` with no UI dependency. `classify` uses `regex`; `render_inline` is a hand-written single-pass scanner that emits structured spans (replacing the TS HTML-string output, which is a better fit for Iced `rich_text` and for testing).

**Tech Stack:** Rust, `regex` (line classification), `std::sync::LazyLock` (compiled-regex statics).

---

## File Structure (files added/changed in Phase 1a)

```
crates/slugline-core/
  Cargo.toml                       # + regex dependency
  src/
    lib.rs                         # + pub mod doc;
    doc/
      mod.rs                       # re-exports classify_line/Line, render_inline/Span
      classify.rs                  # port of web/src/lib/doc/classify.ts (+ types.ts subset)
      render_inline.rs             # port of web/src/lib/doc/renderInline.ts -> Vec<Span>
```

---

### Task 1: Port the line classifier — `classify_line`

**Files:**
- Modify: `crates/slugline-core/Cargo.toml`, `crates/slugline-core/src/lib.rs`
- Create: `crates/slugline-core/src/doc/mod.rs`, `crates/slugline-core/src/doc/classify.rs`

- [ ] **Step 1: Add the `regex` dependency** — in `crates/slugline-core/Cargo.toml` under `[dependencies]`:

```toml
regex = "1"
```

- [ ] **Step 2: Declare the module** — add to `crates/slugline-core/src/lib.rs`:

```rust
pub mod doc;
```

and create `crates/slugline-core/src/doc/mod.rs`:

```rust
pub mod classify;
pub mod render_inline;

pub use classify::{classify_line, Line};
pub use render_inline::{render_inline, Span};
```

- [ ] **Step 3: Write the failing test** — `crates/slugline-core/src/doc/classify.rs`:

```rust
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

pub fn classify_line(raw: &str) -> Line {
    todo!()
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
}
```

- [ ] **Step 4: Run test to verify it fails** — `cargo test -p slugline-core doc::classify`
Expected: FAIL (`todo!()` panic).

- [ ] **Step 5: Implement `classify_line`** (replace the `todo!()` body; keep the regexes as module statics above it):

```rust
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
```

- [ ] **Step 6: Run test to verify it passes** — `cargo test -p slugline-core doc::classify`
Expected: PASS (3 tests).

- [ ] **Step 7: Port the remaining cases** from `web/src/lib/doc/classify.test.ts` into the `tests` module (ordered-list depth by indentation, `meta:` with no value, uppercase `X` tasks, blockquote without a space, `*`/`+` bullets). Run `cargo test -p slugline-core doc::classify` until green.

- [ ] **Step 8: Commit**

```bash
git add crates/slugline-core/
git commit -m "feat(core): port classify_line to a Rust Line enum"
```

---

### Task 2: Port the inline renderer → `Vec<Span>`

**Files:** Create `crates/slugline-core/src/doc/render_inline.rs`

Note: the TS version emits an HTML string via sequential regex passes. The Rust port emits a
**structured span list** (better for Iced and for tests). We hand-write a single-pass scanner: each
delimiter run yields one styled `Span`. Fidelity is pinned by porting `renderInline.test.ts` in
Step 5, rewriting each HTML assertion as a span assertion.

- [ ] **Step 1: Write the failing test** — `crates/slugline-core/src/doc/render_inline.rs`:

```rust
/// One styled run of inline text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub strike: bool,
    pub highlight: bool,
    pub code: bool,
    pub link: Option<String>,
}

impl Span {
    fn plain(text: impl Into<String>) -> Self {
        Span { text: text.into(), ..Default::default() }
    }
}

pub fn render_inline(text: &str) -> Vec<Span> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_is_one_span() {
        assert_eq!(render_inline("hello world"), vec![Span::plain("hello world")]);
    }

    #[test]
    fn bold_and_code() {
        assert_eq!(
            render_inline("a **b** c"),
            vec![
                Span::plain("a "),
                Span { text: "b".into(), bold: true, ..Default::default() },
                Span::plain(" c"),
            ]
        );
        assert_eq!(
            render_inline("x `code` y"),
            vec![
                Span::plain("x "),
                Span { text: "code".into(), code: true, ..Default::default() },
                Span::plain(" y"),
            ]
        );
    }

    #[test]
    fn link_extracts_label_and_url() {
        assert_eq!(
            render_inline("see [docs](https://example.com) ok"),
            vec![
                Span::plain("see "),
                Span { text: "docs".into(), link: Some("https://example.com".into()), ..Default::default() },
                Span::plain(" ok"),
            ]
        );
    }
}
```

- [ ] **Step 2: Run test to verify it fails** — `cargo test -p slugline-core doc::render_inline`
Expected: FAIL (`todo!()`).

- [ ] **Step 3: Implement the scanner** (replace `todo!()` and add the helpers below it):

```rust
const SAFE_SCHEMES: [&str; 3] = ["https:", "http:", "mailto:"];

pub fn render_inline(text: &str) -> Vec<Span> {
    let chars: Vec<char> = text.chars().collect();
    let mut spans: Vec<Span> = Vec::new();
    let mut plain = String::new();
    let mut i = 0;

    macro_rules! flush_plain {
        () => {
            if !plain.is_empty() {
                spans.push(Span::plain(std::mem::take(&mut plain)));
            }
        };
    }

    while i < chars.len() {
        // Code span: `...` (protects its contents from further parsing).
        if chars[i] == '`' {
            if let Some(end) = find(&chars, i + 1, '`') {
                flush_plain!();
                spans.push(Span { text: chars[i + 1..end].iter().collect(), code: true, ..Default::default() });
                i = end + 1;
                continue;
            }
        }
        // Link: [label](url), safe schemes only.
        if chars[i] == '[' {
            if let Some((label, url, next)) = parse_link(&chars, i) {
                if SAFE_SCHEMES.iter().any(|s| url.starts_with(s)) {
                    flush_plain!();
                    spans.push(Span { text: label, link: Some(url), ..Default::default() });
                    i = next;
                    continue;
                }
            }
        }
        // Emphasis, longest delimiter first.
        if let Some((delim, bold, italic, strike, highlight)) = emphasis_at(&chars, i) {
            let open = i + delim.len();
            if let Some(close) = find_seq(&chars, open, delim) {
                flush_plain!();
                spans.push(Span {
                    text: chars[open..close].iter().collect(),
                    bold, italic, strike, highlight,
                    ..Default::default()
                });
                i = close + delim.len();
                continue;
            }
        }
        plain.push(chars[i]);
        i += 1;
    }
    flush_plain!();
    if spans.is_empty() {
        spans.push(Span::plain(String::new()));
    }
    spans
}

fn find(chars: &[char], from: usize, target: char) -> Option<usize> {
    (from..chars.len()).find(|&j| chars[j] == target)
}

fn find_seq(chars: &[char], from: usize, seq: &[char]) -> Option<usize> {
    if seq.is_empty() || from + seq.len() > chars.len() {
        return None;
    }
    (from..=chars.len() - seq.len()).find(|&j| &chars[j..j + seq.len()] == seq)
}

/// (delimiter, bold, italic, strike, highlight) if an emphasis run opens at `i`.
fn emphasis_at(chars: &[char], i: usize) -> Option<(&'static [char], bool, bool, bool, bool)> {
    let two = |a: char| i + 1 < chars.len() && chars[i] == a && chars[i + 1] == a;
    if two('*') { return Some((&['*', '*'], true, false, false, false)); }
    if two('~') { return Some((&['~', '~'], false, false, true, false)); }
    if two('=') { return Some((&['=', '='], false, false, false, true)); }
    if chars[i] == '*' || chars[i] == '_' {
        let d: &'static [char] = if chars[i] == '*' { &['*'] } else { &['_'] };
        return Some((d, false, true, false, false));
    }
    None
}

fn parse_link(chars: &[char], i: usize) -> Option<(String, String, usize)> {
    let close_br = find(chars, i + 1, ']')?;
    if close_br + 1 >= chars.len() || chars[close_br + 1] != '(' {
        return None;
    }
    let close_par = find(chars, close_br + 2, ')')?;
    let label: String = chars[i + 1..close_br].iter().collect();
    let url: String = chars[close_br + 2..close_par].iter().collect();
    if url.chars().any(char::is_whitespace) {
        return None;
    }
    Some((label, url, close_par + 1))
}
```

- [ ] **Step 4: Run test to verify it passes** — `cargo test -p slugline-core doc::render_inline`
Expected: PASS (3 tests).

- [ ] **Step 5: Port the remaining cases** from `web/src/lib/doc/renderInline.test.ts`, rewriting each HTML assertion as a span-list assertion: `~~strike~~`, `==highlight==`, `_italic_`, unsafe-scheme links left as literal text (a plain span containing the original `[..](..)`), and code spans protecting their contents. Run until green.

- [ ] **Step 6: Commit**

```bash
git add crates/slugline-core/src/doc/render_inline.rs
git commit -m "feat(core): port renderInline to a structured Vec<Span>"
```

---

## Self-Review (performed while writing this plan)

- **Spec coverage:** Implements the `doc` half of design Section 3 — `classify_line → Line` and the
  `render_inline → Vec<Span>` split the editor pane consumes in Phase 1c.
- **Type consistency:** `Line` and `Span` are defined once here and re-exported from `doc/mod.rs`;
  Phase 1c's editor pane imports exactly these names.
- **Placeholder scan:** the only `todo!()`s are intentional red-phase stubs, each replaced within its
  own task. "Port remaining cases from `<file>.test.ts`" points at a real, existing artifact.
- **Known divergence (intentional):** columns/spans use Unicode scalar (`char`) indices, not UTF-16
  code units like the TS. Identical for BMP text; differs only for astral characters (emoji), which
  the design lists as out of scope.
