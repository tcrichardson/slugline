use iced::widget::{button, column, container, row, span, text};
use iced::{Element, Length};

use slugline_core::todos::{TodoGroup, TodoItem};

use crate::app::Message;
use crate::ui::palette;

/// The sidebar's To Do section: the 7-day aggregation kept fresh in `App::todo_groups`
/// (a `Task`-driven disk read, unlike Agenda's per-render derivation — see design
/// Section 5). Port of `web/src/lib/components/TodoList.svelte`.
pub fn view<'a>(groups: &[TodoGroup]) -> Element<'a, Message> {
    let header = container(text("To Do").size(13).color(palette::HEADING[1]));

    let body: Element<'a, Message> = if groups.is_empty() {
        text("No to dos in the last 7 days")
            .size(12)
            .color(palette::MUTED)
            .into()
    } else {
        let mut list = column![].spacing(8);
        for group in groups {
            list = list.push(group_view(group));
        }
        list.into()
    };

    container(column![header, body].spacing(6).width(Length::Fill))
        .padding([10, 12])
        .style(|_theme| container::Style {
            border: iced::Border {
                color: palette::STATUS_BAR,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn group_view<'a>(group: &TodoGroup) -> Element<'a, Message> {
    let mut list = column![text(group.date.clone()).size(11).color(palette::MUTED)].spacing(2);
    for todo in &group.todos {
        list = list.push(todo_row(group.date.clone(), todo));
    }
    list.into()
}

fn todo_row<'a>(date: String, todo: &TodoItem) -> Element<'a, Message> {
    let box_glyph = if todo.done { "\u{2611}" } else { "\u{2610}" }; // ☑ / ☐
    let text_color = if todo.done {
        palette::TODO_DONE
    } else {
        palette::FG
    };

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
        .style(|_theme, status| {
            let background = if status == button::Status::Hovered {
                Some(palette::EDIT_BAR_BG.into())
            } else {
                None
            };
            button::Style {
                background,
                text_color: palette::FG,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            }
        })
        .into()
}
