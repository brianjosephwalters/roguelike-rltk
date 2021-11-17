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
mod room_exploder;
mod room_corner_rounding;
mod room_cooridors_dogleg;
mod room_corridors_bsp;
mod room_sorter;
mod room_draw;
mod rooms_corridors_nearest;
mod rooms_corridors_lines;
mod room_corridor_spawner;
mod door_placement;
mod town;
mod forest;
mod yellow_brick_road;
mod limestone_cavern;


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
use crate::map_builders::room_exploder::RoomExploder;
use crate::map_builders::room_corner_rounding::RoomCornerRounder;
use crate::map_builders::room_cooridors_dogleg::DoglegCorridors;
use crate::map_builders::room_corridors_bsp::BspCorridors;
use crate::map_builders::room_sorter::{RoomSorter, RoomSort};
use crate::map_builders::room_draw::RoomDrawer;
use crate::map_builders::rooms_corridors_nearest::NearestCorridors;
use crate::map_builders::rooms_corridors_lines::StraightLineCorridors;
use crate::map_builders::room_corridor_spawner::CorridorSpawner;
use crate::map_builders::door_placement::DoorPlacement;
use crate::map_builders::town::town_builder;
use crate::map_builders::forest::forest_builder;
use crate::map_builders::limestone_cavern::limestone_builder;

pub struct BuilderMap {
    pub spawn_list: Vec<(usize, String)>,
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub corridors: Option<Vec<Vec<usize>>>,
    pub history: Vec<Map>,
    pub width: i32,
    pub height: i32
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
    pub fn new<S: ToString>(new_depth: i32, width: i32, height: i32, name: S) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth, width, height, name),
                starting_position: None,
                rooms: None,
                corridors: None,
                history: Vec::new(),
                width,
                height
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

#[allow(unused)]
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

pub fn level_builder(new_depth: i32, rng: &mut RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    rltk::console::log(format!("Depth: {}", new_depth));
    match new_depth {
        1 => town_builder(new_depth, rng, width, height),
        2 => forest_builder(new_depth, rng, width, height),
        3 => limestone_builder(new_depth, rng, width, height),
        _ => random_builder(new_depth, rng, width, height)
    }
}

pub fn random_builder(new_depth: i32, rng: &mut RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height, "New Map");
    let type_roll = rng.roll_dice(1, 2);
    match type_roll {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(rng, &mut builder)
    }

    if rng.roll_dice(1, 3) == 1 {
        builder.with(WaveformCollapseBuilder::new());

        // Now set the start to a random starting area
        let (start_x, start_y) = random_start_position(rng);
        builder.with(AreaStartingPosition::new(start_x, start_y));

        // Setup an exit and mob spawns
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }

    if rng.roll_dice(1, 20) == 1 {
        builder.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::UNDERGROUND_FORT));
    }

    builder.with(DoorPlacement::new());
    builder.with(PrefabBuilder::vaults());

    // let mut builder = BuilderChain::new(new_depth);
    // builder.start_with(SimpleMapBuilder::new());
    // builder.with(RoomDrawer::new());
    // builder.with(RoomSorter::new(RoomSort::LEFTMOST));
    // builder.with(StraightLineCorridors::new());
    // builder.with(RoomBasedSpawner::new());
    // builder.with(CorridorSpawner::new());
    // builder.with(RoomBasedStairs::new());
    // builder.with(RoomBasedStartingPosition::new());
    // builder.with(DoorPlacement::new());

    builder
}

fn random_start_position(rng: &mut rltk::RandomNumberGenerator) -> (XStart, YStart) {
    let x;
    let x_roll = rng.roll_dice(1, 3);
    match x_roll {
        1 => x = XStart::LEFT,
        2 => x = XStart::CENTER,
        _ => x = XStart::RIGHT
    }

    let y;
    let y_roll = rng.roll_dice(1, 3);
    match y_roll {
        1 => y = YStart::BOTTOM,
        2 => y = YStart::CENTER,
        _ => y = YStart::TOP
    }

    (x, y)
}

fn random_room_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    println!("Random Room Builder Path:");
    let build_room = rng.roll_dice(1, 3);
    match build_room {
        1 => builder.start_with(SimpleMapBuilder::new()),
        2 => builder.start_with(BspDungeonBuilder::new()),
        _ => builder.start_with(BspInteriorBuilder::new())
    }

    if build_room != 3 {
        let sort_roll = rng.roll_dice(1, 5);
        match sort_roll {
            1 => builder.with(RoomSorter::new(RoomSort::LEFTMOST)),
            2 => builder.with(RoomSorter::new(RoomSort::RIGHTMOST)),
            3 => builder.with(RoomSorter::new(RoomSort::TOPMOST)),
            4 => builder.with(RoomSorter::new(RoomSort::BOTTOMMOST)),
            _ => builder.with(RoomSorter::new(RoomSort::CENTRAL)),
        }

        builder.with(RoomDrawer::new());

        let corridor_roll = rng.roll_dice(1, 4);
        match corridor_roll {
            1 => builder.with(DoglegCorridors::new()),
            2 => builder.with(NearestCorridors::new()),
            3 => builder.with(StraightLineCorridors::new()),
            _ => builder.with(BspCorridors::new())
        }

        let cspawner_roll = rng.roll_dice(1, 2);
        if cspawner_roll == 1 {
            builder.with(CorridorSpawner::new());
        }

        let modifier_roll = rng.roll_dice(1, 6);
        match modifier_roll {
            1 => builder.with(RoomExploder::new()),
            2 => builder.with(RoomCornerRounder::new()),
            _ => {}
        }
    }

    let start_roll = rng.roll_dice(1, 2);
    match start_roll {
        1 => builder.with(RoomBasedStartingPosition::new()),
        _ => {
            let (start_x, start_y) = random_start_position(rng);
            builder.with(AreaStartingPosition::new(start_x, start_y))
        }
    }

    let exit_roll = rng.roll_dice(1, 2);
    match exit_roll {
        1 => builder.with(RoomBasedStairs::new()),
        _ => builder.with(DistantExit::new()),
    }

    let spawn_roll = rng.roll_dice(1, 2);
    match spawn_roll {
        1 => builder.with(RoomBasedSpawner::new()),
        _ => builder.with(VoronoiSpawning::new())
    }
}

fn random_shape_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    println!("Random Shape Builder Path:");
    let builder_roll = rng.roll_dice(1, 16);
    match builder_roll {
        1 => builder.start_with(CellularAutomataBuilder::new()),
        2 => builder.start_with(DrunkardsWalkBuilder::open_area()),
        3 => builder.start_with(DrunkardsWalkBuilder::open_halls()),
        4 => builder.start_with(DrunkardsWalkBuilder::winding_passages()),
        5 => builder.start_with(DrunkardsWalkBuilder::fat_passages()),
        6 => builder.start_with(DrunkardsWalkBuilder::fearful_symmetry()),
        7 => builder.start_with(MazeBuilder::new()),
        8 => builder.start_with(DLABuilder::walk_inwards()),
        9 => builder.start_with(DLABuilder::walk_outwards()),
        10 => builder.start_with(DLABuilder::central_attractor()),
        11 => builder.start_with(DLABuilder::insectoid()),
        12 => builder.start_with(VoronoiCellBuilder::pythagoras()),
        13 => builder.start_with(VoronoiCellBuilder::manhattan()),
        _ => builder.start_with(PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED)),
    }

    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    builder.with(CullUnreachable::new());

    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPosition::new(start_x, start_y));

    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
}
