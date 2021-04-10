use crate::TileType;
use rltk::RandomNumberGenerator;
use crate::map_builders::{InitialMapBuilder, BuilderMap};

pub struct CellularAutomataBuilder {}

impl InitialMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CellularAutomataBuilder {
    pub fn new() -> Box<CellularAutomataBuilder> {
        Box::new(CellularAutomataBuilder {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // First, we completely randomize the map, setting 55% to be floor.
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let index = build_data.map.xy_index(x, y);
                if roll > 55 { build_data.map.tiles[index] = TileType::Floor; }
                else { build_data.map.tiles[index] = TileType::Wall; }
            }
        }
        build_data.take_snapshot();

        // Now we iteratively apply cellular automata rules.
        for _i in 0..15 {
            let mut newtiles = build_data.map.tiles.clone();
            for y in 1..build_data.map.height - 1 {
                for x in 1..build_data.map.width - 1 {
                    let index = build_data.map.xy_index(x, y);
                    let mut neighbors = 0;
                    if build_data.map.tiles[index - 1] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[index + 1] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[index - build_data.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[index + build_data.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[index - (build_data.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[index - (build_data.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[index + (build_data.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[index + (build_data.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[index] = TileType::Wall;
                    } else {
                        newtiles[index] = TileType::Floor;
                    }
                }
            }

            build_data.map.tiles = newtiles.clone();
            build_data.take_snapshot();
        }
    }
}