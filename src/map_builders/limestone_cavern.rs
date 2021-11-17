use rltk::RandomNumberGenerator;
use crate::map_builders::area_starting_points::{AreaStartingPosition, XStart, YStart};
use crate::map_builders::{BuilderChain, BuilderMap, MetaMapBuilder};
use crate::map_builders::cull_unreachable::CullUnreachable;
use crate::map_builders::distant_exit::DistantExit;
use crate::map_builders::drunkard::DrunkardsWalkBuilder;
use crate::map_builders::voronoi_spawning::VoronoiSpawning;
use crate::TileType;

pub fn limestone_builder(new_depth: i32, _rng: &mut RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Limestone Caverns");
    chain.start_with(DrunkardsWalkBuilder::winding_passages());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain.with(CaveDecorator::new());
    chain
}

pub struct CaveDecorator {}

impl MetaMapBuilder for CaveDecorator {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CaveDecorator {
    #[allow(dead_code)]
    pub fn new() -> Box<CaveDecorator> {
        Box::new(CaveDecorator{})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let old_map = build_data.map.clone();
        for (index, tt) in build_data.map.tiles.iter_mut().enumerate() {
            // Gravel Spawning
            if *tt == TileType::Floor && rng.roll_dice(1, 6) == 1 {
                *tt = TileType::Gravel;
            }
            // Spawn passable pools
            else if *tt == TileType::Floor && rng.roll_dice(1, 10) == 1 {
                *tt = TileType::ShallowWater;
            } else if *tt == TileType::Wall {
                // Spawn deep pools and stalactites
                let mut neighbors = 0;
                let x = index as i32 % old_map.width;
                let y = index as i32 / old_map.width;
                if x > 0 && old_map.tiles[index - 1] == TileType::Wall { neighbors += 1; }
                if x < old_map.width -2 && old_map.tiles[index + 1] == TileType::Wall { neighbors += 1; }
                if y > 0 && old_map.tiles[index - old_map.width as usize] == TileType::Wall { neighbors += 1; }
                if y < old_map.height - 2 && old_map.tiles[index + old_map.width as usize] == TileType::Wall { neighbors += 1; }
                if neighbors == 2 {
                    *tt = TileType::DeepWater;
                } else if neighbors == 1 {
                    let roll = rng.roll_dice(1, 4);
                    match roll {
                        1 => *tt = TileType::Stalactite,
                        2 => *tt = TileType::Stalagmite,
                        _ => {}
                    }
                }
            }
        }
        build_data.take_snapshot();
        build_data.map.outdoors = false;
    }
}
