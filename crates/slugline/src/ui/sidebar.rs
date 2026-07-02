use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use slugline_core::dates::YearMonth;

use crate::app::Message;
use crate::ui::calendar;

/// The sidebar pane: a collapse header followed by the calendar section.
/// Additional sections (agenda, todos) land in Phase 4, stacked below the
/// calendar in this same column. Port of `web/src/lib/components/Sidebar.svelte`.
pub fn view<'a>(
    calendar_month: YearMonth,
    today: &str,
    active: &str,
    notes_with_files: &[String],
) -> Element<'a, Message> {
    let header = row![
        container(text("Slugline").size(13)).width(Length::Fill),
        button(text("\u{ab}").size(13)) // «
            .on_press(Message::ToggleSidebar)
            .padding([2, 8]),
    ]
    .align_y(Alignment::Center)
    .padding([8, 10]);

    column![
        header,
        calendar::view(calendar_month, today, active, notes_with_files),
    ]
    .width(Length::Fill)
    .into()
}

/// The slim rail shown instead of the sidebar when it is collapsed.
pub fn collapsed_rail<'a>() -> Element<'a, Message> {
    container(
        button(text("\u{bb}").size(13)) // »
            .on_press(Message::ToggleSidebar)
            .padding([6, 8]),
    )
    .padding([8, 4])
    .into()
}
