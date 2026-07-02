use iced::font::{Style as FontStyle, Weight};
use iced::widget::{column, container, rich_text, row, scrollable, span, text};
use iced::{Element, Font, Length};

use slugline_core::doc::{Line, Span, classify_line, render_inline};
use slugline_core::editor::{EditorState, Mode};

use crate::theme_iced::Palette;

const MONO: Font = Font::MONOSPACE;

pub fn view<'a, Message: Clone + 'static>(
    editor: &'a EditorState,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let mut col = column![].padding([16, 24]).spacing(2).width(Length::Fill);
    for (i, line) in editor.lines.iter().enumerate() {
        if i == editor.cursor.line {
            col = col.push(active_line(line, editor.cursor.col, editor.mode, palette));
        } else {
            col = col.push(pretty_line(line, palette));
        }
    }
    scrollable(col).height(Length::Fill).into()
}

fn active_line<'a, Message: Clone + 'static>(
    line: &str,
    col: usize,
    mode: Mode,
    palette: &Palette,
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

    let cursor_color = palette.cursor;
    let bg_color = palette.bg;
    let cursor: Element<'a, Message> = match mode {
        Mode::Normal => container(text(cursor_char).font(MONO).color(bg_color))
            .style(move |_| container::Style {
                background: Some(cursor_color.into()),
                ..container::Style::default()
            })
            .into(),
        Mode::Insert => row![
            container(text(""))
                .width(2)
                .height(Length::Fixed(18.0))
                .style(move |_| container::Style {
                    background: Some(cursor_color.into()),
                    ..container::Style::default()
                }),
            text(cursor_char).font(MONO),
        ]
        .into(),
    };

    let edit_bar_bg = palette.edit_bar_bg;
    let rule = palette.rule;
    let line_row = row![text(before).font(MONO), cursor, text(after).font(MONO)];
    container(line_row)
        .width(Length::Fill)
        .padding([2, 0])
        .style(move |_| container::Style {
            background: Some(edit_bar_bg.into()),
            border: iced::Border {
                color: rule,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn pretty_line<'a, Message: Clone + 'static>(
    line: &str,
    palette: &Palette,
) -> Element<'a, Message> {
    match classify_line(line) {
        Line::Blank => text(" ").into(),
        Line::Heading { level, text: t } => {
            let color = palette.heading[(level as usize).clamp(1, 6) - 1];
            let size = 24.0 - (level as f32 - 1.0) * 2.0;
            inline(
                &render_inline(&t),
                Some(color),
                Some(size),
                Weight::Bold,
                false,
                palette,
            )
        }
        Line::Task { done, text: t } => {
            let box_glyph = if done { "\u{2611}" } else { "\u{2610}" }; // ☑ / ☐
            let content = inline(
                &render_inline(&t),
                if done { Some(palette.todo_done) } else { None },
                None,
                Weight::Normal,
                done, // strikethrough when done
                palette,
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
                inline(
                    &render_inline(&t),
                    None,
                    None,
                    Weight::Normal,
                    false,
                    palette
                ),
            ]
            .into()
        }
        Line::Blockquote { text: t } => {
            let border_color = palette.blockquote_border;
            container(inline(
                &render_inline(&t),
                Some(palette.muted),
                None,
                Weight::Normal,
                false,
                palette,
            ))
            .padding([0, 12])
            .style(move |_| container::Style {
                border: iced::Border {
                    color: border_color,
                    width: 3.0,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            })
            .into()
        }
        Line::Meta { key, text: t } => row![
            text(key.to_uppercase()).size(11).color(palette.muted),
            text(" "),
            inline(
                &render_inline(&t),
                Some(palette.muted),
                Some(12.0),
                Weight::Normal,
                false,
                palette,
            ),
        ]
        .into(),
        Line::Paragraph { text: t } => inline(
            &render_inline(&t),
            None,
            None,
            Weight::Normal,
            false,
            palette,
        ),
    }
}

/// Build a `rich_text` from spans. `base_*` apply to every span; per-span flags layer on top.
fn inline<'a, Message: Clone + 'static>(
    spans: &[Span],
    base_color: Option<iced::Color>,
    base_size: Option<f32>,
    base_weight: Weight,
    base_strike: bool,
    palette: &Palette,
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
                sp = sp.color(palette.muted);
            }
            if s.link.is_some() {
                // No dedicated `--link` token exists in `web/`'s theme (links had no
                // custom CSS color there); reuse `--accent` rather than add a token
                // with no built-in-palette counterpart to diverge on.
                sp = sp.color(palette.accent).underline(true);
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
                sp = sp.color(palette.highlight_bg);
            }
            sp
        })
        .collect();
    rich_text(built).into()
}
