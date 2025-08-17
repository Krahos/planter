use iced::{
    Color, Element, Length,
    widget::{
        PaneGrid, button,
        pane_grid::{self, DragEvent},
        row, scrollable, text,
    },
};
use planter_core::project::Project;
use ui::{personnel_page, tasks_page};

use crate::ui::{
    personnel_page::{PersonnelMessage, PersonnelState},
    tasks_page::{TasksMessage, TasksState},
};

mod ui;

fn main() -> iced::Result {
    iced::application(Appstate::default, update, view).run()
}

struct Appstate {
    project: Project,
    tasks_state: TasksState,
    personnel_state: PersonnelState,
    panes: pane_grid::State<Pane>,
    focus: Option<pane_grid::Pane>,
}

#[derive(Default)]
enum PaneType {
    #[default]
    Tasks,
    Personnel,
}

#[derive(Clone, Debug)]
enum AppMessage {
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    TasksMessage(TasksMessage),
    PersonnelMessage(PersonnelMessage),
    Close(pane_grid::Pane),
    Restore,
    Maximize(pane_grid::Pane),
    TogglePin(pane_grid::Pane),
}

#[derive(Default)]
struct Pane {
    is_pinned: bool,
    pane_type: PaneType,
}

fn update(state: &mut Appstate, message: AppMessage) {
    match message {
        AppMessage::PaneDragged(drag_event) => {
            if let DragEvent::Dropped { pane, target } = drag_event {
                state.panes.drop(pane, target);
            }
        }
        AppMessage::PaneResized(resize_event) => {
            state.panes.resize(resize_event.split, resize_event.ratio);
        }
        AppMessage::TasksMessage(tasks_message) => {
            tasks_page::update(&mut state.tasks_state, &mut state.project, tasks_message)
        }
        AppMessage::PersonnelMessage(personnel_message) => personnel_page::update(
            &mut state.personnel_state,
            &mut state.project,
            personnel_message,
        ),
        AppMessage::PaneClicked(pane) => state.focus = Some(pane),
        AppMessage::Close(pane) => {
            if let Some((_, sibling)) = state.panes.close(pane) {
                state.focus = Some(sibling);
            }
        }
        AppMessage::Restore => state.panes.restore(),
        AppMessage::Maximize(pane) => state.panes.maximize(pane),
        AppMessage::TogglePin(pane) => {
            if let Some(Pane { is_pinned, .. }) = state.panes.get_mut(pane) {
                *is_pinned = !*is_pinned;
            }
        }
    }
}

fn view(app_state: &Appstate) -> Element<'_, AppMessage> {
    let focus = app_state.focus;
    let total_panes = app_state.panes.len();

    PaneGrid::new(&app_state.panes, |id, pane, is_maximized| {
        let is_focused = focus == Some(id);

        let (title, widget) = match pane.pane_type {
            PaneType::Tasks => (
                "Tasks",
                tasks_page::view(&app_state.tasks_state).map(AppMessage::from),
            ),
            PaneType::Personnel => (
                "Personnel",
                personnel_page::view(&app_state.personnel_state).map(AppMessage::from),
            ),
        };
        let title = row![text(title).color(if is_focused {
            PANE_ID_COLOR_FOCUSED
        } else {
            PANE_ID_COLOR_UNFOCUSED
        }),]
        .spacing(5);

        let title_bar = pane_grid::TitleBar::new(title)
            .controls(pane_grid::Controls::dynamic(
                view_controls(id, total_panes, pane.is_pinned, is_maximized),
                button(text("X").size(14))
                    .style(button::danger)
                    .padding(3)
                    .on_press_maybe(if total_panes > 1 && !pane.is_pinned {
                        Some(AppMessage::Close(id))
                    } else {
                        None
                    }),
            ))
            .padding(10)
            .style(if is_focused {
                style::title_bar_focused
            } else {
                style::title_bar_active
            });

        pane_grid::Content::new(scrollable(widget))
            .title_bar(title_bar)
            .style(if is_focused {
                style::pane_focused
            } else {
                style::pane_active
            })
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .spacing(5)
    .on_click(AppMessage::PaneClicked)
    .on_drag(AppMessage::PaneDragged)
    .on_resize(10, AppMessage::PaneResized)
    .into()
}

impl Appstate {
    fn new() -> Self {
        let (mut panes, pane) = pane_grid::State::new(Pane {
            is_pinned: false,
            pane_type: PaneType::Tasks,
        });
        panes.split(
            pane_grid::Axis::Vertical,
            pane,
            Pane {
                is_pinned: false,
                pane_type: PaneType::Personnel,
            },
        );

        Appstate {
            panes,
            project: Project::new("World conquer"),
            tasks_state: TasksState::default(),
            personnel_state: PersonnelState::default(),
            focus: None,
        }
    }
}

impl Default for Appstate {
    fn default() -> Self {
        Appstate::new()
    }
}

impl From<TasksMessage> for AppMessage {
    fn from(value: TasksMessage) -> Self {
        AppMessage::TasksMessage(value)
    }
}

impl From<PersonnelMessage> for AppMessage {
    fn from(value: PersonnelMessage) -> Self {
        AppMessage::PersonnelMessage(value)
    }
}

const PANE_ID_COLOR_UNFOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xC7 as f32 / 255.0,
);
const PANE_ID_COLOR_FOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0x47 as f32 / 255.0,
    0x47 as f32 / 255.0,
);

fn view_controls<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    is_maximized: bool,
) -> Element<'a, AppMessage> {
    let pin = button(text(if is_pinned { "Unpin" } else { "Pin" }).size(14))
        .on_press(AppMessage::TogglePin(pane))
        .padding(3);

    let maximize = if total_panes > 1 {
        let (content, message) = if is_maximized {
            ("Restore", AppMessage::Restore)
        } else {
            ("Maximize", AppMessage::Maximize(pane))
        };

        Some(
            button(text(content).size(14))
                .style(button::secondary)
                .padding(3)
                .on_press(message),
        )
    } else {
        None
    };

    let close = button(text("Close").size(14))
        .style(button::danger)
        .padding(3)
        .on_press_maybe(if total_panes > 1 && !is_pinned {
            Some(AppMessage::Close(pane))
        } else {
            None
        });

    row![pin, maximize, close].spacing(5).into()
}

mod style {
    use iced::widget::container;
    use iced::{Border, Theme};

    pub fn title_bar_active(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        container::Style {
            text_color: Some(palette.background.strong.text),
            background: Some(palette.background.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn title_bar_focused(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        container::Style {
            text_color: Some(palette.primary.strong.text),
            background: Some(palette.primary.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn pane_active(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                width: 2.0,
                color: palette.background.strong.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }

    pub fn pane_focused(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                width: 2.0,
                color: palette.primary.strong.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }
}
