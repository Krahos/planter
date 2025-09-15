use core::panic;
use iced::{
    Element, Task,
    widget::{Column, Row, button},
};
use planter_core::{
    person::{EmailAddress, Person, PhoneNumber},
    project::Project,
    resources::Resource,
};
use std::str::FromStr;

use crate::AppMessage;

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

pub fn update(
    state: &mut PersonnelState,
    project: &mut Project,
    message: PersonnelMessage,
) -> Task<AppMessage> {
    match message {
        PersonnelMessage::UpdateName(i, res_id, n) => {
            match project.resource_mut(res_id).unwrap() {
                Resource::Personnel { person, .. } => {
                    state.repr[i].is_first_name_err = person.update_first_name(&n).is_err();
                }
                _ => panic!(),
            }
            state.repr[i].first_name = n;
            Task::none()
        }
        PersonnelMessage::UpdateSurname(i, res_id, s) => {
            match project.resource_mut(res_id).unwrap() {
                Resource::Personnel { person, .. } => {
                    state.repr[i].is_last_name_err = person.update_last_name(&s).is_err();
                }
                _ => panic!(),
            }

            state.repr[i].last_name = s;
            Task::none()
        }
        PersonnelMessage::UpdateEmail(i, res_id, e) => {
            match project.resource_mut(res_id).unwrap() {
                Resource::Personnel { person, .. } => {
                    if e.is_empty() {
                        person.rm_email();
                        state.repr[i].is_email_err = false;
                    } else if let Ok(email) = EmailAddress::from_str(&e) {
                        person.update_email(email);
                        state.repr[i].is_email_err = false;
                    } else {
                        state.repr[i].is_email_err = true;
                    }
                }
                _ => panic!(),
            }
            state.repr[i].email = e;
            Task::none()
        }
        PersonnelMessage::UpdatePhoneNumber(i, res_id, p) => {
            match project.resource_mut(res_id).unwrap() {
                Resource::Personnel { person, .. } => {
                    if p.is_empty() {
                        person.rm_phone();
                        state.repr[i].is_phone_err = false;
                    } else if let Ok(phone) = PhoneNumber::from_str(&p) {
                        person.update_phone(phone);
                        state.repr[i].is_phone_err = false;
                    } else {
                        state.repr[i].is_phone_err = true;
                    }
                }
                _ => panic!(),
            }
            state.repr[i].phone_number = p;
            Task::none()
        }
        PersonnelMessage::UpdateNewName(n) => {
            state.new_person_name = n;
            Task::none()
        }
        PersonnelMessage::UpdateNewSurname(n) => {
            state.new_person_surname = n;
            Task::none()
        }
        PersonnelMessage::CreateNewPersonnel => {
            if state.new_person_name.is_empty() || state.new_person_surname.is_empty() {
                return Task::none();
            }
            if let Some(person) = Person::new(&state.new_person_name, &state.new_person_surname) {
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
            project.rm_resource(res_id);
            state.repr.remove(i);
            Task::perform(async move { res_id }, AppMessage::ResourceDeleted)
        }
        PersonnelMessage::UpdateHourlyRate(i, res_id, r) => {
            if let Ok(amount) = r.parse::<f32>() {
                match project.resource_mut(res_id).unwrap() {
                    Resource::Personnel { hourly_rate, .. } => {
                        *hourly_rate = Some((amount * 100.) as u16);
                    }
                    _ => panic!(),
                }
                state.repr[i].hourly_rate = r;
                state.repr[i].is_rate_err = false;
            } else if r.is_empty() {
                // TODO: Remove rate from project
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
            Row::new()
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
                )
                .push(
                    button("Del")
                        .on_press(PersonnelMessage::DeletePersonnel(i, r.res_id))
                        .width(100)
                        .height(50),
                )
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
