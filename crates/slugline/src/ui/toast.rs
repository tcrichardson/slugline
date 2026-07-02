use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length};

use crate::app::Message;

/// A fixed-position, bottom-centered error toast with a dismiss button. Colors are
/// hardcoded (not theme tokens) — port of `web/src/lib/components/Toast.svelte`, which
/// also hardcodes its red `#b3261e` background rather than reading a CSS variable.
pub fn view<'a>(message: &str) -> Element<'a, Message> {
    let bar = container(
        row![
            text(message.to_string()).size(13).color(iced::Color::WHITE),
            button(text("\u{d7}").size(15).color(iced::Color::WHITE))
                .on_press(Message::DismissError)
                .padding([0, 6])
                .style(|_theme, _status| button::Style {
                    background: None,
                    text_color: iced::Color::WHITE,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                }),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding([8, 14])
    .style(|_theme| container::Style {
        background: Some(iced::Color::from_rgb8(0xb3, 0x26, 0x1e).into()),
        border: iced::Border {
            radius: 8.0.into(),
            ..iced::Border::default()
        },
        shadow: iced::Shadow {
            color: iced::Color::BLACK,
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..container::Style::default()
    });

    container(bar)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(iced::alignment::Vertical::Bottom)
        .padding(iced::Padding {
            bottom: 40.0,
            ..iced::Padding::ZERO
        })
        .into()
}
