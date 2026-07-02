use iced::widget::{button, container, row, text};
use iced::{Element, Length};

use slugline_core::tabs::TabsState;

use crate::app::Message;
use crate::theme_iced::Palette;

/// A simple horizontal strip of tab buttons reflecting the open dates and the active
/// one. Port of `web/src/lib/components/Tabs.svelte`'s styling: the active tab gets an
/// `--edit-bar-bg` background + `--fg` text; inactive tabs are transparent + `--muted`.
pub fn view<'a>(tabs: &TabsState, palette: &Palette) -> Element<'a, Message> {
    let mut strip = row![].spacing(6).padding([6, 8]).width(Length::Fill);
    for (i, date) in tabs.tabs.iter().enumerate() {
        let active = i == tabs.active_index;
        let marker = if active { "\u{25b8} " } else { "" };
        let fg = palette.fg;
        let muted = palette.muted;
        let edit_bar_bg = palette.edit_bar_bg;
        let label = button(text(format!("{marker}{date}")).size(13))
            .on_press(Message::SwitchTab(i))
            .padding([4, 10])
            .style(move |_theme, _status| button::Style {
                background: if active {
                    Some(edit_bar_bg.into())
                } else {
                    None
                },
                text_color: if active { fg } else { muted },
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            });
        let close = button(text("\u{00d7}").size(13))
            .on_press(Message::CloseTab(i))
            .padding([4, 8])
            .style(move |_theme, _status| button::Style {
                background: None,
                text_color: muted,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            });
        strip = strip.push(label).push(close);
    }
    container(strip).width(Length::Fill).into()
}
