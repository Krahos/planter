use std::path::PathBuf;

use iced::widget::button::{Status, Style};
use iced::{
    Border, Color, Element, Length, Padding, Task, alignment,
    widget::scrollable::{Direction, Scrollbar},
    widget::{
        Column, PaneGrid, button, checkbox, container,
        pane_grid::{self, Axis::Vertical, DragEvent},
        row, scrollable, text,
    },
};
use iced_aw::{
    menu::{Menu, MenuBar},
    menu_items,
};
use planter_core::project::Project;
use ui::{personnel_page, tasks_page};

use crate::ui::{
    materials_page::{self, MaterialsMessage, MaterialsState},
    personnel_page::{PersonnelMessage, PersonnelState},
    tasks_page::{TasksMessage, TasksState},
};

mod ui;

fn main() -> iced::Result {
    iced::application(Appstate::default, update, view)
        .title(|state: &Appstate| {
            let file_label = state
                .file_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string());
            let dirty_mark = if state.dirty { " *" } else { "" };
            format!("{file_label}{dirty_mark}")
        })
        .run()
}

struct Appstate {
    project: Project,
    file_path: Option<PathBuf>,
    dirty: bool,
    error: Option<String>,
    tasks_state: TasksState,
    personnel_state: PersonnelState,
    materials_state: MaterialsState,
    panes: pane_grid::State<Pane>,
    focus: Option<pane_grid::Pane>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum PaneType {
    #[default]
    Tasks,
    Personnel,
    Materials,
}

#[derive(Clone, Debug)]
enum AppMessage {
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    TasksMessage(TasksMessage),
    PersonnelMessage(PersonnelMessage),
    MaterialsMessage(MaterialsMessage),
    ResourceDeleted(usize),
    TogglePaneVisibility(PaneType),
    Restore,
    Maximize(pane_grid::Pane),
    NewProject,
    OpenFile,
    FileOpened(Option<(PathBuf, String)>),
    SaveFile,
    SaveAsFile,
    FileSaved(Option<PathBuf>),
    FileWriteResult(Result<PathBuf, String>),
    DismissError,
}

#[derive(Default)]
struct Pane {
    pane_type: PaneType,
}

fn ensure_planter_ext(path: PathBuf) -> PathBuf {
    if path.extension().is_some_and(|e| e == "planter") {
        path
    } else {
        path.with_extension("planter")
    }
}

fn update(state: &mut Appstate, message: AppMessage) -> Task<AppMessage> {
    match message {
        AppMessage::NewProject => {
            state.project = Project::new("Untitled");
            state.file_path = None;
            state.dirty = false;
            state.error = None;
            state.tasks_state = TasksState::default();
            state.personnel_state = PersonnelState::default();
            state.materials_state = MaterialsState::default();
            Task::none()
        }
        AppMessage::OpenFile => Task::perform(
            async {
                let handle = rfd::AsyncFileDialog::new()
                    .set_title("Open Project")
                    .add_filter("PlanTer Project", &["planter"])
                    .pick_file()
                    .await;
                match handle {
                    Some(h) => {
                        let path = h.path().to_path_buf();
                        let data = h.read().await;
                        let content = String::from_utf8(data).ok()?;
                        Some((path, content))
                    }
                    None => None,
                }
            },
            AppMessage::FileOpened,
        ),
        AppMessage::FileOpened(Some((path, content))) => {
            match ron::from_str(&content) {
                Ok(project) => {
                    state.project = project;
                    state.file_path = Some(path);
                    state.dirty = false;
                    state.error = None;
                    tasks_page::reset_project(&mut state.tasks_state, &mut state.project);
                    personnel_page::reset_project(&mut state.personnel_state, &state.project);
                    materials_page::reset_project(&mut state.materials_state, &state.project);
                }
                Err(e) => {
                    state.error = Some(format!("Failed to open project: {e}"));
                }
            }
            Task::none()
        }
        AppMessage::FileOpened(None) => Task::none(),
        AppMessage::SaveFile => {
            if let Some(ref path) = state.file_path {
                match ron::ser::to_string_pretty(&state.project, ron::ser::PrettyConfig::new()) {
                    Ok(data) => {
                        let path = path.clone();
                        Task::perform(
                            async move {
                                std::fs::write(&path, &data)
                                    .map(|_| path)
                                    .map_err(|e| format!("Failed to save: {e}"))
                            },
                            AppMessage::FileWriteResult,
                        )
                    }
                    Err(e) => {
                        state.error = Some(format!("Failed to serialize: {e}"));
                        Task::none()
                    }
                }
            } else {
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Save Project")
                            .add_filter("PlanTer Project", &["planter"])
                            .save_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    AppMessage::FileSaved,
                )
            }
        }
        AppMessage::SaveAsFile => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .set_title("Save Project As")
                    .add_filter("PlanTer Project", &["planter"])
                    .save_file()
                    .await
                    .map(|h| h.path().to_path_buf())
            },
            AppMessage::FileSaved,
        ),
        AppMessage::FileSaved(Some(path)) => {
            let path = ensure_planter_ext(path);
            match ron::ser::to_string_pretty(&state.project, ron::ser::PrettyConfig::new()) {
                Ok(data) => Task::perform(
                    async move {
                        std::fs::write(&path, &data)
                            .map(|_| path)
                            .map_err(|e| format!("Failed to save: {e}"))
                    },
                    AppMessage::FileWriteResult,
                ),
                Err(e) => {
                    state.error = Some(format!("Failed to serialize: {e}"));
                    Task::none()
                }
            }
        }
        AppMessage::FileSaved(None) => Task::none(),
        AppMessage::FileWriteResult(Ok(path)) => {
            state.file_path = Some(path);
            state.dirty = false;
            state.error = None;
            Task::none()
        }
        AppMessage::FileWriteResult(Err(e)) => {
            state.error = Some(e);
            Task::none()
        }
        AppMessage::DismissError => {
            state.error = None;
            Task::none()
        }
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
            state.dirty = true;
            tasks_page::update(&mut state.tasks_state, &mut state.project, tasks_message)
        }
        AppMessage::PersonnelMessage(personnel_message) => {
            state.dirty = true;
            personnel_page::update(
                &mut state.personnel_state,
                &mut state.project,
                personnel_message,
            )
        }
        AppMessage::MaterialsMessage(materials_message) => {
            state.dirty = true;
            materials_page::update(
                &mut state.materials_state,
                &mut state.project,
                materials_message,
            )
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
        AppMessage::ResourceDeleted(res_id) => {
            state.dirty = true;
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
        AppMessage::TogglePaneVisibility(pane_type) => {
            let pane = state
                .panes
                .iter()
                .find(|(_, p)| p.pane_type == pane_type)
                .map(|(id, _)| *id);

            if let Some(id) = pane {
                state.panes.close(id);
            } else {
                let split_target = state.panes.iter().next().map(|(id, _)| *id);
                if let Some(id) = split_target {
                    state.panes.split(Vertical, id, Pane { pane_type });
                }
            }
            Task::none()
        }
    }
}

