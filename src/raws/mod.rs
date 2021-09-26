use std::sync::Mutex;

use serde::Deserialize;

use item_structs::Item;
use mob_structs::Mob;
pub use rawmaster::*;
use spawn_table_structs::SpawnTableEntry;
use crate::raws::prop_structs::Prop;

mod item_structs;
mod mob_structs;
mod spawn_table_structs;
mod rawmaster;
mod prop_structs;

#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items : Vec<Item>,
    pub mobs : Vec<Mob>,
    pub props: Vec<Prop>,
    pub spawn_table : Vec<SpawnTableEntry>,
}

#[derive(Deserialize, Debug)]
pub struct Renderable {
    pub glyph: String,
    pub fg : String,
    pub bg : String,
    pub order: i32
}

lazy_static! {
    pub static ref RAWS : Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}

rltk::embedded_resource!(RAW_FILE, "../../raws/spawns.json");

pub fn load_raws() {
    rltk::link_resource!(RAW_FILE, "../../raws/spawns.json");

    // Retrieve the raw data as an array of u8 (8-bit unsigned chars)
    let raw_data = rltk::embedding::EMBED
        .lock()
        .get_resource("../../raws/spawns.json".to_string())
        .unwrap();
    let raw_string = std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");
    let decoder : Raws = serde_json::from_str(&raw_string).expect("Unable to parse JSON");

    RAWS.lock().unwrap().load(decoder);
}

