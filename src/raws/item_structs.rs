use serde::{Deserialize};
use std::collections::HashMap;
use super::Renderable;

#[derive(Deserialize, Debug)]
pub struct Item {
    pub name : String,
    pub renderable : Option<Renderable>,
    pub consumable : Option<Consumable>,
    pub weapon : Option<Weapon>,
    pub wearable : Option<Wearable>
}

#[derive(Deserialize, Debug)]
pub struct Consumable {
    pub effects : HashMap<String, String>
}

#[derive(Deserialize, Debug)]
pub struct Weapon {
    pub range: String,
    pub attribute: String,
    pub base_damage: String,
    pub hit_bonus: i32
}

#[derive(Deserialize, Debug)]
pub struct Wearable {
    pub armor_class: f32,
    pub slot : String
}

#[derive(Deserialize, Debug)]
pub struct Shield {
    pub defense_bonus: i32
}
