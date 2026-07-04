use chrono::NaiveDateTime;
use iced::alignment::{Horizontal, Vertical};
use iced::font::Family;
use iced::widget::{Column, Row, button, checkbox, container, text};
use iced::{Element, Font, Length};
use iced_aw::ContextMenu;
use once_cell::sync::Lazy;
use planter_core::duration::NonNegativeDuration;
use planter_core::project::{Project, RelDir, TimeRelationship};
use planter_core::task::Task;
use planter_core::Uuid;
use regex::bytes::Regex;
use std::collections::HashMap;

use super::components::data_cell::data_cell;
use super::components::data_label::data_label;

#[derive(Debug)]
pub struct TasksState {
    repr: Vec<Repr>,
    new_task: String,
}

#[derive(Debug, Default)]
struct Repr {
    id: Uuid,
    outline: String,
    prefix: String,
    depth: usize,
    parent_idx: Option<usize>,
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
    IndentTask(usize),
    OutdentTask(usize),
    InsertAbove(usize),
    InsertBelow(usize),
    AddChildTask(usize),
}

impl Default for TasksState {
    fn default() -> Self {
        TasksState {
            repr: Vec::new(),
            new_task: "".to_owned(),
        }
    }
}

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M";

pub fn update(state: &mut TasksState, project: &mut Project, message: TasksMessage) {
    match message {
        TasksMessage::UpdateName(i, n) => {
            state.repr[i].name = n.clone();
            let id = task_id_at(state, i);
            project.task_mut(id).unwrap().edit_name(n);
        }
        TasksMessage::UpdateDescription(i, d) => {
            state.repr[i].description = d.clone();
            let id = task_id_at(state, i);
            project.task_mut(id).unwrap().edit_description(d);
        }
        TasksMessage::ToggleCompleted(i) => {
            state.repr[i].completed = !state.repr[i].completed;
            let id = task_id_at(state, i);
            project.task_mut(id).unwrap().toggle_completed();
        }
        TasksMessage::UpdateStart(i, s) => {
            if let Ok(date) = NaiveDateTime::parse_from_str(&s, DATE_FORMAT) {
                let id = task_id_at(state, i);
                if project.task_mut(id).unwrap().edit_start(date.and_utc()).is_ok() {
                    update_start_finish_duration(state, project, i);
                    sync_dates_upward(state, project, id);
                } else {
                    state.repr[i].is_start_err = true;
                }
            } else {
                state.repr[i].is_start_err = true;
            }
            state.repr[i].start = s;
        }
        TasksMessage::UpdateFinish(i, s) => {
            if let Ok(date) = NaiveDateTime::parse_from_str(&s, DATE_FORMAT) {
                let id = task_id_at(state, i);
                if project.task_mut(id).unwrap().edit_finish(date.and_utc()).is_ok() {
                    update_start_finish_duration(state, project, i);
                    sync_dates_upward(state, project, id);
                } else {
                    state.repr[i].is_finish_err = true;
                }
            } else {
                state.repr[i].is_finish_err = true;
            }
            state.repr[i].finish = s;
        }
        TasksMessage::UpdateDuration(i, d) => {
            if let Ok(duration) = NonNegativeDuration::parse_from_str(&d) {
                let id = task_id_at(state, i);
                project.task_mut(id).unwrap().edit_duration(duration);
                update_start_finish_duration(state, project, i);
                sync_dates_upward(state, project, id);
            } else {
                state.repr[i].is_duration_err = true;
            }
            state.repr[i].duration = d;
        }
        TasksMessage::UpdateResources(_i, _s) => {}
        TasksMessage::UpdatePredecessors(i, p) => {
            let id = task_id_at(state, i);
            let old_pred_ids: Vec<Uuid> = project.predecessors_ids(id).collect();
            let is_failure = if let Some(indices) = parse_indices(&p) {
                if indices.iter().any(|&idx| idx >= state.repr.len()) {
                    true
                } else {
                    let ids: Vec<Uuid> = indices.iter().map(|&idx| task_id_at(state, idx)).collect();
                    project.update_relationships(id, &ids, RelDir::Predecessors, TimeRelationship::FinishToStart).is_err()
                }
            } else {
                true
            };

            if is_failure {
                project
                    .update_relationships(id, &[], RelDir::Predecessors, TimeRelationship::FinishToStart)
                    .expect("It should have been possible to remove predecessors. This is a bug.");
            }

            let new_pred_ids: Vec<Uuid> = project.predecessors_ids(id).collect();
            update_predecessors_repr(state, project, &new_pred_ids);
            update_predecessors_repr(state, project, &old_pred_ids);

            state.repr[i].is_predecessors_err = is_failure;
            state.repr[i].predecessors = p;
        }
        TasksMessage::UpdateSuccessors(i, p) => {
            let id = task_id_at(state, i);
            let old_succ_ids: Vec<Uuid> = project.successors_ids(id).collect();
            let is_failure = if let Some(indices) = parse_indices(&p) {
                if indices.iter().any(|&idx| idx >= state.repr.len()) {
                    true
                } else {
                    let ids: Vec<Uuid> = indices.iter().map(|&idx| task_id_at(state, idx)).collect();
                    project.update_relationships(id, &ids, RelDir::Successors, TimeRelationship::FinishToStart).is_err()
                }
            } else {
                true
            };
            if is_failure {
                project
                    .update_relationships(id, &[], RelDir::Successors, TimeRelationship::FinishToStart)
                    .expect("It should have been possible to remove successors. This is a bug.");
            }

            let new_succ_ids: Vec<Uuid> = project.successors_ids(id).collect();
            update_successors_repr(state, project, &new_succ_ids);
            update_successors_repr(state, project, &old_succ_ids);

            state.repr[i].is_successors_err = is_failure;
            state.repr[i].successors = p;
        }
        TasksMessage::CreateNewTask => {
            let task = Task::new(state.new_task.clone());
            project.add_task(task);
            state.new_task = "".to_owned();
            update_repr(state, project);
        }
        TasksMessage::UpdateNewTask(n) => state.new_task = n,
        TasksMessage::DeleteTask(i) => {
            let id = task_id_at(state, i);
            project
                .rm_task(id)
                .expect("Should have been possible to remove a task. This is a bug.");
            update_repr(state, project);
        }
        TasksMessage::IndentTask(i) => {
            if i == 0 {
                return;
            }
            let current_depth = state.repr[i].depth;
            let parent_idx = (0..i).rev().find(|&j| state.repr[j].depth <= current_depth);
            if let Some(parent_idx) = parent_idx {
                let id = task_id_at(state, i);
                let parent_id = task_id_at(state, parent_idx);
                let _ = project.add_subtask(parent_id, id);
                remove_relationships_with_ancestors(&mut *project, id);
                sync_dates_upward(state, project, id);
                update_repr(state, project);
            }
        }
        TasksMessage::OutdentTask(i) => {
            let id = task_id_at(state, i);
            remove_relationships_with_ancestors(&mut *project, id);
            if let Some(parent_id) = project.parent_of(id) {
                let _ = project.remove_subtask(id);
                // Move the task right after its former parent in the task order
                // so it appears immediately after the parent's subtree.
                project.move_task_after(id, parent_id);
                // Promote one level: if the parent has a parent, become its child instead.
                if let Some(grandparent_id) = project.parent_of(parent_id) {
                    let _ = project.add_subtask(grandparent_id, id);
                }
            }
            update_repr(state, project);
        }
        TasksMessage::InsertAbove(i) => {
            let sibling_id = task_id_at(state, i);
            project.add_sibling_before(Task::new(""), sibling_id);
            update_repr(state, project);
        }
        TasksMessage::InsertBelow(i) => {
            let sibling_id = task_id_at(state, i);
            project.add_sibling_after(Task::new(""), sibling_id);
            update_repr(state, project);
        }
        TasksMessage::AddChildTask(i) => {
            let parent_id = task_id_at(state, i);
            let child_id = project.add_task(Task::new(""));
            // `add_subtask` handles cycle detection — brand-new task can't cycle.
            let _ = project.add_subtask(parent_id, child_id);
            update_repr(state, project);
        }
    }
}

