use chrono::NaiveDateTime;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Column, Row, button, checkbox, container};
use iced::{Element, Length};
use once_cell::sync::Lazy;
use planter_core::duration::PositiveDuration;
use planter_core::project::Project;
use planter_core::task::Task;
use regex::bytes::Regex;

use super::components::data_cell::data_cell;
use super::components::data_label::data_label;

#[derive(Debug)]
pub struct TasksState {
    repr: Vec<Repr>,
    new_task: String,
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
    predecessors: String,
    is_predecessors_err: bool,
    successors: String,
    is_successors_err: bool,
    resources: String,
}

#[derive(Debug, Clone)]
pub enum TasksMessage {
    UpdateName(usize, String),
    UpdateDescription(usize, String),
    ToggleCompleted(usize),
    UpdateStart(usize, String),
    UpdateFinish(usize, String),
    UpdateDuration(usize, String),
    UpdatePredecessors(usize, String),
    UpdateSuccessors(usize, String),
    UpdateResources(usize, String),
    UpdateNewTask(String),
    CreateNewTask,
    DeleteTask(usize),
}

impl Default for TasksState {
    fn default() -> Self {
        TasksState {
            repr: Vec::new(),
            new_task: "".to_owned(),
        }
    }
}

pub fn update(state: &mut TasksState, project: &mut Project, message: TasksMessage) {
    match message {
        TasksMessage::UpdateName(i, n) => {
            state.repr[i].name = n.clone();
            project.task_mut(i).unwrap().edit_name(n);
        }
        TasksMessage::UpdateDescription(i, d) => {
            state.repr[i].description = d.clone();
            project.task_mut(i).unwrap().edit_description(d);
        }
        TasksMessage::ToggleCompleted(i) => {
            state.repr[i].completed = !state.repr[i].completed;
            project.task_mut(i).unwrap().toggle_completed();
        }
        TasksMessage::UpdateStart(i, s) => {
            if let Ok(date) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M") {
                project.task_mut(i).unwrap().edit_start(date.and_utc());
                if let Some(duration) = project.task_mut(i).unwrap().duration() {
                    state.repr[i].duration = duration.to_string();
                    state.repr[i].duration = format!("{} hour(s)", duration.num_hours());
                }
                state.repr[i].is_start_err = false;
            } else {
                state.repr[i].is_start_err = true;
            }
            state.repr[i].start = s;
        }
        TasksMessage::UpdateFinish(i, s) => {
            if let Ok(date) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M") {
                project.task_mut(i).unwrap().edit_finish(date.and_utc());
                if let Some(duration) = project.task_mut(i).unwrap().duration() {
                    state.repr[i].duration = format!("{} hour(s)", duration.num_hours());
                }

                state.repr[i].is_finish_err = false;
            } else {
                state.repr[i].is_finish_err = true;
            }
            state.repr[i].finish = s;
        }
        TasksMessage::UpdateDuration(i, d) => {
            if let Ok(duration) = PositiveDuration::parse_from_str(&d) {
                project.task_mut(i).unwrap().edit_duration(duration);
                state.repr[i].is_duration_err = false;
            } else {
                state.repr[i].is_duration_err = true;
            }
            state.repr[i].duration = d;
        }
        TasksMessage::UpdateResources(_i, _s) => {}
        TasksMessage::UpdatePredecessors(i, p) => {
            let predecessors = project.predecessors_indices(i).collect::<Vec<usize>>();
            let is_failure = if let Some(indices) = parse_indices(&p) {
                project.update_predecessors(i, &indices).is_err()
            } else {
                true
            };

            if is_failure {
                project
                    .update_predecessors(i, &[])
                    .expect("It should have been possible to remove predecessors. This is a bug.");
            }

            update_predecessors_repr(
                state,
                project,
                &project
                    .predecessors_indices(i)
                    .filter(|index| !predecessors.contains(index))
                    .collect::<Vec<usize>>(),
            );
            update_predecessors_repr(state, project, &predecessors);

            state.repr[i].is_predecessors_err = is_failure;
            state.repr[i].predecessors = p;
        }
        TasksMessage::UpdateSuccessors(i, p) => {
            let successors = project.successors_indices(i).collect::<Vec<usize>>();
            let is_failure = if let Some(indices) = parse_indices(&p) {
                project.update_successors(i, &indices).is_err()
            } else {
                true
            };
            if is_failure {
                project
                    .update_successors(i, &[])
                    .expect("It should have been possible to remove predecessors. This is a bug.");
            }

            // Update old and new successors.
            update_successors_repr(
                state,
                project,
                &project
                    .successors_indices(i)
                    .filter(|index| !successors.contains(index))
                    .collect::<Vec<usize>>(),
            );
            update_successors_repr(state, project, &successors);

            state.repr[i].is_successors_err = is_failure;
            state.repr[i].successors = p;
        }
        TasksMessage::CreateNewTask => {
            let task = Task::new(state.new_task.clone());
            project.add_task(task);
            state.repr.push(Repr {
                name: state.new_task.clone(),
                ..Default::default()
            });
            state.new_task = "".to_owned();
        }
        TasksMessage::UpdateNewTask(n) => state.new_task = n,
        TasksMessage::DeleteTask(i) => {
            project
                .rm_task(i)
                .expect("Should have been possible to remove a task. This is a bug.");
            update_repr(state, project);
        }
    }
}

