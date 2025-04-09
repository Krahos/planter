use iced::{Alignment, Element, Length, widget::text};

pub fn data_label<'a, Message>(value: &'a str) -> Element<'a, Message> {
    text(value)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
}