fn update_start_finish_duration(state: &mut TasksState, project: &Project, task_index: usize) {
    let id = task_id_at(state, task_index);
    let task = project.task(id).unwrap();

    if let Some(start) = task.start() {
        state.repr[task_index].start = start.naive_local().format(DATE_FORMAT).to_string();
        state.repr[task_index].is_start_err = false;
    }
    if let Some(finish) = task.finish() {
        state.repr[task_index].finish = finish.naive_local().format(DATE_FORMAT).to_string();
        state.repr[task_index].is_finish_err = false;
    }
    if let Some(duration) = task.duration() {
        state.repr[task_index].duration = format!("{} h", duration.num_hours());
        state.repr[task_index].is_duration_err = false;
    }
}

fn dfs_order(state: &mut TasksState, project: &Project, task_order: &[Uuid]) {
    let roots: Vec<Uuid> = task_order
        .iter()
        .filter(|&&id| project.parent_of(id).is_none())
        .copied()
        .collect();

    let mut display_order = Vec::new();
    for &root in &roots {
        dfs_visit(root, project, task_order, &mut display_order);
    }

    let id_to_idx: HashMap<Uuid, usize> = display_order
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    for (i, &id) in display_order.iter().enumerate() {
        let task = project.task(id).unwrap();

        let mut depth = 0usize;
        let mut ancestor = id;
        while let Some(parent) = project.parent_of(ancestor) {
            depth += 1;
            ancestor = parent;
        }

        let parent_idx = project.parent_of(id).and_then(|pid| id_to_idx.get(&pid).copied());

        let outline = i.to_string();

        let predecessors = project
            .predecessors_ids(id)
            .filter_map(|p_id| id_to_idx.get(&p_id).map(|p| p.to_string()))
            .collect::<Vec<String>>()
            .join(";");
        let successors = project
            .successors_ids(id)
            .filter_map(|s_id| id_to_idx.get(&s_id).map(|s| s.to_string()))
            .collect::<Vec<String>>()
            .join(";");

        state.repr.push(Repr {
            id,
            outline,
            prefix: String::new(),
            depth,
            parent_idx,
            name: task.name().to_owned(),
            description: task.description().map(|d| d.to_owned()).unwrap_or_default(),
            completed: task.completed(),
            start: task.start().map(|s| s.naive_local().format(DATE_FORMAT).to_string()).unwrap_or_default(),
            is_start_err: false,
            finish: task.finish().map(|f| f.naive_local().format(DATE_FORMAT).to_string()).unwrap_or_default(),
            is_finish_err: false,
            duration: task.duration().map(|d| d.to_string()).unwrap_or_default(),
            is_duration_err: false,
            predecessors,
            is_predecessors_err: false,
            successors,
            is_successors_err: false,
            resources: "".to_owned(),
        });
    }
}

