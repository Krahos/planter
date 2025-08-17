use std::fmt::Display;

use iced::{
    Alignment, Length,
    alignment::{Horizontal, Vertical},
    widget::{Container, container, text},
};

pub fn data_label<'a, Message>(
    value: impl Display + iced::advanced::text::IntoFragment<'a>,
) -> Container<'a, Message> {
    container(
        text(value)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Vertical::Center)
            .width(100),
    )
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .style(|theme: &iced::Theme| container::Style {
        border: iced::Border {
            width: 1.0,
            radius: 0.0.into(),
            color: theme.palette().primary,
        },
        ..Default::default()
    })
}
