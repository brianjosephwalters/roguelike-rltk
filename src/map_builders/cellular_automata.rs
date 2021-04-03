use std::collections::HashMap;
use super::{ MapBuilder, Map, Position,
    common::{ remove_unreachable_areas_returning_most_distant, generate_voronoi_spawn_regions }
};
use crate::{ SHOW_MAPGEN_VISUALIZER, TileType, spawner };
use specs::World;
use rltk::RandomNumberGenerator;

const MIN_ROOM_SIZE: i32 = 8;

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    spawn_list: Vec<(usize, String)>,
}

impl MapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self) {
        self.build();
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true
            }
            self.history.push(snapshot);
        }
    }

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }
}

impl CellularAutomataBuilder {
    pub fn new(new_depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            spawn_list: Vec::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // First, we completely randomize the map, setting 55% to be floor.
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let index = self.map.xy_index(x, y);
                if roll > 55 { self.map.tiles[index] = TileType::Floor; }
                else { self.map.tiles[index] = TileType::Wall; }
            }
        }
        self.take_snapshot();

        // Now we iteratively apply cellular automata rules.
        for _i in 0..15 {
            let mut newtiles = self.map.tiles.clone();
            for y in 1..self.map.height - 1 {
                for x in 1..self.map.width - 1 {
                    let index = self.map.xy_index(x, y);
                    let mut neighbors = 0;
                    if self.map.tiles[index - 1] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[index + 1] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[index - self.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[index + self.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[index - (self.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[index - (self.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[index + (self.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[index + (self.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[index] = TileType::Wall;
                    } else {
                        newtiles[index] = TileType::Floor;
                    }
                }
            }

            self.map.tiles = newtiles.clone();
            self.take_snapshot();
        }

        // Find a starting point; start at the middle and walk left until we find an tile.
        self.starting_position = Position { x: self.map.width / 2, y: self.map.height / 2 };
        let mut start_index = self.map.xy_index(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_index] != TileType::Floor {
            self.starting_position.x -= 1;
            start_index = self.map.xy_index(self.starting_position.x, self.starting_position.y);
        }

        // Find all of the tiles we can reach from the starting point
        let map_starts: Vec<usize> = vec![start_index];
        let dijkstra_map = rltk::DijkstraMap::new(
            self.map.width, 
            self.map.height, 
            &map_starts, 
            &self.map, 
            200.0
        );
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_index);
        self.take_snapshot();

        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Now build a noise map for use in spawning entities
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);

        // Spawn the entities
        for area in self.noise_areas.iter() {
            spawner::spawn_region(&self.map, &mut rng, area.1, self.depth, &mut self.spawn_list);
        }
    }
}