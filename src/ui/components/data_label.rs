use std::fmt::Display;

use iced::{
    Alignment, Length,
    alignment::{Horizontal, Vertical},
    widget::{Container, container, text},
};

use crate::ui::style::{
    BORDER_RADIUS_SM, BORDER_WIDTH_THIN, CELL_WIDTH as WIDTH, PADDING_SM, THEME,
};

pub fn data_label<'a, Message>(
    value: impl Display + iced::advanced::text::IntoFragment<'a>,
) -> Container<'a, Message> {
    container(
        text(value)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Vertical::Center),
    )
    .width(WIDTH)
    .padding(PADDING_SM)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .style(|_theme: &iced::Theme| container::Style {
        background: Some(THEME.background_alt.into()),
        text_color: Some(THEME.text),
        border: iced::Border {
            width: BORDER_WIDTH_THIN,
            radius: BORDER_RADIUS_SM.into(),
            color: THEME.text.scale_alpha(0.15),
        },
        ..Default::default()
    })
}
