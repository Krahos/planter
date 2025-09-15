use std::fmt::Display;

use iced::{
    Element, Length, Task,
    widget::{Column, Row, Space, button, pick_list},
};
use planter_core::{
    project::Project,
    resources::{Material, NonConsumable, Resource},
};

use crate::{AppMessage, ui::constants};

use super::components::{data_cell::data_cell, data_label::data_label};

#[derive(Debug, Default)]
pub struct MaterialsState {
    repr: Vec<Repr>,
    new_material_name: String,
    is_new_material_err: bool,
}

#[derive(Debug)]
enum Repr {
    Consumable(ConsumableRepr),
    NonConsumable(NonConsumableRepr),
}

impl Default for Repr {
    fn default() -> Self {
        Self::Consumable(ConsumableRepr::default())
    }
}

#[derive(Debug, Default, Clone)]
struct ConsumableRepr {
    res_id: usize,
    name: String,
    quantity: String,
    is_quantity_err: bool,
    cost_per_unit: String,
    is_cost_err: bool,
}

#[derive(Debug, Default, Clone)]
struct NonConsumableRepr {
    res_id: usize,
    name: String,
    quantity: String,
    is_quantity_err: bool,
    hourly_rate: String,
    is_rate_err: bool,
}

impl Into<NonConsumableRepr> for ConsumableRepr {
    fn into(self) -> NonConsumableRepr {
        NonConsumableRepr {
            res_id: self.res_id,
            name: self.name,
            quantity: self.quantity,
            is_quantity_err: self.is_quantity_err,
            hourly_rate: "".to_owned(),
            is_rate_err: false,
        }
    }
}

impl Into<ConsumableRepr> for NonConsumableRepr {
    fn into(self) -> ConsumableRepr {
        ConsumableRepr {
            res_id: self.res_id,
            name: self.name,
            quantity: self.quantity,
            is_quantity_err: self.is_quantity_err,
            cost_per_unit: "".to_owned(),
            is_cost_err: false,
        }
    }
}

impl Repr {
    fn res_id(&self) -> usize {
        match self {
            Repr::Consumable(consumable_repr) => consumable_repr.res_id,
            Repr::NonConsumable(non_consumable_repr) => non_consumable_repr.res_id,
        }
    }

    fn update_res_id(&mut self, res_id: usize) {
        match self {
            Repr::Consumable(consumable_repr) => consumable_repr.res_id = res_id,
            Repr::NonConsumable(non_consumable_repr) => non_consumable_repr.res_id = res_id,
        }
    }

    fn update_name(&mut self, new_name: impl ToString) {
        match self {
            Repr::Consumable(consumable_repr) => consumable_repr.name = new_name.to_string(),
            Repr::NonConsumable(non_consumable_repr) => {
                non_consumable_repr.name = new_name.to_string()
            }
        }
    }

    fn update_quantity(&mut self, quantity: impl ToString) {
        match self {
            Repr::Consumable(consumable_repr) => consumable_repr.quantity = quantity.to_string(),
            Repr::NonConsumable(non_consumable_repr) => {
                non_consumable_repr.quantity = quantity.to_string()
            }
        }
    }

    fn update_is_quantity_err(&mut self, is_err: bool) {
        match self {
            Repr::Consumable(consumable_repr) => consumable_repr.is_quantity_err = is_err,
            Repr::NonConsumable(non_consumable_repr) => {
                non_consumable_repr.is_quantity_err = is_err
            }
        }
    }

    fn update_cost(&mut self, cost: impl ToString) {
        match self {
            Repr::Consumable(consumable_repr) => consumable_repr.cost_per_unit = cost.to_string(),
            Repr::NonConsumable(non_consumable_repr) => {
                non_consumable_repr.hourly_rate = cost.to_string()
            }
        }
    }

