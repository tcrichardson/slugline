use iced::font::{Style as FontStyle, Weight};
use iced::widget::{column, container, rich_text, row, scrollable, span, text};
use iced::{Element, Font, Length};

use slugline_core::doc::{Line, Span, classify_line, render_inline};
use slugline_core::editor::{EditorState, Mode};

use super::palette;

const MONO: Font = Font::MONOSPACE;

pub fn view<Message: Clone + 'static>(editor: &EditorState) -> Element<'_, Message> {
    let mut col = column![].padding([16, 24]).spacing(2).width(Length::Fill);
    for (i, line) in editor.lines.iter().enumerate() {
        if i == editor.cursor.line {
            col = col.push(active_line(line, editor.cursor.col, editor.mode));
        } else {
            col = col.push(pretty_line(line));
        }
    }
    scrollable(col).height(Length::Fill).into()
}

fn active_line<'a, Message: Clone + 'static>(
    line: &str,
    col: usize,
    mode: Mode,
) -> Element<'a, Message> {
    let chars: Vec<char> = line.chars().collect();
    let col = col.min(chars.len());
    let before: String = chars[..col].iter().collect();
    let cursor_char: String = chars
        .get(col)
        .map(|c| c.to_string())
        .unwrap_or_else(|| " ".into());
    let after: String = if col < chars.len() {
        chars[col + 1..].iter().collect()
    } else {
        String::new()
    };

    let cursor: Element<'a, Message> = match mode {
        Mode::Normal => container(text(cursor_char).font(MONO).color(palette::BG))
            .style(|_| container::Style {
                background: Some(palette::CURSOR.into()),
                ..container::Style::default()
            })
            .into(),
        Mode::Insert => row![
            container(text(""))
                .width(2)
                .height(Length::Fixed(18.0))
                .style(|_| container::Style {
                    background: Some(palette::CURSOR.into()),
                    ..container::Style::default()
                }),
            text(cursor_char).font(MONO),
        ]
        .into(),
    };

    let line_row = row![text(before).font(MONO), cursor, text(after).font(MONO)];
    container(line_row)
        .width(Length::Fill)
        .padding([2, 0])
        .style(|_| container::Style {
            background: Some(palette::EDIT_BAR_BG.into()),
            border: iced::Border {
                color: palette::RULE,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn pretty_line<'a, Message: Clone + 'static>(line: &str) -> Element<'a, Message> {
    match classify_line(line) {
        Line::Blank => text(" ").into(),
        Line::Heading { level, text: t } => {
            let color = palette::HEADING[(level as usize).clamp(1, 6) - 1];
            let size = 24.0 - (level as f32 - 1.0) * 2.0;
            inline(
                &render_inline(&t),
                Some(color),
                Some(size),
                Weight::Bold,
                false,
            )
        }
        Line::Task { done, text: t } => {
            let box_glyph = if done { "\u{2611}" } else { "\u{2610}" }; // ☑ / ☐
            let content = inline(
                &render_inline(&t),
                if done { Some(palette::TODO_DONE) } else { None },
                None,
                Weight::Normal,
                done, // strikethrough when done
            );
            row![text(box_glyph), text(" "), content].into()
        }
        Line::List {
            ordered,
            number,
            depth,
            text: t,
        } => {
            let prefix = if ordered {
                format!("{}. ", number.unwrap_or(1))
            } else {
                "\u{2022} ".to_string() // •
            };
            row![
                container(text("")).width(Length::Fixed(depth as f32 * 20.0)),
                text(prefix),
                inline(&render_inline(&t), None, None, Weight::Normal, false),
            ]
            .into()
        }
        Line::Blockquote { text: t } => container(inline(
            &render_inline(&t),
            Some(palette::MUTED),
            None,
            Weight::Normal,
            false,
        ))
        .padding([0, 12])
        .style(|_| container::Style {
            border: iced::Border {
                color: palette::BLOCKQUOTE_BORDER,
                width: 3.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into(),
        Line::Meta { key, text: t } => row![
            text(key.to_uppercase()).size(11).color(palette::MUTED),
            text(" "),
            inline(
                &render_inline(&t),
                Some(palette::MUTED),
                Some(12.0),
                Weight::Normal,
                false
            ),
        ]
        .into(),
        Line::Paragraph { text: t } => {
            inline(&render_inline(&t), None, None, Weight::Normal, false)
        }
    }
}

/// Build a `rich_text` from spans. `base_*` apply to every span; per-span flags layer on top.
fn inline<'a, Message: Clone + 'static>(
    spans: &[Span],
    base_color: Option<iced::Color>,
    base_size: Option<f32>,
    base_weight: Weight,
    base_strike: bool,
) -> Element<'a, Message> {
    let built: Vec<_> = spans
        .iter()
        .map(|s| {
            let mut sp = span(s.text.clone());
            // Font: bold/italic/code.
            let mut font = Font {
                weight: base_weight,
                ..Font::DEFAULT
            };
            if s.bold {
                font.weight = Weight::Bold;
            }
            if s.italic {
                font.style = FontStyle::Italic;
            }
            if s.code {
                font = MONO;
            }
            sp = sp.font(font);
            if let Some(c) = base_color {
                sp = sp.color(c);
            }
            if s.code {
                sp = sp.color(palette::MUTED);
            }
            if s.link.is_some() {
                sp = sp.color(palette::LINK).underline(true);
            }
            if let Some(sz) = base_size {
                sp = sp.size(sz);
            }
            if base_strike || s.strike {
                sp = sp.strikethrough(true);
            }
            // Highlight (==text==): span background if supported by the pinned version;
            // otherwise fall back to the highlight color as foreground.
            if s.highlight {
                sp = sp.color(palette::HIGHLIGHT_BG);
            }
            sp
        })
        .collect();
    rich_text(built).into()
}
