use iced::widget::{button, container, row, text};
use iced::{Element, Length};

use slugline_core::tabs::TabsState;

use crate::app::Message;

/// A simple horizontal strip of tab buttons reflecting the open dates and the active one.
/// Styling is intentionally default (theming/polish lands in Phase 6).
pub fn view(tabs: &TabsState) -> Element<'_, Message> {
    let mut strip = row![].spacing(6).padding([6, 8]).width(Length::Fill);
    for (i, date) in tabs.tabs.iter().enumerate() {
        let marker = if i == tabs.active_index {
            "\u{25b8} "
        } else {
            ""
        };
        let label = button(text(format!("{marker}{date}")).size(13))
            .on_press(Message::SwitchTab(i))
            .padding([4, 10]);
        let close = button(text("\u{00d7}").size(13))
            .on_press(Message::CloseTab(i))
            .padding([4, 8]);
        strip = strip.push(label).push(close);
    }
    container(strip).width(Length::Fill).into()
}
