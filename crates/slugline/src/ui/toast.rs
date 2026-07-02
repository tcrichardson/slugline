use iced::widget::{container, text};
use iced::{Element, Length};

use crate::app::Message;

pub fn view<'a>(_message: &str) -> Element<'a, Message> {
    container(text("")).width(Length::Fill).into()
}
