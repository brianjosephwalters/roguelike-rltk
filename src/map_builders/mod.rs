pub mod simple_map;
pub mod common;

use super::{Map, Position, World};
use self::simple_map::SimpleMapBuilder;


pub trait MapBuilder {
    fn build_map(&mut self) -> (Map, Position);
    fn spawn_entities(&mut self, map : &Map, ecs : &mut World);
}

pub fn random_builder(new_depth : i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(new_depth))
}