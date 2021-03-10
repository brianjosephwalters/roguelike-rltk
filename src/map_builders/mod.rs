pub mod simple_map;
pub mod bsp_dungeon;
pub mod bsp_interior;
pub mod cellular_automata;
pub mod common;

use super::{Map, Position, World};
use self::simple_map::SimpleMapBuilder;
use self::bsp_dungeon::BspDungeonBuilder;
use self::bsp_interior::BspInteriorBuilder;
use self::cellular_automata::CellularAutomataBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs : &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth : i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 2);
    match builder {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        2 => Box::new(BspInteriorBuilder::new(new_depth)),
        3 => Box::new(CellularAutomataBuilder::new(new_depth)),
        _ => Box::new(SimpleMapBuilder::new(new_depth))
    }
}