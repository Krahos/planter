use iced::{
    Element, Task,
    widget::{Column, Row, button},
};
use iced_aw::ContextMenu;
use planter_core::{
    person::{EmailAddress, Person, PhoneNumber},
    project::Project,
    resources::Resource,
};
use std::str::FromStr;

use crate::{AppMessage, ui::constants};

use super::components::{data_cell::data_cell, data_label::data_label};

#[derive(Debug, Default)]
pub struct PersonnelState {
    repr: Vec<Repr>,
    new_person_name: String,
    new_person_surname: String,
    is_new_name_err: bool,
}

#[derive(Debug, Default)]
struct Repr {
    res_id: usize,
    first_name: String,
    is_first_name_err: bool,
    last_name: String,
    is_last_name_err: bool,
    email: String,
    is_email_err: bool,
    phone_number: String,
    is_phone_err: bool,
    hourly_rate: String,
    is_rate_err: bool,
}

#[derive(Debug, Clone)]
pub enum PersonnelMessage {
    UpdateName(usize, usize, String),
    UpdateSurname(usize, usize, String),
    UpdateEmail(usize, usize, String),
    UpdatePhoneNumber(usize, usize, String),
    UpdateHourlyRate(usize, usize, String),
    UpdateNewName(String),
    UpdateNewSurname(String),
    CreateNewPersonnel,
    DeletePersonnel(usize, usize),
    ResourceDeleted(usize),
}

pub fn reset_project(state: &mut PersonnelState, project: &Project) {
    state.repr.clear();
    for (res_id, resource) in project.resources().iter().enumerate() {
        if let Resource::Personnel {
            person,
            hourly_rate,
        } = resource
        {
            state.repr.push(Repr {
                res_id,
                first_name: person.first_name().to_owned(),
                last_name: person.last_name().to_owned(),
                email: person
                    .email()
                    .as_ref()
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default(),
                phone_number: person
                    .phone()
                    .as_ref()
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default(),
                hourly_rate: hourly_rate
                    .map(|r| (r as f32 / 100.0).to_string())
                    .unwrap_or_default(),
                ..Default::default()
            });
        }
    }
    state.new_person_name = String::new();
    state.new_person_surname = String::new();
    state.is_new_name_err = false;
}

pub fn update(
    state: &mut PersonnelState,
    project: &mut Project,
    message: PersonnelMessage,
) -> Task<AppMessage> {
    match message {
        PersonnelMessage::UpdateName(i, res_id, n) => {
            if let Resource::Personnel { person, .. } = project.resource_mut(res_id).unwrap() {
                state.repr[i].is_first_name_err = person.update_first_name(&n).is_err();
            } else {
                eprintln!("Expected Personnel resource at index {res_id}");
            }
            state.repr[i].first_name = n;
            Task::none()
        }
        PersonnelMessage::UpdateSurname(i, res_id, s) => {
            if let Resource::Personnel { person, .. } = project.resource_mut(res_id).unwrap() {
                state.repr[i].is_last_name_err = person.update_last_name(&s).is_err();
            } else {
                eprintln!("Expected Personnel resource at index {res_id}");
            }

            state.repr[i].last_name = s;
            Task::none()
        }
        PersonnelMessage::UpdateEmail(i, res_id, e) => {
            if let Resource::Personnel { person, .. } = project.resource_mut(res_id).unwrap() {
                if e.is_empty() {
                    person.rm_email();
                    state.repr[i].is_email_err = false;
                } else if let Ok(email) = EmailAddress::from_str(&e) {
                    person.update_email(email);
                    state.repr[i].is_email_err = false;
                } else {
                    state.repr[i].is_email_err = true;
                }
            } else {
                eprintln!("Expected Personnel resource at index {res_id}");
            }
            state.repr[i].email = e;
            Task::none()
        }
        PersonnelMessage::UpdatePhoneNumber(i, res_id, p) => {
            if let Resource::Personnel { person, .. } = project.resource_mut(res_id).unwrap() {
                if p.is_empty() {
                    person.rm_phone();
                    state.repr[i].is_phone_err = false;
                } else if let Ok(phone) = PhoneNumber::from_str(&p) {
                    person.update_phone(phone);
                    state.repr[i].is_phone_err = false;
                } else {
                    state.repr[i].is_phone_err = true;
                }
            } else {
                eprintln!("Expected Personnel resource at index {res_id}");
            }
            state.repr[i].phone_number = p;
            Task::none()
        }
        PersonnelMessage::UpdateNewName(n) => {
            state.is_new_name_err = false;
            state.new_person_name = n;
            Task::none()
        }
        PersonnelMessage::UpdateNewSurname(n) => {
            state.is_new_name_err = false;
            state.new_person_surname = n;
            Task::none()
        }
        PersonnelMessage::CreateNewPersonnel => {
            if state.new_person_name.is_empty() || state.new_person_surname.is_empty() {
                state.is_new_name_err = true;
                return Task::none();
            }
            if let Ok(person) = Person::new(&state.new_person_name, &state.new_person_surname) {
                state.is_new_name_err = false;
                let personnel = Resource::Personnel {
                    person,
                    hourly_rate: None,
                };
                project.add_resource(personnel);
                let res_id = project.resources().len() - 1;
                state.repr.push(Repr {
                    res_id,
                    first_name: state.new_person_name.clone(),
                    last_name: state.new_person_surname.clone(),
                    ..Default::default()
                });
                state.new_person_name = "".to_owned();
                state.new_person_surname = "".to_owned();
            } else {
                state.is_new_name_err = true;
            }
            Task::none()
        }
        PersonnelMessage::DeletePersonnel(i, res_id) => {
            let _ = project.rm_resource(res_id);
            state.repr.remove(i);
            Task::perform(async move { res_id }, AppMessage::ResourceDeleted)
        }
        PersonnelMessage::UpdateHourlyRate(i, res_id, r) => {
            if let Ok(amount) = r.parse::<f32>() {
                if let Resource::Personnel { hourly_rate, .. } =
                    project.resource_mut(res_id).unwrap()
                {
                    let cents = (amount * 100.).round() as u32;
                    *hourly_rate = Some(u16::try_from(cents).unwrap_or(u16::MAX));
                } else {
                    eprintln!("Expected Personnel resource at index {res_id}");
                }
                state.repr[i].hourly_rate = r;
                state.repr[i].is_rate_err = false;
            } else if r.is_empty() {
                if let Resource::Personnel { hourly_rate, .. } =
                    project.resource_mut(res_id).unwrap()
                {
                    *hourly_rate = None;
                } else {
                    eprintln!("Expected Personnel resource at index {res_id}");
                }
                state.repr[i].hourly_rate = r;
                state.repr[i].is_rate_err = false;
            }
            Task::none()
        }
        PersonnelMessage::ResourceDeleted(res_id) => {
            state.repr.iter_mut().for_each(|r| {
                if r.res_id > res_id {
                    r.res_id -= 1;
                }
            });
            Task::none()
        }
    }
}