fn dfs_visit(id: Uuid, project: &Project, task_order: &[Uuid], order: &mut Vec<Uuid>) {
    order.push(id);
    let mut children: Vec<Uuid> = project.subtasks(id).collect();
    children.sort_by_key(|child| task_order.iter().position(|&t| t == *child).unwrap_or(usize::MAX));
    for child in children {
        dfs_visit(child, project, task_order, order);
    }
}

fn update_repr(state: &mut TasksState, project: &mut Project) {
    state.repr.clear();

    let task_order: Vec<Uuid> = project.tasks().map(|t| t.id()).collect();
    dfs_order(state, project, &task_order);

    // Second pass: compute tree-line prefixes.
    for i in 0..state.repr.len() {
        let depth = state.repr[i].depth;
        if depth == 0 {
            continue;
        }

        let mut prefix = String::new();
        for level in 0..depth - 1 {
            // Check if the ancestor at this level has another child below.
            let next = state.repr[i + 1..]
                .iter()
                .position(|r| r.depth <= level + 1);
            let continues = next.is_some_and(|offset| {
                let nxt = i + 1 + offset;
                state.repr[nxt].depth == level + 1
            });
            prefix.push_str(if continues { "│   " } else { "    " });
        }

        // Check for sibling after this task (same parent).
        let has_sibling = match state.repr[i].parent_idx {
            Some(pid) => state.repr[i + 1..].iter().any(|r| r.parent_idx == Some(pid)),
            None => false,
        };

        prefix.push_str(if has_sibling { "├── " } else { "└── " });
        state.repr[i].prefix = prefix;
    }
}

fn update_predecessors_repr(state: &mut TasksState, project: &mut Project, predecessor_ids: &[Uuid]) {
    let id_to_idx: HashMap<Uuid, usize> = state.repr.iter().enumerate().map(|(i, r)| (r.id, i)).collect();
    for &pred_id in predecessor_ids {
        if let Some(&idx) = id_to_idx.get(&pred_id) {
            state.repr[idx].successors = project
                .successors_ids(pred_id)
                .filter_map(|s_id| id_to_idx.get(&s_id).map(|s| s.to_string()))
                .collect::<Vec<String>>()
                .join(";");
        }
    }
}

