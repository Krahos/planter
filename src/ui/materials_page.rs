use std::fmt::Display;

use iced::{
    Element, Length,
    widget::{Column, Row, pick_list},
};
use planter_core::{
    project::Project,
    resources::{Material, NonConsumable, Resource},
};

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
    name: String,
    quantity: String,
    is_quantity_err: bool,
    cost_per_unit: String,
    is_cost_err: bool,
}

#[derive(Debug, Default, Clone)]
struct NonConsumableRepr {
    name: String,
    quantity: String,
    is_quantity_err: bool,
    hourly_rate: String,
    is_rate_err: bool,
}

impl Into<NonConsumableRepr> for ConsumableRepr {
    fn into(self) -> NonConsumableRepr {
        NonConsumableRepr {
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
            name: self.name,
            quantity: self.quantity,
            is_quantity_err: self.is_quantity_err,
            cost_per_unit: "".to_owned(),
            is_cost_err: false,
        }
    }
}

impl Repr {
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
enum Selection {
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
    UpdateName(usize, String),
    UpdateQuantity(usize, String),
    UpdateCost(usize, String),
    Typeselected(usize, Selection),
    UpdateNewName(String),
    CreateNewMaterial,
    DeleteMaterial(usize),
}

pub fn update(state: &mut MaterialsState, project: &mut Project, message: MaterialsMessage) {
    match message {
        MaterialsMessage::UpdateName(i, n) => {
            match project.resource_mut(i).unwrap() {
                Resource::Material(material) => material.update_name(&n),
                _ => panic!(),
            }
            state.repr[i].update_name(n);
        }
        MaterialsMessage::UpdateQuantity(i, q) => {
            if let Ok(quantity) = q.parse::<u16>() {
                match project.resource_mut(i).unwrap() {
                    Resource::Material(material) => material.update_quantity(quantity),
                    _ => panic!(),
                }
                state.repr[i].update_is_quantity_err(false);
            } else if q.is_empty() {
                match project.resource_mut(i).unwrap() {
                    Resource::Material(material) => material.remove_quantity(),
                    _ => panic!(),
                }
                state.repr[i].update_is_quantity_err(false);
            } else {
                state.repr[i].update_is_quantity_err(true);
            }
            state.repr[i].update_quantity(q);
        }
        MaterialsMessage::UpdateCost(i, c) => {
            if let Ok(cost) = c.parse::<f32>() {
                match project.resource_mut(i).unwrap() {
                    Resource::Material(material) => {
                        material.update_cost_per_unit((cost * 100.) as u16)
                    }
                    _ => panic!(),
                }
                state.repr[i].update_is_cost_err(false);
            } else if c.is_empty() {
                match project.resource_mut(i).unwrap() {
                    Resource::Material(material) => material.remove_cost_per_unit(),
                    _ => panic!(),
                }
                state.repr[i].update_is_cost_err(false);
            } else {
                state.repr[i].update_is_cost_err(true);
            }
            state.repr[i].update_cost(c);
        }
        MaterialsMessage::Typeselected(i, s) => match (&mut state.repr[i], s) {
            (Repr::Consumable(_), Selection::Consumable) => {}
            (Repr::Consumable(consumable_repr), Selection::NonConsumable) => {
                state.repr[i] = Repr::NonConsumable((consumable_repr).clone().into());
            }
            (Repr::NonConsumable(non_consumable_repr), Selection::Consumable) => {
                state.repr[i] = Repr::Consumable((non_consumable_repr).clone().into());
            }
            (Repr::NonConsumable(_), Selection::NonConsumable) => {}
        },
        MaterialsMessage::UpdateNewName(n) => {
            state.new_material_name = n;
        }
        MaterialsMessage::CreateNewMaterial => {
            if state.new_material_name.is_empty() {
                return;
            }

            let material = Material::NonConsumable(NonConsumable::new(&state.new_material_name));
            project.add_resource(Resource::Material(material));
            state.repr.push(Repr::NonConsumable(NonConsumableRepr {
                name: state.new_material_name.clone(),
                ..Default::default()
            }));

            state.new_material_name = "".to_owned();
        }
        MaterialsMessage::DeleteMaterial(_) => todo!(),
    }
}

pub fn view(state: &MaterialsState) -> Element<'_, MaterialsMessage> {
    let headers = Row::new()
        .push(data_label("Index"))
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
                .push(data_label(i))
                .push(
                    data_cell("Stimpack", &consumable.name, false)
                        .on_input(move |n| MaterialsMessage::UpdateName(i, n)),
                )
                .push(
                    pick_list(options, Some(Selection::Consumable), move |s| {
                        MaterialsMessage::Typeselected(i, s)
                    })
                    .width(100),
                )
                .push(
                    data_cell("1", &consumable.quantity, consumable.is_quantity_err)
                        .on_input(move |q| MaterialsMessage::UpdateQuantity(i, q)),
                )
                .push(
                    data_cell("20", &consumable.cost_per_unit, consumable.is_cost_err)
                        .on_input(move |c| MaterialsMessage::UpdateCost(i, c)),
                )
                .into(),
            Repr::NonConsumable(non_consumable) => Row::new()
                .push(data_label(i))
                .push(
                    data_cell("Crowbar", &non_consumable.name, false)
                        .on_input(move |n| MaterialsMessage::UpdateName(i, n)),
                )
                .push(
                    pick_list(options, Some(Selection::NonConsumable), move |s| {
                        MaterialsMessage::Typeselected(i, s)
                    })
                    .width(100),
                )
                .push(
                    data_cell(
                        "1",
                        &non_consumable.quantity,
                        non_consumable.is_quantity_err,
                    )
                    .on_input(move |q| MaterialsMessage::UpdateQuantity(i, q)),
                )
                .push(
                    data_cell(
                        "20",
                        &non_consumable.hourly_rate,
                        non_consumable.is_rate_err,
                    )
                    .on_input(move |c| MaterialsMessage::UpdateCost(i, c)),
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