pub fn view(state: &PersonnelState) -> Element<'_, PersonnelMessage> {
    let headers = Row::new()
        .push(data_label("Resource ID"))
        .push(data_label("Name"))
        .push(data_label("Surname"))
        .push(data_label("E-Mail"))
        .push(data_label("Phone"))
        .push(data_label("Hourly Rate"));

    let content_rows: Vec<Element<'_, _>> = state
        .repr
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let res_id = r.res_id;
            let row = Row::new()
                .push(data_label(r.res_id))
                .push(
                    data_cell("Sebastiano", &r.first_name, false)
                        .on_input(move |n| PersonnelMessage::UpdateName(i, r.res_id, n)),
                )
                .push(
                    data_cell("Giordano", &r.last_name, false)
                        .on_input(move |s| PersonnelMessage::UpdateSurname(i, r.res_id, s)),
                )
                .push(
                    data_cell("sebastiano.giordano@planter.com", &r.email, r.is_email_err)
                        .on_input(move |e| PersonnelMessage::UpdateEmail(i, r.res_id, e)),
                )
                .push(
                    data_cell("+39 3284929293", &r.phone_number, r.is_phone_err)
                        .on_input(move |p| PersonnelMessage::UpdatePhoneNumber(i, r.res_id, p)),
                )
                .push(
                    data_cell("50.00", &r.hourly_rate, false)
                        .on_input(move |h| PersonnelMessage::UpdateHourlyRate(i, r.res_id, h)),
                );

            ContextMenu::new(row, move || -> Element<'_, PersonnelMessage> {
                Column::with_children(vec![
                    button("Delete")
                        .on_press(PersonnelMessage::DeletePersonnel(i, res_id))
                        .width(constants::CONTEXT_MENU_WIDTH)
                        .into(),
                ])
                .spacing(constants::CONTEXT_MENU_SPACING)
                .into()
            })
            .into()
        })
        .collect();

    let new_row = Row::new()
        .push(data_label(""))
        .push(
            data_cell("Sebastiano", &state.new_person_name, state.is_new_name_err)
                .on_input(PersonnelMessage::UpdateNewName)
                .on_submit(PersonnelMessage::CreateNewPersonnel),
        )
        .push(
            data_cell("Giordano", &state.new_person_surname, state.is_new_name_err)
                .on_input(PersonnelMessage::UpdateNewSurname)
                .on_submit(PersonnelMessage::CreateNewPersonnel),
        )
        .push(data_cell("", "", false))
        .push(data_cell("", "", false))
        .push(data_cell("", "", false));

    Column::new()
        .push(headers)
        .extend(content_rows)
        .push(new_row)
        .into()
}
