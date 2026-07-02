use iced::widget::{button, column, container, row, span, text};
use iced::{Element, Length};

use slugline_core::agenda::{AgendaItem, derive_agenda};

use crate::app::Message;
use crate::theme_iced::Palette;

/// The sidebar's Agenda section: scheduled meetings for the currently open note,
/// derived fresh from its lines on every render (no stored state, mirroring the
/// web's `$derived(deriveAgenda(app.editor.lines))`). Port of
/// `web/src/lib/components/Agenda.svelte`.
pub fn view<'a>(lines: &[String], active: &str, palette: &Palette) -> Element<'a, Message> {
    let items = derive_agenda(lines);
    let status_bar = palette.status_bar;

    let header = container(text("Agenda").size(13).color(palette.heading[1]));
    let body: Element<'a, Message> = if items.is_empty() {
        text("No scheduled meetings")
            .size(12)
            .color(palette.muted)
            .into()
    } else {
        let mut list = column![].spacing(2);
        for item in items {
            list = list.push(agenda_row(item, active, palette));
        }
        list.into()
    };

    container(column![header, body].spacing(6).width(Length::Fill))
        .padding([10, 12])
        .style(move |_theme| container::Style {
            border: iced::Border {
                color: status_bar,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn agenda_row<'a>(item: AgendaItem, active: &str, palette: &Palette) -> Element<'a, Message> {
    let done = item.ended.is_some();
    let name_color = if done { palette.todo_done } else { palette.fg };
    let fg = palette.fg;
    let todo_done = palette.todo_done;
    let edit_bar_bg = palette.edit_bar_bg;

    let mut label = row![
        text(item.time).size(12).color(palette.accent),
        iced::widget::rich_text([span(item.name).color(name_color).strikethrough(done)]).size(12),
    ]
    .spacing(6);
    if done {
        label = label.push(text("\u{2713}").size(11).color(todo_done));
    }

    button(label)
        .padding([2, 4])
        .width(Length::Fill)
        .on_press(Message::OpenDateAndLine(
            active.to_string(),
            item.heading_line_index,
        ))
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
