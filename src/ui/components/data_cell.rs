use std::fmt::Display;

use iced::{
    Alignment,
    widget::{TextInput, text_input},
};

pub fn data_cell<'a, Message>(
    placeholder: impl Display,
    value: impl Display,
    is_error: bool,
) -> TextInput<'a, Message>
where
    Message: 'a + Clone,
{
    text_input(&placeholder.to_string(), &value.to_string())
        .align_x(Alignment::Center)
        .style(move |theme: &iced::Theme, status| text_input::Style {
            border: iced::Border {
                color: if !is_error {
                    theme.palette().primary
                } else {
                    theme.extended_palette().danger.base.color
                },
                radius: 0.0.into(),
                width: 1.0,
            },
            ..text_input::default(theme, status)
        })
        .width(100)
}
