use std::fmt::Display;

use iced::{Alignment, Element, Length, widget::text};

pub fn data_label<'a, Message>(
    value: impl Display + iced::advanced::text::IntoFragment<'a>,
) -> Element<'a, Message> {
    text(value)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
}
