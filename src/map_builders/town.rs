use super::BuilderChain;
use rltk::{RandomNumberGenerator, Tile};
use crate::map_builders::area_starting_points::AreaStartingPosition;
use crate::map_builders::distant_exit::DistantExit;
use crate::map_builders::{InitialMapBuilder, BuilderMap};
use crate::{TileType, Position};
use std::collections::HashSet;
use std::process::exit;

enum BuildingTag {
    Pub, Temple, Blacksmith, Clothier, Alchemist, PlayerHouse, Hovel, Abandoned, Unassigned
}

pub fn town_builder(new_depth: i32, rng: &mut RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height);
    chain.start_with(TownBuilder::new());
    chain
}

pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_rooms(rng, build_data);
    }
}

impl TownBuilder {
    pub fn new() -> Box<TownBuilder> {
        Box::new(TownBuilder{})
    }

    pub fn build_rooms(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.grass_layer(build_data);
        self.water_and_piers(rng, build_data);

        let (mut available_building_tiles, wall_gap_y) = self.town_walls(rng, build_data);
        let mut buildings = self.buildings(rng, build_data, &mut available_building_tiles);

        let doors = self.add_doors(rng, build_data, &mut buildings, wall_gap_y);
        self.add_paths(build_data, &doors);

        let exit_index = build_data.map.xy_index(build_data.width - 5, wall_gap_y);
        build_data.map.tiles[exit_index] = TileType::DownStairs;

        let building_size = self.sort_buildings(&buildings);
        self.building_factory(rng, build_data, &buildings, &building_size);

        // Start in the pub
        let the_pub = &buildings[building_size[0].0];
        build_data.starting_position = Some(Position {
            x: the_pub.0 + (the_pub.2 / 2),
            y: the_pub.1 + (the_pub.3 / 2)
        });

        // Make visible for screenshot
        for t in build_data.map.visible_tiles.iter_mut() {
            *t = true;
        }
        build_data.take_snapshot();
    }

    fn building_factory(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &[(i32, i32, i32, i32)],
        building_index: &[(usize, i32, BuildingTag)]
    ) {
        for(i, building) in buildings.iter().enumerate() {
            let build_type = &building_index[i].2;
            match build_type {
                BuildingTag::Pub => self.build_pub(&building, build_data, rng),
                BuildingTag::Temple => self.build_temple(&building, build_data, rng),
                BuildingTag::Blacksmith => self.build_smith(&building, build_data, rng),
                BuildingTag::Clothier => self.build_clothier(&building, build_data, rng),
                BuildingTag::Alchemist => self.build_alchemist(&building, build_data, rng),
                BuildingTag::PlayerHouse => self.build_my_house(&building, build_data, rng),
                BuildingTag::Hovel => self.build_hovel(&building, build_data, rng),
                BuildingTag::Abandoned => self.build_abandoned_house(&building, build_data, rng),
                _ => {}
            }
        }
    }

    fn sort_buildings(&mut self, buildings: &[(i32, i32, i32, i32)]) -> Vec<(usize, i32, BuildingTag)> {
        let mut building_size: Vec<(usize, i32, BuildingTag)> = Vec::new();
        for (i, building) in buildings.iter().enumerate() {
            building_size.push((
                i,
                building.2 * building.3,
                BuildingTag::Unassigned
            ));
        }
        building_size.sort_by(|a, b| b.1.cmp(&a.1));
        building_size[0].2 = BuildingTag::Pub;
        building_size[1].2 = BuildingTag::Temple;
        building_size[2].2 = BuildingTag::Blacksmith;
        building_size[3].2 = BuildingTag::Clothier;
        building_size[4].2 = BuildingTag::Alchemist;
        building_size[5].2 = BuildingTag::PlayerHouse;
        for b in building_size.iter_mut().skip(6) {
            b.2 = BuildingTag::Hovel;
        }
        let last_index = building_size.len() - 1;
        building_size[last_index].2 = BuildingTag::Abandoned;

        building_size
    }

    fn grass_layer(&mut self, build_data: &mut BuilderMap) {
        // We'll start with a nice layer of grass
        for t in build_data.map.tiles.iter_mut() {
            *t = TileType::Grass;
        }
        build_data.take_snapshot();
    }

    fn build_pub(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator
    ) {
        // Place the player
        build_data.starting_position = Some(Position{
            x: building.0 + (building.2 / 2),
            y: building.1 + (building.3 / 2)
        });
        let player_index = build_data.map.xy_index(
            building.0 + (building.2 / 2),
            building.1 + (building.3 /2)
        );

        // Place other items
        let mut to_place: Vec<&str> = vec!["Barkeep", "Shady Salesman", "Patron", "Patron", "Keg",
            "Table", "Chair", "Table", "Chair"
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, player_index);
    }

    fn build_temple(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator
    ) {
        // Place items
        let mut to_place: Vec<&str> = vec!["Priest", "Parishioner", "Parishioner", "Chair", "Chair", "Candle", "Candle"];
    }