fn update_repr(state: &mut TasksState, project: &mut Project) {
    state.repr.clear();

    for (i, task) in project.tasks().enumerate() {
        state.repr.push(Repr {
            name: task.name().to_owned(),
            description: task.description().to_owned(),
            completed: task.completed(),
            start: if let Some(start) = task.start() {
                start.to_string()
            } else {
                "".to_owned()
            },
            is_start_err: false,
            finish: if let Some(finish) = task.finish() {
                finish.to_string()
            } else {
                "".to_owned()
            },
            is_finish_err: false,
            duration: if let Some(duration) = task.duration() {
                duration.to_string()
            } else {
                "".to_owned()
            },
            is_duration_err: false,
            predecessors: project
                .predecessors_indices(i)
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(";"),
            is_predecessors_err: false,
            successors: project
                .successors_indices(i)
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(";"),
            is_successors_err: false,
            resources: "".to_owned(),
        });
    }
}

fn update_predecessors_repr(state: &mut TasksState, project: &mut Project, predecessors: &[usize]) {
    for &predecessor in predecessors {
        state.repr[predecessor].successors = project
            .successors_indices(predecessor)
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(";");
    }
}

fn update_successors_repr(state: &mut TasksState, project: &mut Project, successors: &[usize]) {
    for &successor in successors {
        state.repr[successor].predecessors = project
            .predecessors_indices(successor)
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(";");
    }
}

fn parse_indices(s: &str) -> Option<Vec<usize>> {
    let bytes = s.as_bytes();
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[0-9]{1,3}(;[0-9]{1,3})*$")
            .expect("It wasn't possible to compile a hardcoded regex. This is a bug.")
    });
    if RE.is_match(bytes) {
        Some(
            s.split(';')
                .map(|index_s| {
                    index_s.parse::<usize>().unwrap_or_else(|_| panic!(
                        "It should have been possible to parse {index_s} as usize. This is a bug."
                    ))
                })
                .collect(),
        )
    } else {
        None
    }
}

pub fn view(state: &TasksState) -> Element<'_, TasksMessage> {
    let headers = Row::new()
        .push(data_label("Index"))
        .push(data_label("Name"))
        .push(data_label("Description"))
        .push(data_label("Completed"))
        .push(data_label("Start"))
        .push(data_label("Finish"))
        .push(data_label("Duration"))
        .push(data_label("Predecessors"))
        .push(data_label("Successors"))
        .push(data_label("Resources"))
        .push(data_label("Delete"));

    let content_rows: Vec<Element<'_, _>> = state
        .repr
        .iter()
        .enumerate()
        .map(|(i, r)| {
            Row::new()
                // Index
                .push(data_label(i))
                // Name
                .push(
                    data_cell(format!("Task n{i}"), &r.name, false)
                        .on_input(move |n| TasksMessage::UpdateName(i, n)),
                )
                // Description
                .push(
                    data_cell("This task...", &r.description, false)
                        .on_input(move |n| TasksMessage::UpdateDescription(i, n)),
                )
                // Completed
                .push(
                    container(
                        checkbox("", r.completed)
                            .on_toggle(move |_| TasksMessage::ToggleCompleted(i)),
                    )
                    .width(100)
                    .height(50)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center),
                )
                // Start
                .push(
                    data_cell("1992-04-01 09:15", &r.start, r.is_start_err)
                        .on_input(move |s| TasksMessage::UpdateStart(i, s)),
                )
                // Finish
                .push(
                    data_cell("1993-27-05 10:20", &r.finish, r.is_finish_err)
                        .on_input(move |s| TasksMessage::UpdateFinish(i, s)),
                )
                // Duration
                .push(
                    data_cell("48 h", &r.duration, r.is_duration_err)
                        .on_input(move |d| TasksMessage::UpdateDuration(i, d)),
                )
                // Predecessors
                .push(
                    data_cell("1;2", &r.predecessors, r.is_predecessors_err)
                        .on_input(move |p| TasksMessage::UpdatePredecessors(i, p)),
                )
                // Successors
                .push(
                    data_cell("1;2", &r.successors, r.is_successors_err)
                        .on_input(move |p| TasksMessage::UpdateSuccessors(i, p)),
                )
                // Resources
                .push(
                    data_cell("", &r.resources, false)
                        .on_input(move |res| TasksMessage::UpdateResources(i, res)),
                )
                // Delete
                .push(button("Del").on_press(TasksMessage::DeleteTask(i)))
                .into()
        })
        .collect();

    let new_row = Row::new()
        // Index
        .push(data_label(""))
        // Name
        .push(
            data_cell("New task name", &state.new_task, false)
                .on_input(TasksMessage::UpdateNewTask)
                .on_submit(TasksMessage::CreateNewTask),
        )
        // Description
        .push(data_cell("This task...", "", false))
        // Completed
        .push(
            container(checkbox("", false))
                .width(100)
                .height(50)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        )
        // Start
        .push(data_cell("", "", false))
        // Finish
        .push(data_cell("", "", false))
        // Duration
        .push(data_cell("", "", false))
        // Predecessors
        .push(data_cell("", "", false))
        // Successors
        .push(data_cell("", "", false))
        // Resources
        .push(data_cell("", "", false));

    Column::new()
        .push(headers)
        .extend(content_rows)
        .push(new_row)
        .height(Length::Shrink)
        .into()
}

#[cfg(test)]
mod tests {
    use proptest::{prelude::Strategy, proptest};

    use crate::ui::tasks_page::parse_indices;

    fn string_array_strategy() -> impl Strategy<Value = String> {
        r"[0-9]{1,3}(;[0-9]{1,3})*".prop_map(|s| s)
    }

    proptest! {
        #[test]
        fn parse_indices_works(s in string_array_strategy()) {
            let arr = parse_indices(&s);

            assert!(arr.is_some());
        }
    }
}
