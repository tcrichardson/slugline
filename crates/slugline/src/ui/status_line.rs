use iced::widget::{container, row, text};
use iced::{Element, Length};

use slugline_core::editor::EditorState;

use crate::app::Message;
use crate::theme_iced::Palette;

pub fn view<'a>(_editor: &EditorState, _palette: &Palette) -> Element<'a, Message> {
    container(text("")).width(Length::Fill).into()
}