    fn build_smith(&mut self,
                   building: &(i32, i32, i32, i32),
                   build_data : &mut BuilderMap,
                   rng: &mut rltk::RandomNumberGenerator)
    {
        // Place items
        let mut to_place : Vec<&str> = vec!["Blacksmith", "Anvil", "Water Trough", "Weapon Rack", "Armor Stand"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_clothier(&mut self,
                      building: &(i32, i32, i32, i32),
                      build_data : &mut BuilderMap,
                      rng: &mut rltk::RandomNumberGenerator)
    {
        // Place items
        let mut to_place : Vec<&str> = vec!["Clothier", "Cabinet", "Table", "Loom", "Hide Rack"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_alchemist(&mut self,
                       building: &(i32, i32, i32, i32),
                       build_data : &mut BuilderMap,
                       rng: &mut rltk::RandomNumberGenerator)
    {
        // Place items
        let mut to_place : Vec<&str> = vec!["Alchemist", "Chemistry Set", "Dead Thing", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_my_house(&mut self,
                      building: &(i32, i32, i32, i32),
                      build_data : &mut BuilderMap,
                      rng: &mut rltk::RandomNumberGenerator)
    {
        // Place items
        let mut to_place : Vec<&str> = vec!["Mom", "Bed", "Cabinet", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_hovel(&mut self,
                   building: &(i32, i32, i32, i32),
                   build_data : &mut BuilderMap,
                   rng: &mut rltk::RandomNumberGenerator)
    {
        // Place items
        let mut to_place : Vec<&str> = vec!["Peasant", "Bed", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_abandoned_house(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator
    ) {
        for y in building.1 .. building.1 + building.3 {
            for x in building.0 .. building.0 + building.2 {
                let index = build_data.map.xy_index(x, y);
                if build_data.map.tiles[index] == TileType::WoodFloor
                    && index != 0
                    && rng.roll_dice(1, 2) == 1 {
                    build_data.spawn_list.push((index, "Rat".to_string()));
                }
            }
        }
    }

    fn random_building_spawn(
        &mut self,
        building: &(i32, i32, i32, i32),
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
        to_place: &mut Vec<&str>,
        player_index: usize
    ) {
        for y in building.1..building.1 + building.3 {
            for x in building.0..building.0 + building.2 {
                let index = build_data.map.xy_index(x, y);
                if build_data.map.tiles[index] == TileType::WoodFloor
                    && index != player_index
                    && rng.roll_dice(1, 3) == 1
                    && !to_place.is_empty() {
                    let entity_tag = to_place[0];
                    to_place.remove(0);
                    build_data.spawn_list.push((index, entity_tag.to_string()));
                }
            }
        }
    }

    fn water_and_piers(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut n = (rng.roll_dice(1, 65535) as f32) / 65535f32;
        let mut water_width: Vec<i32> = Vec::new();

        for y in 0..build_data.height {
            let n_water = (f32::sin(n) * 10.0) as i32 + 14 + rng.roll_dice(1, 6);
            water_width.push(n_water);
            n += 0.1;
            for x in 0..n_water {
                let index = build_data.map.xy_index(x, y);
                build_data.map.tiles[index] = TileType::DeepWater;
            }
            for x in n_water..n_water + 3 {
                let index = build_data.map.xy_index(x, y);
                build_data.map.tiles[index] = TileType::ShallowWater;
            }
        }
        build_data.take_snapshot();

        // Add pier
        for _i in 0..rng.roll_dice(1, 4) + 6 {
            let y = rng.roll_dice(1, build_data.height) - 1;
            for x in 2 + rng.roll_dice(1, 6) .. water_width[y as usize] + 4 {
                let index = build_data.map.xy_index(x, y);
                build_data.map.tiles[index] = TileType::WoodFloor;
            }
        }
        build_data.take_snapshot();
    }

    fn town_walls(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) -> (HashSet<usize>, i32) {
        let mut available_building_tiles: HashSet<usize> = HashSet::new();
        let wall_gap_y = rng.roll_dice(1, build_data.height - 9) + 5;
        for y in 1..build_data.height - 2 {
            if !(y > wall_gap_y - 4 && y < wall_gap_y + 4) {
                let index = build_data.map.xy_index(30, y);
                build_data.map.tiles[index] = TileType::Wall;
                build_data.map.tiles[index - 1] = TileType::Floor;
                let index_right = build_data.map.xy_index(build_data.width - 2, y);
                build_data.map.tiles[index_right] = TileType::Wall;
                for x in 31 .. build_data.width - 2 {
                    let gravel_index = build_data.map.xy_index(x, y);
                    build_data.map.tiles[gravel_index] = TileType::Gravel;
                    if y > 2 && y < build_data.height - 1 {
                        available_building_tiles.insert(gravel_index);
                    }
                }
            } else {
                for x in 30 .. build_data.width {
                    let road_index = build_data.map.xy_index(x, y);
                    build_data.map.tiles[road_index] = TileType::Road;
                }
            }
        }

        build_data.take_snapshot();

        for x in 30 .. build_data.width - 1 {
            let index_top = build_data.map.xy_index(x, 1);
            build_data.map.tiles[index_top] = TileType::Wall;
            let index_bottom = build_data.map.xy_index(x, build_data.height - 2);
            build_data.map.tiles[index_bottom] = TileType::Wall;
        }
        build_data.take_snapshot();

        (available_building_tiles, wall_gap_y)
    }

    fn buildings(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        available_building_tiles: &mut HashSet<usize>
    ) -> Vec<(i32, i32, i32, i32)> {
        let mut buildings: Vec<(i32, i32, i32, i32)> = Vec::new();
        let mut n_buildings = 0;
        while n_buildings < 12 {
            let bx = rng.roll_dice(1, build_data.map.width - 32) + 30;
            let by = rng.roll_dice(1, build_data.map.height) - 2;
            let bw = rng.roll_dice(1, 8) + 4;
            let bh = rng.roll_dice(1, 8) + 4;
            let mut possible = true;
            for y in by.. by+bh {
                for x in bx.. bx+bw {
                    if x < 0 || x > build_data.width - 1 || y < 0 || y > build_data.height - 1 {
                        possible = false;
                    } else {
                        let index = build_data.map.xy_index(x, y);
                        if !available_building_tiles.contains(&index) {
                            possible = false;
                        }
                    }
                }
            }

            if possible {
                n_buildings += 1;
                buildings.push((bx, by, bw, bh));
                for y in by .. by+bh {
                    for x in bx .. bx+bw {
                        let index = build_data.map.xy_index(x, y);
                        build_data.map.tiles[index] = TileType::WoodFloor;
                        available_building_tiles.remove(&index);
                        available_building_tiles.remove(&(index + 1));
                        available_building_tiles.remove(&(index + build_data.width as usize));
                        available_building_tiles.remove(&(index - 1));
                        available_building_tiles.remove(&(index - build_data.width as usize));
                    }
                }
                build_data.take_snapshot();
            }
        }

        // Outline buildings
        let mut mapclone = build_data.map.clone();
        for y in 2..build_data.height - 2 {
            for x in 32..build_data.width - 2 {
                let index = build_data.map.xy_index(x, y);
                if build_data.map.tiles[index] == TileType::WoodFloor {
                    let mut neighbors = 0;
                    if build_data.map.tiles[index - 1] != TileType::WoodFloor { neighbors += 1; }
                    if build_data.map.tiles[index + 1] != TileType::WoodFloor { neighbors += 1; }
                    if build_data.map.tiles[index - build_data.width as usize] != TileType::WoodFloor { neighbors += 1; }
                    if build_data.map.tiles[index + build_data.width as usize] != TileType::WoodFloor { neighbors += 1; }
                    if neighbors > 0 {
                        mapclone.tiles[index] = TileType::Wall;
                    }
                }
            }
        }
        build_data.map = mapclone;
        build_data.take_snapshot();
        buildings
    }

    fn add_doors(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data : &mut BuilderMap,
        buildings: &mut Vec<(i32, i32, i32, i32)>,
        wall_gap_y : i32) -> Vec<usize>
    {
        let mut doors = Vec::new();
        for building in buildings.iter() {
            let door_x = building.0 + 1 + rng.roll_dice(1, building.2 - 3);
            let cy = building.1 + (building.3 / 2);
            let index = if cy > wall_gap_y {
                // Door on the north wall
                build_data.map.xy_index(door_x, building.1)
            } else {
                build_data.map.xy_index(door_x, building.1 + building.3 - 1)
            };
            build_data.map.tiles[index] = TileType::Floor;
            build_data.spawn_list.push((index, "Door".to_string()));
            doors.push(index);
        }
        build_data.take_snapshot();
        doors
    }

    fn add_paths(
        &mut self,
        build_data: &mut BuilderMap,
        doors: &[usize]
    ) {
        let mut roads = Vec::new();
        for y in 0..build_data.height {
            for x in 0..build_data.width {
                let index = build_data.map.xy_index(x, y);
                if build_data.map.tiles[index] == TileType::Road {
                    roads.push(index);
                }
            }
        }

        build_data.map.populate_blocked();
        for door_index in doors.iter() {
            let mut nearest_roads: Vec<(usize, f32)> = Vec::new();
            let door_ptr = rltk::Point::new(
                *door_index as i32 % build_data.map.width as i32,
                *door_index as i32 / build_data.map.width as i32
            );
            for r in roads.iter() {
                nearest_roads.push((
                    *r,
                    rltk::DistanceAlg::PythagorasSquared.distance2d(
                        door_ptr,
                        rltk::Point::new(
                            *r as i32 % build_data.map.width,
                            *r as i32 / build_data.map.width
                        )
                    )
                ));
            }
            nearest_roads.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            let destination = nearest_roads[0].0;
            let path = rltk::a_star_search(*door_index, destination, &mut build_data.map);
            if path.success {
                for step in path.steps.iter() {
                    let index = *step as usize;
                    build_data.map.tiles[index] = TileType::Road;
                    roads.push(index);
                }
            }
            build_data.take_snapshot();
        }
    }
}