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
    pub fn plain(text: impl Into<String>) -> Self {
        Span { text: text.into(), ..Default::default() }
    }
}

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

    #[test]
    fn italic_asterisk_and_underscore() {
        assert_eq!(
            render_inline("*it*"),
            vec![Span { text: "it".into(), italic: true, ..Default::default() }]
        );
        assert_eq!(
            render_inline("_it_"),
            vec![Span { text: "it".into(), italic: true, ..Default::default() }]
        );
    }

    #[test]
    fn strikethrough() {
        assert_eq!(
            render_inline("~~deleted~~"),
            vec![Span { text: "deleted".into(), strike: true, ..Default::default() }]
        );
    }

    #[test]
    fn highlight() {
        assert_eq!(
            render_inline("==marked=="),
            vec![Span { text: "marked".into(), highlight: true, ..Default::default() }]
        );
    }

    #[test]
    fn unsafe_link_left_as_literal() {
        assert_eq!(
            render_inline("[x](javascript:alert(1))"),
            vec![Span::plain("[x](javascript:alert(1))")]
        );
    }

    #[test]
    fn code_protects_inner_markup() {
        assert_eq!(
            render_inline("`a*b*c`"),
            vec![Span { text: "a*b*c".into(), code: true, ..Default::default() }]
        );
        assert_eq!(
            render_inline("`~~raw~~`"),
            vec![Span { text: "~~raw~~".into(), code: true, ..Default::default() }]
        );
        assert_eq!(
            render_inline("`==raw==`"),
            vec![Span { text: "==raw==".into(), code: true, ..Default::default() }]
        );
    }

    #[test]
    fn bold_and_strikethrough_together() {
        assert_eq!(
            render_inline("**bold** and ~~strike~~"),
            vec![
                Span { text: "bold".into(), bold: true, ..Default::default() },
                Span::plain(" and "),
                Span { text: "strike".into(), strike: true, ..Default::default() },
            ]
        );
    }
}
