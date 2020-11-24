pub mod simple_map;
pub mod common;

use super::{Map, Position};
use self::simple_map::SimpleMapBuilder;


trait MapBuilder {
    fn build(new_depth: i32) -> (Map, Position);
}

pub fn build_random_map(new_depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(new_depth)
}