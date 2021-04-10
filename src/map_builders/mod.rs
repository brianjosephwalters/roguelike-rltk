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
mod room_based_spawner;
mod room_based_starting_position;
mod room_based_stairs;
mod area_starting_points;
mod cull_unreachable;
mod voronoi_spawning;
mod distant_exit;

use super::{Map, Position, World};
use self::simple_map::SimpleMapBuilder;
use self::bsp_dungeon::BspDungeonBuilder;
use self::bsp_interior::BspInteriorBuilder;
use self::cellular_automata::CellularAutomataBuilder;
use self::drunkard::DrunkardsWalkBuilder;
use self::maze::MazeBuilder;
use crate::map_builders::dla::DLABuilder;
use crate::map_builders::voronoi::VoronoiCellBuilder;
use crate::map_builders::waveform_collapse::WaveformCollapseBuilder;
use crate::map_builders::prefab_builder::PrefabBuilder;
use crate::{spawner, Rect, SHOW_MAPGEN_VISUALIZER};
use crate::map_builders::room_based_spawner::RoomBasedSpawner;
use crate::map_builders::room_based_starting_position::RoomBasedStartingPosition;
use crate::map_builders::room_based_stairs::RoomBasedStairs;
use crate::map_builders::voronoi_spawning::VoronoiSpawning;
use crate::map_builders::distant_exit::DistantExit;
use crate::map_builders::cull_unreachable::CullUnreachable;
use rltk::RandomNumberGenerator;
use crate::map_builders::area_starting_points::{AreaStartingPosition, XStart, YStart};

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new(new_depth: i32) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth),
                starting_position: None,
                rooms: None,
                history: Vec::new(),
            }
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder."),
        };
    }

    pub fn with(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain wihtout a starting builder."),
            Some(starter) => {
                // Build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // Build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

fn random_initial_builder(rng: &mut RandomNumberGenerator) -> (Box<dyn InitialMapBuilder>, bool) {
    let builder = rng.roll_dice(1, 17);
    let result: (Box<dyn InitialMapBuilder>, bool);
    match builder {
        1 => result = (BspDungeonBuilder::new(), true),
        2 => result = (BspInteriorBuilder::new(), true),
        3 => result = (CellularAutomataBuilder::new(), false),
        4 => result = (DrunkardsWalkBuilder::open_area(), false),
        5 => result = (DrunkardsWalkBuilder::open_halls(), false),
        6 => result = (DrunkardsWalkBuilder::winding_passages(), false),
        7 => result = (DrunkardsWalkBuilder::fat_passages(), false),
        8 => result = (DrunkardsWalkBuilder::fearful_symmetry(), false),
        9 => result = (MazeBuilder::new(), false),
        10 => result = (DLABuilder::walk_inwards(), false),
        11 => result = (DLABuilder::walk_outwards(), false),
        12 => result = (DLABuilder::central_attractor(), false),
        13 => result = (DLABuilder::insectoid(), false),
        14 => result = (VoronoiCellBuilder::pythagoras(), false),
        15 => result = (VoronoiCellBuilder::manhattan(), false),
        16 => result = (PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED), false),
        _ => result = (SimpleMapBuilder::new(), true)
    }
    result
}

pub fn random_builder(new_depth: i32, rng: &mut RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    let (random_starter, has_rooms) = random_initial_builder(rng);
    print!("rooms: {}: ", has_rooms);
    builder.start_with(random_starter);
    if has_rooms {
        builder.with(RoomBasedSpawner::new());
        builder.with(RoomBasedStairs::new());
        builder.with(RoomBasedStartingPosition::new());
    }
    else {
        builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
        builder.with(CullUnreachable::new());
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }

    if rng.roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());
    }

    if rng.roll_dice(1, 20) == 1 {
        builder.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::UNDERGROUND_FORT));
    }

    builder.with(PrefabBuilder::vaults());

    builder
}
