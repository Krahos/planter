use std::fmt::Debug;

use chrono::NaiveDateTime;
use iced::widget::checkbox;
use iced::{Element, Length};
use iced_aw::{Grid, GridRow};
use planter_core::duration::PositiveDuration;
use planter_core::project::Project;
use planter_core::task::Task;

use super::components::data_cell::data_cell;
use super::components::data_label::data_label;

#[derive(Debug)]
pub struct State {
    project: Project,
    repr: Vec<Repr>,
}

#[derive(Debug, Default)]
struct Repr {
    name: String,
    description: String,
    completed: bool,
    start: String,
    is_start_err: bool,
    finish: String,
    is_finish_err: bool,
    duration: String,
    is_duration_err: bool,
    resources: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateName(usize, String),
    UpdateDescription(usize, String),
    ToggleCompleted(usize),
    UpdateStart(usize, String),
    UpdateFinish(usize, String),
    UpdateDuration(usize, String),
    UpdateResources(usize, String),
}

impl Default for State {
    fn default() -> Self {
        State {
            project: Project::new(String::new())
                .with_tasks((0..10).into_iter().map(|_| Task::new(String::new()))),
            repr: (0..10).into_iter().map(|_| Repr::default()).collect(),
        }
    }
}

pub fn update(state: &mut State, message: Message) {
    match message {
        Message::UpdateName(i, n) => {
            state.repr[i].name = n.clone();
            state.project.tasks_mut()[i].edit_name(n);
        }
        Message::UpdateDescription(i, d) => {
            state.repr[i].description = d.clone();
            state.project.tasks_mut()[i].edit_description(d);
        }
        Message::ToggleCompleted(i) => {
            state.repr[i].completed = !state.repr[i].completed;
            state.project.tasks_mut()[i].toggle_completed();
        }
        Message::UpdateStart(i, s) => {
            if let Ok(date) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M") {
                state.project.tasks_mut()[i].edit_start(date.and_utc());
                if let Some(duration) = state.project.tasks()[i].duration() {
                    state.repr[i].duration = duration.to_string();
                    state.repr[i].duration = format!("{} hour(s)", duration.num_hours());
                }
                state.repr[i].is_start_err = false;
            } else {
                state.repr[i].is_start_err = true;
            }
            state.repr[i].start = s;
        }
        Message::UpdateFinish(i, s) => {
            if let Ok(date) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M") {
                state.project.tasks_mut()[i].edit_finish(date.and_utc());
                if let Some(duration) = state.project.tasks()[i].duration() {
                    state.repr[i].duration = format!("{} hour(s)", duration.num_hours());
                }

                state.repr[i].is_finish_err = false;
            } else {
                state.repr[i].is_finish_err = true;
            }
            state.repr[i].finish = s;
        }
        Message::UpdateDuration(i, d) => {
            if let Ok(duration) = PositiveDuration::parse_from_str(&d) {
                state.project.tasks_mut()[i].edit_duration(duration);
                state.repr[i].is_duration_err = false;
            } else {
                state.repr[i].is_duration_err = true;
            }
            state.repr[i].duration = d;
        }
        Message::UpdateResources(_, _) => {}
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let headers = GridRow::new()
        .push(data_label("Name"))
        .push(data_label("Description"))
        .push(data_label("Completed"))
        .push(data_label("Start"))
        .push(data_label("Finish"))
        .push(data_label("Duration"))
        .push(data_label("Resources"));

    let content_rows: Vec<GridRow<'_, _>> = state
        .repr
        .iter()
        .enumerate()
        .map(|(i, r)| {
            GridRow::new()
                // Name
                .push(
                    data_cell("Task n1", &r.name, false)
                        .on_input(move |n| Message::UpdateName(i, n)),
                )
                // Description
                .push(
                    data_cell("This task...", &r.description, false)
                        .on_input(move |n| Message::UpdateDescription(i, n)),
                )
                // Completed
                .push(checkbox("", r.completed).on_toggle(move |_| Message::ToggleCompleted(i)))
                // Start
                .push(
                    data_cell("1992-04-01 09:15", &r.start, r.is_start_err)
                        .on_input(move |s| Message::UpdateStart(i, s)),
                )
                // Finish
                .push(
                    data_cell("1993-27-05 10:20", &r.finish, r.is_finish_err)
                        .on_input(move |s| Message::UpdateFinish(i, s)),
                )
                // Duration
                .push(
                    data_cell("48 h", &r.duration, r.is_duration_err)
                        .on_input(move |d| Message::UpdateDuration(i, d)),
                )
                // Resources
                .push(
                    data_cell("", &r.resources, false)
                        .on_input(move |res| Message::UpdateResources(i, res)),
                )
        })
        .collect();

    Grid::new()
        .push(headers)
        .extend(content_rows)
        .width(Length::Fill)
        .height(Length::Shrink)
        .into()
}
