use crate::map_builders::{MetaMapBuilder, BuilderMap};
use rltk::RandomNumberGenerator;
use crate::TileType;

pub struct CullUnreachable {}

impl MetaMapBuilder for CullUnreachable {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CullUnreachable {
    pub fn new() -> Box<CullUnreachable> {
        Box::new(CullUnreachable{})
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
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                // We can't get to this tile - so we'll make it a wall
                if distance_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                }
            }
        }
    }
}