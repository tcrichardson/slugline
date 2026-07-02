use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use slugline_core::dates::{MonthCell, YearMonth, month_grid};

use crate::app::Message;
use crate::theme_iced::Palette;

const DOW: [&str; 7] = ["S", "M", "T", "W", "T", "F", "S"];
const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const CELL: f32 = 28.0;

/// The calendar section of the sidebar: month header with prev/next, a
/// day-of-week row, and a 6x7 grid of day cells. Days with a note file get a
/// dot; today gets an outline; the active date is filled.
/// Port of `web/src/lib/components/Calendar.svelte`.
pub fn view<'a>(
    calendar: YearMonth,
    today: &str,
    active: &str,
    notes_with_files: &[String],
    palette: &Palette,
) -> Element<'a, Message> {
    let header = row![
        button(text("\u{2039}").size(14))
            .on_press(Message::PrevMonth)
            .padding([2, 8]),
        container(text(month_label(calendar)).size(13)).center_x(Length::Fill),
        button(text("\u{203a}").size(14))
            .on_press(Message::NextMonth)
            .padding([2, 8]),
    ]
    .align_y(Alignment::Center);

    let mut dow_row = row![].spacing(2);
    for d in DOW {
        dow_row = dow_row
            .push(container(text(d).size(11).color(palette.muted)).center_x(Length::Fixed(CELL)));
    }

    let mut grid = column![dow_row].spacing(2);
    for week in month_grid(calendar.year, calendar.month) {
        let mut wk = row![].spacing(2);
        for cell in &week {
            let has_note = notes_with_files.iter().any(|d| d == &cell.date);
            wk = wk.push(day_cell(cell, today, active, has_note, palette));
        }
        grid = grid.push(wk);
    }

    column![header, grid].spacing(8).padding(12).into()
}

fn day_cell<'a>(
    cell: &MonthCell,
    today: &str,
    active: &str,
    has_note: bool,
    palette: &Palette,
) -> Element<'a, Message> {
    let day_num = cell.date[8..10].trim_start_matches('0').to_string();
    let day_num = if day_num.is_empty() {
        "0".to_string()
    } else {
        day_num
    };
    let is_today = cell.date == today;
    let is_selected = cell.date == active;
    let in_month = cell.in_month;
    let palette = *palette;

    let dot = text(if has_note { "\u{2022}" } else { " " }).size(9);
    let label = column![text(day_num).size(12), dot].align_x(Alignment::Center);

    button(label)
        .width(Length::Fixed(CELL))
        .height(Length::Fixed(CELL))
        .padding(0.0)
        .on_press(Message::OpenDate(cell.date.clone()))
        .style(move |_theme, status| {
            let background = if is_selected {
                Some(palette.accent.into())
            } else if status == button::Status::Hovered {
                Some(palette.edit_bar_bg.into())
            } else {
                None
            };
            let text_color = if is_selected {
                palette.bg
            } else if !in_month {
                palette.muted
            } else {
                palette.fg
            };
            button::Style {
                background,
                text_color,
                border: iced::Border {
                    color: if is_today {
                        palette.accent
                    } else {
                        iced::Color::TRANSPARENT
                    },
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: iced::Shadow::default(),
            }
        })
        .into()
}

fn month_label(ym: YearMonth) -> String {
    let name = MONTH_NAMES[(ym.month as usize - 1).min(11)];
    format!("{name} {}", ym.year)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_month_and_year() {
        assert_eq!(
            month_label(YearMonth {
                year: 2026,
                month: 6
            }),
            "June 2026"
        );
        assert_eq!(
            month_label(YearMonth {
                year: 2027,
                month: 1
            }),
            "January 2027"
        );
    }
}
