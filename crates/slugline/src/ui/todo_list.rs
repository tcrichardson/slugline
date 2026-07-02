use iced::widget::{button, column, container, row, span, text};
use iced::{Element, Length};

use slugline_core::todos::{TodoGroup, TodoItem};

use crate::app::Message;
use crate::theme_iced::Palette;

/// The sidebar's To Do section: the 7-day aggregation kept fresh in `App::todo_groups`
/// (a `Task`-driven disk read, unlike Agenda's per-render derivation — see design
/// Section 5). Port of `web/src/lib/components/TodoList.svelte`.
pub fn view<'a>(groups: &[TodoGroup], palette: &Palette) -> Element<'a, Message> {
    let status_bar = palette.status_bar;
    let header = container(text("To Do").size(13).color(palette.heading[1]));

    let body: Element<'a, Message> = if groups.is_empty() {
        text("No to dos in the last 7 days")
            .size(12)
            .color(palette.muted)
            .into()
    } else {
        let mut list = column![].spacing(8);
        for group in groups {
            list = list.push(group_view(group, palette));
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

fn group_view<'a>(group: &TodoGroup, palette: &Palette) -> Element<'a, Message> {
    let mut list = column![text(group.date.clone()).size(11).color(palette.muted)].spacing(2);
    for todo in &group.todos {
        list = list.push(todo_row(group.date.clone(), todo, palette));
    }
    list.into()
}

fn todo_row<'a>(date: String, todo: &TodoItem, palette: &Palette) -> Element<'a, Message> {
    let box_glyph = if todo.done { "\u{2611}" } else { "\u{2610}" }; // ☑ / ☐
    let text_color = if todo.done {
        palette.todo_done
    } else {
        palette.fg
    };
    let fg = palette.fg;
    let edit_bar_bg = palette.edit_bar_bg;

    let label = row![
        text(box_glyph).size(12),
        iced::widget::rich_text([span(todo.text.clone())
            .color(text_color)
            .strikethrough(todo.done)])
        .size(12),
    ]
    .spacing(6);

    button(label)
        .padding([2, 4])
        .width(Length::Fill)
        .on_press(Message::OpenDateAndLine(date, todo.line_index))
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
