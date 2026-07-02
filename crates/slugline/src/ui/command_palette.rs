use iced::alignment::{Horizontal, Vertical};
use iced::font::Weight;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Font, Length};

use slugline_core::doc::{ArgKind, COMMANDS, CommandSpec};

use crate::app::Message;
use crate::theme_iced::Palette;

const MONO: Font = Font::MONOSPACE;
const MAX_SUGGESTIONS: usize = 8;

/// A short usage hint shown after each command's name, derived from its `ArgKind`.
fn usage_hint(spec: &CommandSpec) -> &'static str {
    match spec.arg_kind {
        ArgKind::None => "",
        ArgKind::Text => " <text>",
        ArgKind::Time => " <HH:MM>",
        ArgKind::Date => " <YYYY-MM-DD>",
        ArgKind::Theme => " [light|dark]",
    }
}

/// A one-line description shown next to each command in the palette list.
fn description(spec: &CommandSpec) -> &'static str {
    match spec.name.canonical() {
        "meeting" => "Start a new meeting",
        "note" => "Start a new note",
        "section" => "Add a subsection here",
        "todo" => "Add a to-do item",
        "start" => "Record the meeting's start time",
        "end" => "Record the meeting's end time",
        "scheduled" => "Set the meeting's scheduled time",
        "purpose" => "Set the meeting's purpose",
        "topic" => "Set the note's topic",
        "people" => "Add people (or :p)",
        "goto" => "Jump to a date",
        "today" => "Jump to today",
        "tab" => "Open a date in a new tab",
        "close" => "Close the active tab",
        "w" => "Save now",
        "theme" => "Switch theme",
        _ => "",
    }
}

/// A subsequence fuzzy score between a typed `query` and a candidate `target`, both
/// compared case-insensitively. Every character of `query` must appear in `target`, in
/// order (not necessarily contiguous); returns `None` if it doesn't. Higher scores are
/// better: matches earlier in `target` and consecutive runs of matched characters score
/// higher. An empty `query` matches everything with a score of `0` (used to show every
/// command when nothing has been typed yet).
pub fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let q: Vec<char> = query.to_lowercase().chars().collect();
    let t: Vec<char> = target.to_lowercase().chars().collect();

    let mut score = 0i32;
    let mut ti = 0usize;
    let mut run = 0i32;
    for &qc in &q {
        let mut matched = false;
        while ti < t.len() {
            if t[ti] == qc {
                run += 1;
                score += if ti == 0 { 10 } else { 1 } + run;
                ti += 1;
                matched = true;
                break;
            }
            run = 0;
            ti += 1;
        }
        if !matched {
            return None;
        }
    }
    Some(score)
}

/// Every command whose name fuzzy-matches `query`, in the canonical `COMMANDS` order when
/// `query` is empty, or best-match-first otherwise. Capped at `MAX_SUGGESTIONS`.
pub fn filter_commands(query: &str) -> Vec<&'static CommandSpec> {
    let mut scored: Vec<(&'static CommandSpec, i32)> = COMMANDS
        .iter()
        .filter_map(|spec| fuzzy_score(query, spec.name.canonical()).map(|score| (spec, score)))
        .collect();
    if !query.is_empty() {
        scored.sort_by(|a, b| b.1.cmp(&a.1));
    }
    scored
        .into_iter()
        .take(MAX_SUGGESTIONS)
        .map(|(spec, _)| spec)
        .collect()
}

/// The command palette overlay: a top-centered box with the typed `:command` text and a
/// fuzzy-filtered list of known commands below it. Rendered in a `stack!` on top of the
/// base view whenever `editor.command.is_some()` (design Section 4).
pub fn view<'a>(typed: &str, palette: &Palette) -> Element<'a, Message> {
    let suggestions = filter_commands(typed);
    let edit_bar_bg = palette.edit_bar_bg;
    let rule = palette.rule;
    let bg = palette.bg;
    let status_bar = palette.status_bar;

    let input = container(
        text(format!(":{typed}"))
            .font(MONO)
            .size(15)
            .color(palette.fg),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(edit_bar_bg.into()),
        border: iced::Border {
            color: rule,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..container::Style::default()
    });

    let mut list = column![].spacing(1);
    if suggestions.is_empty() {
        list = list.push(
            container(text("No matching commands").size(12).color(palette.muted)).padding([4, 8]),
        );
    } else {
        for spec in suggestions {
            list = list.push(suggestion_row(spec, palette));
        }
    }

    let box_ = container(column![input, list].spacing(6).width(Length::Fixed(440.0)))
        .padding(12)
        .style(move |_theme| container::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: status_bar,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::BLACK,
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..container::Style::default()
        });

    container(box_)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Top)
        .padding(iced::Padding {
            top: 48.0,
            ..iced::Padding::ZERO
        })
        .into()
}

fn suggestion_row<'a>(spec: &'static CommandSpec, palette: &Palette) -> Element<'a, Message> {
    let name = spec.name.canonical();
    let accent = palette.accent;
    let muted = palette.muted;
    let fg = palette.fg;
    let edit_bar_bg = palette.edit_bar_bg;
    let label = row![
        text(format!(":{name}{}", usage_hint(spec)))
            .font(Font {
                weight: Weight::Bold,
                ..MONO
            })
            .size(12)
            .color(accent),
        text(description(spec)).size(12).color(muted),
    ]
    .spacing(10);

    button(label)
        .padding([3, 8])
        .width(Length::Fill)
        .on_press(Message::PaletteSuggestionClicked(name.to_string()))
        .style(move |_theme, status| {
            let background = if status == button::Status::Hovered {
                Some(edit_bar_bg.into())
            } else {
                None
            };
            button::Style {
                background,
                text_color: fg,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            }
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_everything_with_score_zero() {
        assert_eq!(fuzzy_score("", "todo"), Some(0));
    }

    #[test]
    fn requires_every_query_char_to_appear_in_order() {
        assert!(fuzzy_score("td", "todo").is_some()); // t...d
        assert!(fuzzy_score("dt", "todo").is_none()); // wrong order
        assert!(fuzzy_score("xyz", "todo").is_none()); // not present at all
    }

    #[test]
    fn an_exact_prefix_scores_higher_than_a_scattered_match() {
        let exact = fuzzy_score("to", "today").unwrap();
        let scattered = fuzzy_score("ty", "today").unwrap();
        assert!(exact > scattered);
    }

    #[test]
    fn filter_commands_with_empty_query_returns_the_first_page_in_canonical_order() {
        let names: Vec<&str> = filter_commands("")
            .into_iter()
            .map(|s| s.name.canonical())
            .collect();
        assert_eq!(
            names,
            vec![
                "meeting",
                "note",
                "section",
                "todo",
                "start",
                "end",
                "scheduled",
                "purpose"
            ]
        );
        assert_eq!(names.len(), MAX_SUGGESTIONS);
    }

    #[test]
    fn filter_commands_narrows_to_fuzzy_matches() {
        let names: Vec<&str> = filter_commands("mee")
            .into_iter()
            .map(|s| s.name.canonical())
            .collect();
        assert!(names.contains(&"meeting"));
        assert!(!names.contains(&"close"));
    }

    #[test]
    fn filter_commands_with_no_matches_is_empty() {
        assert!(filter_commands("zzzzz").is_empty());
    }
}
