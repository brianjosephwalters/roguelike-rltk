pub mod simple_map;
pub mod common;

use super::{Map, Position, World};
use self::simple_map::SimpleMapBuilder;


pub trait MapBuilder {
    fn build_map(&mut self, new_depth: i32) -> (Map, Position);
    fn spawn_entities(&mut self, map : &Map, ecs : &mut World, new_depth: i32);
}

pub fn random_builder() -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder{})
}