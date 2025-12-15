use iced::{
    Element, Length, Task,
    widget::{
        PaneGrid, button,
        pane_grid::{self, DragEvent},
        row, scrollable, text,
    },
};
use planter_core::project::Project;
use ui::{gantt_page, personnel_page, tasks_page};

use crate::ui::{
    gantt_page::{GanttMessage, GanttState},
    materials_page::{self, MaterialsMessage, MaterialsState},
    personnel_page::{PersonnelMessage, PersonnelState},
    style::{PADDING_MD, SPACING_SM, THEME},
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
    materials_state: MaterialsState,
    gantt_state: GanttState,
    panes: pane_grid::State<Pane>,
    focus: Option<pane_grid::Pane>,
}

#[derive(Default)]
enum PaneType {
    #[default]
    Tasks,
    Personnel,
    Materials,
    Gantt,
}

#[derive(Clone, Debug)]
enum AppMessage {
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    TasksMessage(TasksMessage),
    PersonnelMessage(PersonnelMessage),
    MaterialsMessage(MaterialsMessage),
    GanttMessage(GanttMessage),
    ResourceDeleted(usize),
    // Close(pane_grid::Pane),
    Restore,
    Maximize(pane_grid::Pane),
    TogglePin(pane_grid::Pane),
}

#[derive(Default)]
struct Pane {
    is_pinned: bool,
    pane_type: PaneType,
}

fn update(state: &mut Appstate, message: AppMessage) -> Task<AppMessage> {
    match message {
        AppMessage::PaneDragged(drag_event) => {
            if let DragEvent::Dropped { pane, target } = drag_event {
                state.panes.drop(pane, target);
            }
            Task::none()
        }
        AppMessage::PaneResized(resize_event) => {
            state.panes.resize(resize_event.split, resize_event.ratio);
            Task::none()
        }
        AppMessage::TasksMessage(tasks_message) => {
            tasks_page::update(&mut state.tasks_state, &mut state.project, tasks_message);
            Task::none()
        }
        AppMessage::PersonnelMessage(personnel_message) => personnel_page::update(
            &mut state.personnel_state,
            &mut state.project,
            personnel_message,
        ),
        AppMessage::MaterialsMessage(materials_message) => materials_page::update(
            &mut state.materials_state,
            &mut state.project,
            materials_message,
        ),
        AppMessage::GanttMessage(gantt_message) => {
            gantt_page::update(&mut state.gantt_state, &mut state.project, gantt_message);
            Task::none()
        }
        AppMessage::PaneClicked(pane) => {
            state.focus = Some(pane);
            Task::none()
        }
        AppMessage::Restore => {
            state.panes.restore();
            Task::none()
        }
        AppMessage::Maximize(pane) => {
            state.panes.maximize(pane);
            Task::none()
        }
        AppMessage::TogglePin(pane) => {
            if let Some(Pane { is_pinned, .. }) = state.panes.get_mut(pane) {
                *is_pinned = !*is_pinned;
            }
            Task::none()
        }
        AppMessage::ResourceDeleted(res_id) => {
            let task1 = materials_page::update(
                &mut state.materials_state,
                &mut state.project,
                MaterialsMessage::ResourceDeleted(res_id),
            );
            let task2 = personnel_page::update(
                &mut state.personnel_state,
                &mut state.project,
                PersonnelMessage::ResourceDeleted(res_id),
            );

            Task::batch([task1, task2])
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
            PaneType::Materials => (
                "Materials",
                materials_page::view(&app_state.materials_state).map(AppMessage::from),
            ),
            PaneType::Gantt => (
                "Gantt",
                gantt_page::view(&app_state.project).map(AppMessage::from),
            ),
        };
        let title = row![text(title).color(if is_focused {
            THEME.primary
        } else {
            THEME.text_muted
        }),]
        .spacing(SPACING_SM);

        let title_bar = pane_grid::TitleBar::new(title)
            .controls(pane_grid::Controls::new(view_controls(
                id,
                total_panes,
                pane.is_pinned,
                is_maximized,
            )))
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
        let tasks_pane = pane;
        if let Some((pane, _)) = panes.split(
            pane_grid::Axis::Horizontal,
            tasks_pane,
            Pane {
                is_pinned: false,
                pane_type: PaneType::Gantt,
            },
        ) {
            if let Some((pane, _)) = panes.split(
                pane_grid::Axis::Vertical,
                pane,
                Pane {
                    is_pinned: false,
                    pane_type: PaneType::Personnel,
                },
            ) {
                panes.split(
                    pane_grid::Axis::Horizontal,
                    pane,
                    Pane {
                        is_pinned: false,
                        pane_type: PaneType::Materials,
                    },
                );
            }
        }

        Appstate {
            panes,
            project: Project::new("World conquer"),
            tasks_state: TasksState::default(),
            personnel_state: PersonnelState::default(),
            materials_state: MaterialsState::default(),
            gantt_state: GanttState::default(),
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

impl From<MaterialsMessage> for AppMessage {
    fn from(value: MaterialsMessage) -> Self {
        AppMessage::MaterialsMessage(value)
    }
}

impl From<GanttMessage> for AppMessage {
    fn from(value: GanttMessage) -> Self {
        AppMessage::GanttMessage(value)
    }
}

fn view_controls<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    is_maximized: bool,
) -> Element<'a, AppMessage> {
    let pin = button(text(if is_pinned { "Unpin" } else { "Pin" }).size(14))
        .on_press(AppMessage::TogglePin(pane))
        .padding(PADDING_MD);

    let maximize = if total_panes > 1 {
        let (content, message) = if is_maximized {
            ("Restore", AppMessage::Restore)
        } else {
            ("Maximize", AppMessage::Maximize(pane))
        };

        Some(
            button(text(content).size(14))
                .padding(PADDING_MD)
                .on_press(message),
        )
    } else {
        None
    };

    row![pin, maximize].spacing(SPACING_SM).into()
}

mod style {
    use iced::widget::container;
    use iced::{Border, Theme};

    use crate::ui::style::{BORDER_RADIUS_LG, BORDER_WIDTH_THICK, BORDER_WIDTH_THIN, THEME};

    pub fn title_bar_active(_theme: &Theme) -> container::Style {
        container::Style {
            text_color: Some(THEME.text),
            background: Some(THEME.background_alt.into()),
            ..Default::default()
        }
    }

    pub fn title_bar_focused(_theme: &Theme) -> container::Style {
        container::Style {
            text_color: Some(THEME.background),
            background: Some(THEME.primary.into()),
            ..Default::default()
        }
    }

    pub fn pane_active(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(THEME.background.into()),
            border: Border {
                width: BORDER_WIDTH_THIN,
                color: THEME.text_muted.scale_alpha(0.3),
                radius: BORDER_RADIUS_LG.into(),
            },
            ..Default::default()
        }
    }

    pub fn pane_focused(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(THEME.background.into()),
            border: Border {
                width: BORDER_WIDTH_THICK,
                color: THEME.primary,
                radius: BORDER_RADIUS_LG.into(),
            },
            ..Default::default()
        }
    }
}
