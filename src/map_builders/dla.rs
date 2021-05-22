use super::{ Position, common::paint };
use crate::TileType;
use rltk::RandomNumberGenerator;
use crate::map_builders::common::Symmetry;
use crate::map_builders::{InitialMapBuilder, BuilderMap, MetaMapBuilder};

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgorithm { WalkInwards, WalkOutwards, CentralAttractor }

pub struct DLABuilder {
    algorithm: DLAAlgorithm,
    brush_size: i32,
    symmetry: Symmetry,
    floor_percent: f32,
}

impl InitialMapBuilder for DLABuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl MetaMapBuilder for DLABuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DLABuilder {

    pub fn new() -> DLABuilder {
        println!("DLABuilder");
        DLABuilder {
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn walk_inwards() -> Box<DLABuilder> {
        println!("DLABuilder::walk_inwards()");
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    pub fn walk_outwards() -> Box<DLABuilder> {
        println!("DLABuilder::walk_outwards()");
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::WalkOutwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    pub fn central_attractor() -> Box<DLABuilder> {
        println!("DLABuilder::central_attractor()");
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    pub fn insectoid() -> Box<DLABuilder> {
        println!("DLABuilder::insectoid()");
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::Horizontal,
            floor_percent: 0.25,
        })
    }

    pub fn heavy_erosion() -> Box<DLABuilder> {
        println!("DLABuilder::heavy_erosion()");
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.35,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Carve a starting seed
        let starting_position = Position { x: build_data.map.width / 2, y: build_data.map.height / 2 };
        let start_index = build_data.map.xy_index(starting_position.x, starting_position.y);
        build_data.take_snapshot();
        build_data.map.tiles[start_index] = TileType::Floor;
        build_data.map.tiles[start_index - 1] = TileType::Floor;
        build_data.map.tiles[start_index + 1] = TileType::Floor;
        build_data.map.tiles[start_index - build_data.map.width as usize] = TileType::Floor;
        build_data.map.tiles[start_index + build_data.map.width as usize] = TileType::Floor;

        // Random Walker
        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data.map.tiles.iter()
            .filter(|a| **a == TileType::Floor)
            .count();
        while floor_tile_count < desired_floor_tiles {
            match self.algorithm {
                DLAAlgorithm::WalkInwards => {
                    let mut digger_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    let mut previous_x = digger_x;
                    let mut previous_y = digger_y;
                    let mut digger_index = build_data.map.xy_index(digger_x, digger_y);
                    while build_data.map.tiles[digger_index] == TileType::Wall {
                        previous_x = digger_x;
                        previous_y = digger_y;
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 {digger_x -= 1;} }
                            2 => { if digger_x < build_data.map.width - 2 {digger_x += 1;} }
                            3 => { if digger_y > 2 {digger_y -= 1;} }
                            _ => { if digger_y < build_data.map.height - 2 {digger_y += 1;} }
                        }
                        digger_index = build_data.map.xy_index(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, previous_x, previous_y);
                }
                // Seems wrong
                DLAAlgorithm::WalkOutwards => {
                    let mut digger_x = starting_position.x;
                    let mut digger_y = starting_position.y;
                    let mut digger_index = build_data.map.xy_index(digger_x, digger_y);
                    while build_data.map.tiles[digger_index] == TileType::Floor {
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1;} }
                            2 => { if digger_x < build_data.map.width - 2 { digger_x += 1;} }
                            3 => { if digger_y > 2 { digger_y -= 1;} }
                            _ => { if digger_y < build_data.map.height - 2 { digger_y += 1;} }
                        }
                        digger_index = build_data.map.xy_index(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, digger_x, digger_y);
                }
                DLAAlgorithm::CentralAttractor => {
                    let mut digger_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    let mut previous_x = digger_x;
                    let mut previous_y = digger_y;
                    let mut digger_index = build_data.map.xy_index(digger_x, digger_y);

                    let mut path = rltk::line2d(
                        rltk::LineAlg::Bresenham,
                        rltk::Point::new(digger_x, digger_y),
                        rltk::Point::new(starting_position.x, starting_position.y)
                    );

                    while build_data.map.tiles[digger_index] == TileType::Wall && !path.is_empty() {
                        previous_x = digger_x;
                        previous_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                        digger_index = build_data.map.xy_index(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, previous_x, previous_y);
                }
            }
            build_data.take_snapshot();
            floor_tile_count = build_data.map.tiles.iter().filter(
                |a| **a == TileType::Floor
            ).count();
        }
    }

}