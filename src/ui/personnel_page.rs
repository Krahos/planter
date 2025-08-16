use core::panic;
use iced::{
    Element,
    widget::{Column, Row, button},
};
use planter_core::{
    person::{EmailAddress, Person, PhoneNumber},
    project::Project,
    resources::Resource,
};
use std::str::FromStr;

use super::components::{data_cell::data_cell, data_label::data_label};

#[derive(Debug, Default)]
pub struct State {
    project: Project,
    repr: Vec<Repr>,
    new_person_name: String,
    new_person_surname: String,
    is_new_name_err: bool,
}

#[derive(Debug, Default)]
struct Repr {
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
pub enum Message {
    UpdateName(usize, String),
    UpdateSurname(usize, String),
    UpdateEmail(usize, String),
    UpdatePhoneNumber(usize, String),
    UpdateHourlyRate(usize, String),
    UpdateNewName(String),
    UpdateNewSurname(String),
    CreateNewPersonnel,
    DeletePersonnel(usize),
}

pub fn update(state: &mut State, message: Message) {
    match message {
        Message::UpdateName(i, n) => {
            match state.project.resource_mut(i).unwrap() {
                Resource::Personnel { person, .. } => {
                    if person.update_first_name(&n).is_err() {
                        state.repr[i].is_first_name_err = true;
                    } else {
                        state.repr[i].is_first_name_err = false;
                    }
                }
                _ => panic!(),
            }
            state.repr[i].first_name = n;
        }
        Message::UpdateSurname(i, s) => {
            match state.project.resource_mut(i).unwrap() {
                Resource::Personnel { person, .. } => {
                    if person.update_last_name(&s).is_err() {
                        state.repr[i].is_last_name_err = true;
                    } else {
                        state.repr[i].is_last_name_err = false;
                    }
                }
                _ => panic!(),
            }

            state.repr[i].last_name = s
        }
        Message::UpdateEmail(i, e) => {
            match state.project.resource_mut(i).unwrap() {
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
        }
        Message::UpdatePhoneNumber(i, p) => {
            match state.project.resource_mut(i).unwrap() {
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
        }
        Message::UpdateNewName(n) => state.new_person_name = n,
        Message::UpdateNewSurname(n) => state.new_person_surname = n,
        Message::CreateNewPersonnel => {
            if state.new_person_name.is_empty() || state.new_person_surname.is_empty() {
                return;
            }
            if let Some(person) = Person::new(&state.new_person_name, &state.new_person_surname) {
                let personnel = Resource::Personnel {
                    person: person,
                    hourly_rate: None,
                };
                state.project.add_resource(personnel);
                state.repr.push(Repr {
                    first_name: state.new_person_name.clone(),
                    last_name: state.new_person_surname.clone(),
                    ..Default::default()
                });
                state.new_person_name = "".to_owned();
                state.new_person_surname = "".to_owned();
            } else {
                state.is_new_name_err = true;
            }
        }
        Message::DeletePersonnel(i) => {
            state.repr.remove(i);
        }
        Message::UpdateHourlyRate(i, r) => {
            if let Ok(amount) = r.parse::<u16>() {
                match state.project.resource_mut(i).unwrap() {
                    Resource::Personnel { hourly_rate, .. } => {
                        *hourly_rate = Some(amount);
                    }
                    _ => panic!(),
                }
                state.repr[i].hourly_rate = r;
                state.repr[i].is_rate_err = false;
            } else if r.is_empty() {
                state.repr[i].hourly_rate = r;
                state.repr[i].is_rate_err = false;
            }
        }
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let headers = Row::new()
        .push(data_label("Index"))
        .push(data_label("Name"))
        .push(data_label("Surname"))
        .push(data_label("E-Mail"))
        .push(data_label("Phone Number"))
        .push(data_label("Hourly Rate"))
        .push(data_label(""));

    let content_rows: Vec<Element<'_, _>> = state
        .repr
        .iter()
        .enumerate()
        .map(|(i, r)| {
            Row::new()
                .push(data_label(i))
                .push(
                    data_cell("Sebastiano", &r.first_name, false)
                        .on_input(move |n| Message::UpdateName(i, n)),
                )
                .push(
                    data_cell("Giordano", &r.last_name, false)
                        .on_input(move |s| Message::UpdateSurname(i, s)),
                )
                .push(
                    data_cell("sebastiano.giordano@planter.com", &r.email, r.is_email_err)
                        .on_input(move |e| Message::UpdateEmail(i, e)),
                )
                .push(
                    data_cell("+39 3284929293", &r.phone_number, r.is_phone_err)
                        .on_input(move |p| Message::UpdatePhoneNumber(i, p)),
                )
                .push(
                    data_cell("50.00", &r.hourly_rate, false)
                        .on_input(move |h| Message::UpdateHourlyRate(i, h)),
                )
                .push(button("Del").on_press(Message::DeletePersonnel(i)))
                .into()
        })
        .collect();

    let new_row = Row::new()
        .push(data_label(""))
        .push(
            data_cell("Sebastiano", &state.new_person_name, state.is_new_name_err)
                .on_input(Message::UpdateNewName)
                .on_submit(Message::CreateNewPersonnel),
        )
        .push(
            data_cell("Giordano", &state.new_person_surname, state.is_new_name_err)
                .on_input(Message::UpdateNewSurname)
                .on_submit(Message::CreateNewPersonnel),
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
