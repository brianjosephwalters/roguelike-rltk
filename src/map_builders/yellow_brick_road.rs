use rltk::{RandomNumberGenerator, DistanceAlg, Point};
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::{map, TileType};

pub struct YellowBrickRoad {}

impl MetaMapBuilder for YellowBrickRoad {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl YellowBrickRoad {

    #[allow(unused)]
    pub fn new() -> Box<YellowBrickRoad> {
        Box::new(YellowBrickRoad{})
    }

    fn build(&mut self, rng:&mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_index = build_data.map.xy_index(starting_pos.x, starting_pos.y);

        let (end_x, end_y) = self.find_exit(build_data, build_data.map.width - 2, build_data.map.height / 2);
        let end_index = build_data.map.xy_index(end_x, end_y);
        build_data.map.tiles[end_index] = TileType::DownStairs;

        build_data.map.populate_blocked();
        println!("a_star_search for path in yellow_brick_roads.rs");
        let path = rltk::a_star_search(start_index, end_index, &mut build_data.map);

        for index in path.steps.iter() {
            let x = *index as i32 % build_data.map.width;
            let y = *index as i32 / build_data.map.width;
            self.paint_road(build_data, x ,y);
            self.paint_road(build_data, x - 1, y);
            self.paint_road(build_data, x + 1, y);
            self.paint_road(build_data, x, y - 1);
            self.paint_road(build_data, x, y + 1);
        }

        // Place exit
        let exit_dir = rng.roll_dice(1, 2);
        let (seed_x, seed_y, stream_start_x, stream_start_y) = if exit_dir == 1 {
            (build_data.map.width - 1, 1, 0, build_data.map.height - 1)
        } else {
            (build_data.map.width - 1, build_data.map.height - 1, 1, build_data.height - 1)
        };

        let (stairs_x, stairs_y) = self.find_exit(build_data, seed_x, seed_y);
        let stairs_index = build_data.map.xy_index(stairs_x, stairs_y);
        build_data.take_snapshot();

        let (stream_x, stream_y) = self.find_exit(build_data, stream_start_x, stream_start_y);
        let stream_index = build_data.map.xy_index(stream_x, stream_y) as usize;
        println!("a_star_search for stream in yellow_brick_roads.rs");
        let stream = rltk::a_star_search(stairs_index, stream_index, &mut build_data.map);
        for tile in stream.steps.iter() {
            if build_data.map.tiles[*tile as usize] == TileType::Floor {
                build_data.map.tiles[*tile as usize] = TileType::ShallowWater;
            }
        }
        build_data.map.tiles[stairs_index] = TileType::DownStairs;
        build_data.take_snapshot();
    }

    fn find_exit(&self, build_data: &mut BuilderMap, seed_x: i32, seed_y: i32) -> (i32, i32) {
        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (index, tiletype) in build_data.map.tiles.iter().enumerate() {
            if map::tile_walkable(*tiletype) {
                available_floors.push((
                    index,
                    DistanceAlg::PythagorasSquared.distance2d(
                        Point::new(index as i32 % build_data.map.width, index as i32 / build_data.map.width),
                        Point::new(seed_x, seed_y)
                    )
                ));
            }
        }

        if available_floors.is_empty() {
            panic!("No valid floords to start on!");
        }

        available_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let end_x = available_floors[0].0 as i32 % build_data.map.width;
        let end_y = available_floors[0].0 as i32 / build_data.map.width;
        (end_x, end_y)
    }

    fn paint_road(&self, build_data: &mut BuilderMap, x: i32, y: i32) {
        if x < 1
            || x > build_data.map.width - 2
            || y < 1
            || y > build_data.map.height - 2 {
            return;
        }

        let index = build_data.map.xy_index(x, y);
        if build_data.map.tiles[index] != TileType::DownStairs {
            build_data.map.tiles[index] = TileType::Road;
        }
    }

}
