use iced::{
    Element, Length,
    widget::{
        PaneGrid, button, container,
        pane_grid::{self, DragEvent},
        responsive, scrollable, text,
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
    panes: pane_grid::State<PaneType>,
    focus: Option<pane_grid::Pane>,
}

enum PaneType {
    Tasks(PaneData),
    Personnel(PaneData),
}

#[derive(Clone, Debug)]
enum AppMessage {
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    TasksMessage(TasksMessage),
    PersonnelMessage(PersonnelMessage),
}

#[derive(Default)]
struct PaneData {
    is_pinned: bool,
}

impl PaneData {
    fn new(t: PaneType) -> Self {
        Self { is_pinned: false }
    }
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
        AppMessage::PaneClicked(pane) => todo!(),
    }
}

fn view(app_state: &Appstate) -> Element<'_, AppMessage> {
    let focus = app_state.focus;
    let total_panes = app_state.panes.len();

    PaneGrid::new(&app_state.panes, |id, pane_type, is_maximized| {
        let is_focused = focus == Some(id);
        let total_panses = app_state.panes.len();

        pane_grid::Content::new(match pane_type {
            PaneType::Tasks(_) => {
                scrollable(tasks_page::view(&app_state.tasks_state).map(AppMessage::from))
                    .width(800)
            }
            PaneType::Personnel(_) => scrollable(
                container(personnel_page::view(&app_state.personnel_state).map(AppMessage::from))
                    .width(800),
            ),
        })
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .spacing(5)
    .on_drag(AppMessage::PaneDragged)
    .on_resize(10, AppMessage::PaneResized)
    .into()
}

impl Appstate {
    fn new() -> Self {
        let (mut panes, pane) = pane_grid::State::new(PaneType::Tasks(PaneData::default()));
        panes.split(
            pane_grid::Axis::Vertical,
            pane,
            PaneType::Personnel(PaneData::default()),
        );

        Appstate {
            panes: panes,
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
