use rltk::RandomNumberGenerator;
use crate::map_builders::{BuilderMap, MetaMapBuilder};
use crate::{TileType, Rect};

pub struct RoomCornerRounder {}

impl MetaMapBuilder for RoomCornerRounder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomCornerRounder {
    pub fn new() -> Box<RoomCornerRounder> {
        println!("RoomCornerRounder");
        Box::new(RoomCornerRounder{})
    }

    fn fill_if_corner(&mut self, x: i32, y: i32, build_data: &mut BuilderMap) {
        let w = build_data.map.width;
        let h = build_data.map.height;
        let index = build_data.map.xy_index(x, y);
        let mut neighbor_walls = 0;
        if x > 0 && build_data.map.tiles[index - 1] == TileType::Wall { neighbor_walls += 1; }
        if y > 0 && build_data.map.tiles[index - w as usize] == TileType::Wall { neighbor_walls += 1; }
        if x < w - 2 && build_data.map.tiles[index + 1] == TileType::Wall { neighbor_walls += 1; }
        if y < h - 2 && build_data.map.tiles[index + w as usize] == TileType::Wall { neighbor_walls += 1; }

        if neighbor_walls == 2 {
            build_data.map.tiles[index] = TileType::Wall;
        }
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Room Rounding requires a builder with room structure.");
        }

        for room in rooms.iter() {
            self.fill_if_corner(room.x1 + 1, room.y1 + 1, build_data);
            self.fill_if_corner(room.x2, room.y1 + 1, build_data);
            self.fill_if_corner(room.x1 + 1, room.y2, build_data);
            self.fill_if_corner(room.x2, room.y2, build_data);

            build_data.take_snapshot();
        }
    }
}
