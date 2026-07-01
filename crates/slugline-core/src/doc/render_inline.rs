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

pub fn render_inline(_text: &str) -> Vec<Span> {
    todo!()
}
