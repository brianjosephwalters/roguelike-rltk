use crate::map_builders::{BuilderMap, MetaMapBuilder};
use rltk::RandomNumberGenerator;
use crate::TileType;

pub struct DistantExit {}

impl MetaMapBuilder for DistantExit {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DistantExit {
    pub fn new() -> Box<DistantExit> {
        println!("DistantExit");
        Box::new(DistantExit{})
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_index = build_data.map.xy_index(
            starting_pos.x,
            starting_pos.y
        );

        build_data.map.populate_blocked();
        let map_starts: Vec<usize> = vec![start_index];
        let dijkstra_map = rltk::DijkstraMap::new(build_data.map.width as usize, build_data.map.height as usize, &map_starts, &build_data.map, 1000.0);
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                if distance_to_start != std::f32::MAX {
                    // If it is further away than our exit candidate, move the exit
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }

        // Place a staircase
        let stairs_index = exit_tile.0;
        build_data.map.tiles[stairs_index] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}