fn view(app_state: &Appstate) -> Element<'_, AppMessage> {
    let focus = app_state.focus;
    let total_panes = app_state.panes.len();

    let checked = |t| app_state.panes.iter().any(|(_, p)| p.pane_type == t);
    let file_menu = Menu::new(menu_items!(
        (menu_item_button("New Project", AppMessage::NewProject)),
        (menu_item_button("Open...", AppMessage::OpenFile)),
        (menu_item_button("Save", AppMessage::SaveFile)),
        (menu_item_button("Save As...", AppMessage::SaveAsFile)),
    ))
    .spacing(5.0)
    .offset(15.0)
    .width(180.0);

    let view_menu = Menu::new(menu_items!(
        (checkbox(checked(PaneType::Tasks))
            .label("Tasks")
            .on_toggle(|_| AppMessage::TogglePaneVisibility(PaneType::Tasks))
            .width(Length::Fill)),
        (checkbox(checked(PaneType::Personnel))
            .label("Personnel")
            .on_toggle(|_| AppMessage::TogglePaneVisibility(PaneType::Personnel))
            .width(Length::Fill)),
        (checkbox(checked(PaneType::Materials))
            .label("Materials")
            .on_toggle(|_| AppMessage::TogglePaneVisibility(PaneType::Materials))
            .width(Length::Fill)),
    ))
    .spacing(5.0)
    .offset(15.0)
    .width(180.0);

    let menu = MenuBar::new(menu_items!(
        (container(text("File")).padding([4, 12]), file_menu),
        (container(text("View")).padding([4, 12]), view_menu),
    ))
    .padding(Padding::new(5.0));

    let pane_grid = PaneGrid::new(&app_state.panes, |id, pane, is_maximized| {
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
        };
        let title = row![text(title).color(if is_focused {
            PANE_ID_COLOR_FOCUSED
        } else {
            PANE_ID_COLOR_UNFOCUSED
        }),]
        .spacing(5);

        let title_bar = pane_grid::TitleBar::new(title)
            .controls(pane_grid::Controls::new(view_controls(
                id,
                pane.pane_type,
                total_panes,
                is_maximized,
            )))
            .padding(10)
            .style(if is_focused {
                style::title_bar_focused
            } else {
                style::title_bar_active
            });

        pane_grid::Content::new(
            scrollable(container(widget).padding(Padding::default().bottom(10.0)))
                .direction(Direction::Both {
                    vertical: Scrollbar::new(),
                    horizontal: Scrollbar::new(),
                })
                .width(Length::Fill)
                .height(Length::Fill),
        )
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
    .on_resize(10, AppMessage::PaneResized);

    let mut content = Column::new().push(menu).push(pane_grid);

    if let Some(error) = &app_state.error {
        content = content.push(
            row![
                text(error).color(Color::from_rgb(1.0, 0.3, 0.3)),
                button("✕")
                    .style(button::danger)
                    .padding(3)
                    .on_press(AppMessage::DismissError),
            ]
            .spacing(10),
        );
    }

    content.into()
}

