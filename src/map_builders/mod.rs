pub mod simple_map;
pub mod bsp_dungeon;
pub mod bsp_interior;
pub mod cellular_automata;
pub mod drunkard;
pub mod common;
pub mod maze;
pub mod dla;
pub mod voronoi;
pub mod waveform_collapse;
mod prefab_builder;

use super::{Map, Position, World};
use self::simple_map::SimpleMapBuilder;
use self::bsp_dungeon::BspDungeonBuilder;
use self::bsp_interior::BspInteriorBuilder;
use self::cellular_automata::CellularAutomataBuilder;
use self::drunkard::{DrunkardsWalkBuilder, DrunkardSettings, DrunkSpawnMode};
use self::maze::MazeBuilder;
use crate::map_builders::dla::DLABuilder;
use crate::map_builders::voronoi::VoronoiBuilder;
use crate::map_builders::waveform_collapse::WaveformCollapseBuilder;
use crate::map_builders::prefab_builder::PrefabBuilder;
use crate::spawner;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
    fn get_spawn_list(&self) -> &Vec<(usize, String)>;
    fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.get_spawn_list().iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub fn random_builder(new_depth : i32) -> Box<dyn MapBuilder> {
    // let mut rng = rltk::RandomNumberGenerator::new();
    // let builder = rng.roll_dice(1, 16);
    // let mut result: Box<dyn MapBuilder>;
    // match builder {
    //     1 => { result = Box::new(BspDungeonBuilder::new(new_depth)); }
    //     2 => { result = Box::new(BspInteriorBuilder::new(new_depth)); }
    //     3 => { result = Box::new(CellularAutomataBuilder::new(new_depth)); }
    //     4 => { result = Box::new(DrunkardsWalkBuilder::open_area(new_depth)); }
    //     5 => { result = Box::new(DrunkardsWalkBuilder::open_halls(new_depth)); }
    //     6 => { result = Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)); }
    //     7 => { result = Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)); }
    //     8 => { result = Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)); }
    //     9 => { result = Box::new(MazeBuilder::new(new_depth)); }
    //     10 => { result = Box::new(DLABuilder::walk_inwards(new_depth)); }
    //     11 => { result = Box::new(DLABuilder::walk_outwards(new_depth)); }
    //     12 => { result = Box::new(DLABuilder::central_attractor(new_depth)); }
    //     13 => { result = Box::new(DLABuilder::insectoid(new_depth)); }
    //     14 => { result = Box::new(VoronoiBuilder::pythagoras(new_depth)); }
    //     15 => { result = Box::new(VoronoiBuilder::manhattan(new_depth)); }
    //     _ => { result = Box::new(SimpleMapBuilder::new(new_depth)); }
    // }
    //
    // if rng.roll_dice(1, 3) == 1 {
    //     result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
    // }
    //
    // result
    Box::new(PrefabBuilder::new(
        new_depth,
        Some(
            Box::new(
                CellularAutomataBuilder::new(new_depth)
            )
        )
    ))
}