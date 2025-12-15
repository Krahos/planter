use std::fmt::Display;

use iced::{
    Alignment,
    widget::{TextInput, text_input},
};

use crate::ui::style::{
    BORDER_RADIUS_SM, BORDER_WIDTH_THIN, CELL_WIDTH as WIDTH, PADDING_SM, THEME,
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
        .padding(PADDING_SM)
        .style(move |_theme: &iced::Theme, _status| text_input::Style {
            background: THEME.background_alt.into(),
            border: iced::Border {
                color: if !is_error {
                    THEME.text.scale_alpha(0.3)
                } else {
                    THEME.error
                },
                radius: BORDER_RADIUS_SM.into(),
                width: BORDER_WIDTH_THIN,
            },
            icon: THEME.text,
            placeholder: THEME.text.scale_alpha(0.4),
            value: THEME.text,
            selection: THEME.text.scale_alpha(0.3),
        })
        .width(WIDTH)
}