fn update_successors_repr(state: &mut TasksState, project: &mut Project, successor_ids: &[Uuid]) {
    let id_to_idx: HashMap<Uuid, usize> = state.repr.iter().enumerate().map(|(i, r)| (r.id, i)).collect();
    for &succ_id in successor_ids {
        if let Some(&idx) = id_to_idx.get(&succ_id) {
            state.repr[idx].predecessors = project
                .predecessors_ids(succ_id)
                .filter_map(|p_id| id_to_idx.get(&p_id).map(|p| p.to_string()))
                .collect::<Vec<String>>()
                .join(";");
        }
    }
}

fn task_id_at(state: &TasksState, idx: usize) -> Uuid {
    state.repr[idx].id
}

fn remove_relationships_with_ancestors(project: &mut Project, mut id: Uuid) {
    while let Some(ancestor) = project.parent_of(id) {
        let _ = project.rm_time_relationship(ancestor, id);
        let _ = project.rm_time_relationship(id, ancestor);
        id = ancestor;
    }
}

fn sync_dates_upward(state: &mut TasksState, project: &mut Project, child_id: Uuid) {
    let mut current = child_id;
    while let Some(parent_id) = project.parent_of(current) {
        project.sync_parent_dates(parent_id);
        if let Some(idx) = state.repr.iter().position(|r| r.id == parent_id) {
            update_start_finish_duration(state, project, idx);
        }
        current = parent_id;
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
                    index_s.parse::<usize>().unwrap_or_else(|_| {
                        panic!(
                        "It should have been possible to parse {index_s} as usize. This is a bug."
                    )
                    })
                })
                .collect(),
        )
    } else {
        None
    }
}

pub fn view(state: &TasksState) -> Element<'_, TasksMessage> {
    let headers = Row::new()
        .push(data_label("Task ID"))
        .push(data_label("Name"))
        .push(data_label("Description"))
        .push(data_label("Completed"))
        .push(data_label("Start"))
        .push(data_label("Finish"))
        .push(data_label("Duration"))
        .push(data_label("Predecessors"))
        .push(data_label("Successors"))
        .push(data_label("Resources"));

    let content_rows: Vec<Element<'_, _>> = state
        .repr
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let row = Row::new()
                // WBS outline number
                .push(data_label(&r.outline))
                // Name (indented with tree lines)
                .push(
                    Row::new()
                        .push(
                            text(&r.prefix).font(Font {
                                family: Family::Monospace,
                                ..Default::default()
                            }),
                        )
                        .push(
                            data_cell(format!("Task n{i}"), &r.name, false)
                                .on_input(move |n| TasksMessage::UpdateName(i, n)),
                        ),
                )
                // Description
                .push(
                    data_cell("This task...", &r.description, false)
                        .on_input(move |n| TasksMessage::UpdateDescription(i, n)),
                )
                // Completed
                .push(
                    container(
                        checkbox( r.completed)
                            .on_toggle(move |_| TasksMessage::ToggleCompleted(i))
                            .size(16),
                    )
                    .width(100)
                    .height(20)
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
                ;

            let depth = r.depth;
            ContextMenu::new(row, move || -> Element<'_, TasksMessage> {
                let mut items: Vec<Element<'_, TasksMessage>> = vec![
                    button("Insert Above")
                        .on_press(TasksMessage::InsertAbove(i))
                        .width(140)
                        .into(),
                    button("Insert Below")
                        .on_press(TasksMessage::InsertBelow(i))
                        .width(140)
                        .into(),
                    button("Add Child")
                        .on_press(TasksMessage::AddChildTask(i))
                        .width(140)
                        .into(),
                ];
                if i > 0 {
                    items.push(
                        button("→ Indent")
                            .on_press(TasksMessage::IndentTask(i))
                            .width(140)
                            .into(),
                    );
                }
                if depth > 0 {
                    items.push(
                        button("← Outdent")
                            .on_press(TasksMessage::OutdentTask(i))
                            .width(140)
                            .into(),
                    );
                }
                items.push(
                    button("Delete")
                        .on_press(TasksMessage::DeleteTask(i))
                        .width(140)
                        .into(),
                );
                Column::with_children(items).spacing(1).into()
            })
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
        .push(data_cell("", "", false))
        // Completed
        .push(
            container(
                checkbox(false)
                    .size(16),
            )
            .height(20)
            .width(100)
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
