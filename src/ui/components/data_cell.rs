use iced::{
    Alignment,
    widget::{TextInput, text_input},
};

pub fn data_cell<'a, Message>(
    placeholder: &str,
    value: &'a str,
    is_error: bool,
) -> TextInput<'a, Message>
where
    Message: 'a + Clone,
{
    text_input(placeholder, value)
        .align_x(Alignment::Center)
        .style(move |theme: &iced::Theme, status| {
            if value.is_empty() {
                text_input::default(theme, status)
            } else {
                text_input::Style {
                    border: iced::Border {
                        color: if !is_error {
                            theme.extended_palette().success.base.color
                        } else {
                            theme.extended_palette().danger.base.color
                        },
                        radius: 0.0.into(),
                        width: 1.0,
                    },
                    ..text_input::default(theme, status)
                }
            }
        })
}