    fn update_is_cost_err(&mut self, is_err: bool) {
        match self {
            Repr::Consumable(consumable_repr) => consumable_repr.is_cost_err = is_err,
            Repr::NonConsumable(non_consumable_repr) => non_consumable_repr.is_rate_err = is_err,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Consumable,
    NonConsumable,
}

impl Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selection::Consumable => f.write_str("Consumable"),
            Selection::NonConsumable => f.write_str("Non consumable"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MaterialsMessage {
    UpdateName(usize, usize, String),
    UpdateQuantity(usize, usize, String),
    UpdateCost(usize, usize, String),
    Typeselected(usize, usize, Selection),
    UpdateNewName(String),
    CreateNewMaterial,
    DeleteMaterial(usize, usize),
    ResourceDeleted(usize),
}

pub fn update(
    state: &mut MaterialsState,
    project: &mut Project,
    message: MaterialsMessage,
) -> Task<AppMessage> {
    match message {
        MaterialsMessage::UpdateName(i, res_id, n) => {
            match project.resource_mut(res_id).unwrap() {
                Resource::Material(material) => material.update_name(&n),
                _ => panic!(),
            }
            state.repr[i].update_name(n);
            Task::none()
        }
        MaterialsMessage::UpdateQuantity(i, res_id, q) => {
            if let Ok(quantity) = q.parse::<u16>() {
                match project.resource_mut(res_id).unwrap() {
                    Resource::Material(material) => material.update_quantity(quantity),
                    _ => panic!(),
                }
                state.repr[i].update_is_quantity_err(false);
            } else if q.is_empty() {
                match project.resource_mut(res_id).unwrap() {
                    Resource::Material(material) => material.remove_quantity(),
                    _ => panic!(),
                }
                state.repr[i].update_is_quantity_err(false);
            } else {
                state.repr[i].update_is_quantity_err(true);
            }
            state.repr[i].update_quantity(q);
            Task::none()
        }
        MaterialsMessage::UpdateCost(i, res_id, c) => {
            if let Ok(cost) = c.parse::<f32>() {
                match project.resource_mut(res_id).unwrap() {
                    Resource::Material(material) => {
                        material.update_cost_per_unit((cost * 100.) as u16)
                    }
                    _ => panic!(),
                }
                state.repr[i].update_is_cost_err(false);
            } else if c.is_empty() {
                match project.resource_mut(res_id).unwrap() {
                    Resource::Material(material) => material.remove_cost_per_unit(),
                    _ => panic!(),
                }
                state.repr[i].update_is_cost_err(false);
            } else {
                state.repr[i].update_is_cost_err(true);
            }
            state.repr[i].update_cost(c);
            Task::none()
        }
        MaterialsMessage::Typeselected(i, res_id, s) => {
            match (&mut state.repr[i], s) {
                (Repr::Consumable(_), Selection::Consumable) => {}
                (Repr::Consumable(consumable_repr), Selection::NonConsumable) => {
                    state.repr[i] = Repr::NonConsumable((consumable_repr).clone().into());
                    project.res_into_nonconsumable(res_id).unwrap();
                }
                (Repr::NonConsumable(non_consumable_repr), Selection::Consumable) => {
                    state.repr[i] = Repr::Consumable((non_consumable_repr).clone().into());
                    project.res_into_consumable(res_id).unwrap();
                }
                (Repr::NonConsumable(_), Selection::NonConsumable) => {}
            }
            Task::none()
        }
        MaterialsMessage::UpdateNewName(n) => {
            state.new_material_name = n;
            Task::none()
        }
        MaterialsMessage::CreateNewMaterial => {
            if state.new_material_name.is_empty() {
                return Task::none();
            }

            let material = Material::NonConsumable(NonConsumable::new(&state.new_material_name));
            project.add_resource(Resource::Material(material));
            let res_id = project.resources().len() - 1;
            state.repr.push(Repr::NonConsumable(NonConsumableRepr {
                res_id,
                name: state.new_material_name.clone(),
                ..Default::default()
            }));

            state.new_material_name = "".to_owned();
            Task::none()
        }
        MaterialsMessage::DeleteMaterial(i, res_id) => {
            state.repr.remove(i);
            project.rm_resource(res_id);
            Task::perform(async move { res_id }, AppMessage::ResourceDeleted)
        }
        MaterialsMessage::ResourceDeleted(res_id) => {
            state.repr.iter_mut().for_each(|r| {
                if r.res_id() > res_id {
                    r.update_res_id(r.res_id() - 1);
                }
            });
            Task::none()
        }
    }
}

pub fn view(state: &MaterialsState) -> Element<'_, MaterialsMessage> {
    let headers = Row::new()
        .push(data_label("Resource ID"))
        .push(data_label("Name"))
        .push(data_label("Type"))
        .push(data_label("Quantity"))
        .push(data_label("Cost"));

    let options = [Selection::Consumable, Selection::NonConsumable];
    let content_rows: Vec<Element<'_, _>> = state
        .repr
        .iter()
        .enumerate()
        .map(|(i, r)| match r {
            Repr::Consumable(consumable) => Row::new()
                .push(data_label(consumable.res_id))
                .push(
                    data_cell("Stimpack", &consumable.name, false)
                        .on_input(move |n| MaterialsMessage::UpdateName(i, consumable.res_id, n)),
                )
                .push(
                    pick_list(options, Some(Selection::Consumable), move |s| {
                        MaterialsMessage::Typeselected(i, consumable.res_id, s)
                    })
                    .width(constants::WIDTH),
                )
                .push(
                    data_cell("1", &consumable.quantity, consumable.is_quantity_err).on_input(
                        move |q| MaterialsMessage::UpdateQuantity(i, consumable.res_id, q),
                    ),
                )
                .push(
                    data_cell("20", &consumable.cost_per_unit, consumable.is_cost_err)
                        .on_input(move |c| MaterialsMessage::UpdateCost(i, consumable.res_id, c)),
                )
                .push(Space::new(constants::WIDTH, constants::HEIGHT))
                .into(),
            Repr::NonConsumable(non_consumable) => Row::new()
                .push(data_label(non_consumable.res_id))
                .push(
                    data_cell("Crowbar", &non_consumable.name, false).on_input(move |n| {
                        MaterialsMessage::UpdateName(i, non_consumable.res_id, n)
                    }),
                )
                .push(
                    pick_list(options, Some(Selection::NonConsumable), move |s| {
                        MaterialsMessage::Typeselected(i, non_consumable.res_id, s)
                    })
                    .width(constants::WIDTH),
                )
                .push(
                    data_cell(
                        "1",
                        &non_consumable.quantity,
                        non_consumable.is_quantity_err,
                    )
                    .on_input(move |q| {
                        MaterialsMessage::UpdateQuantity(i, non_consumable.res_id, q)
                    }),
                )
                .push(
                    data_cell(
                        "20",
                        &non_consumable.hourly_rate,
                        non_consumable.is_rate_err,
                    )
                    .on_input(move |c| MaterialsMessage::UpdateCost(i, non_consumable.res_id, c)),
                )
                .push(
                    button("Del")
                        .on_press(MaterialsMessage::DeleteMaterial(i, non_consumable.res_id))
                        .width(constants::WIDTH),
                )
                .into(),
        })
        .collect();

    let new_row = Row::new().push(data_label("")).push(
        data_cell(
            "Crowbar",
            &state.new_material_name,
            state.is_new_material_err,
        )
        .on_input(MaterialsMessage::UpdateNewName)
        .on_submit(MaterialsMessage::CreateNewMaterial),
    );

    Column::new()
        .push(headers)
        .extend(content_rows)
        .push(new_row)
        .height(Length::Shrink)
        .into()
}