impl Appstate {
    fn new() -> Self {
        let (mut panes, pane) = pane_grid::State::new(Pane {
            pane_type: PaneType::Tasks,
        });
        if let Some((pane, _)) = panes.split(
            pane_grid::Axis::Vertical,
            pane,
            Pane {
                pane_type: PaneType::Personnel,
            },
        ) {
            panes.split(
                pane_grid::Axis::Horizontal,
                pane,
                Pane {
                    pane_type: PaneType::Materials,
                },
            );
        }

        Appstate {
            panes,
            project: Project::new("Untitled"),
            file_path: None,
            dirty: false,
            error: None,
            tasks_state: TasksState::default(),
            personnel_state: PersonnelState::default(),
            materials_state: MaterialsState::default(),
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

fn menu_item_button<'a>(label: &'a str, msg: AppMessage) -> button::Button<'a, AppMessage> {
    button(
        text(label)
            .width(Length::Fill)
            .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .padding([4, 8])
    .style(menu_button_style)
    .on_press(msg)
}

fn menu_button_style(theme: &iced::Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = Style {
        text_color: palette.background.base.text,
        border: Border::default().rounded(6.0),
        ..Style::default()
    };
    match status {
        Status::Active => Style {
            background: None,
            ..base
        },
        Status::Hovered => Style {
            background: Some(
                Color::from_rgb(
                    (palette.primary.weak.color.r * 1.2).min(1.0),
                    (palette.primary.weak.color.g * 1.2).min(1.0),
                    (palette.primary.weak.color.b * 1.2).min(1.0),
                )
                .into(),
            ),
            ..base
        },
        Status::Pressed => Style {
            background: Some(palette.primary.weak.color.into()),
            ..base
        },
        _ => base,
    }
}

fn view_controls<'a>(
    pane: pane_grid::Pane,
    pane_type: PaneType,
    total_panes: usize,
    is_maximized: bool,
) -> Element<'a, AppMessage> {
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

    let close = Some(
        button(text("✕").size(14))
            .style(button::danger)
            .padding(3)
            .on_press(AppMessage::TogglePaneVisibility(pane_type)),
    );

    row![maximize, close].spacing(5).into()
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
