mod prefab_levels;
mod prefab_sections;

use crate::{Map, Position, SHOW_MAPGEN_VISUALIZER, TileType, spawner};
use crate::map_builders::MapBuilder;
use specs::World;
use crate::map_builders::common::remove_unreachable_areas_returning_most_distant;
use rltk::RandomNumberGenerator;

#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
    Constant { level: prefab_levels::PrefabLevel },
    Sectional { section: prefab_sections::PrefabSection },
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawns: Vec<(usize, String)>,
    previous_builder: Option<Box<dyn MapBuilder>>,
    spawn_list: Vec<(usize, String)>,
}

impl MapBuilder for PrefabBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self)  {
        self.build();
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }
}

impl PrefabBuilder {
    #[allow(dead_code)]
    pub fn new(new_depth : i32, previous_builder: Option<Box<dyn MapBuilder>>) -> PrefabBuilder {
        PrefabBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y : 0 },
            depth : new_depth,
            history : Vec::new(),
            mode : PrefabMode::Sectional{section: prefab_sections::UNDERGROUND_FORT },
            spawns: Vec::new(),
            spawn_list: Vec::new(),
            previous_builder,
        }
    }

    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();
        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let index = self.map.xy_index(x as i32, y as i32);
                        // We're doing some nasty casting to make it easier to type things like '#' in the match
                        self.char_to_map(cell.ch as u8 as char, index);
                    }
                }
            }
        }
    }

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        // Start by converting to a vec, with newlines removed
        let mut string_vec: Vec<char> = level.template.chars().filter(|a| *a != '\r' && *a != '\n').collect();
        for c in string_vec.iter_mut() { if *c as u8 == 160u8 { *c = ' ';} }
        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let index = self.map.xy_index(tx as i32, ty as i32);
                    self.char_to_map(string_vec[i], index);
                }
                i += 1;
            }
        }
    }

    fn read_ascii_to_vec(template: &str) -> Vec<char> {
        let mut string_vec: Vec<char> = template.chars().filter(|a| *a != '\r' && *a != '\n').collect();
        for c in string_vec.iter_mut() { if *c as u8 == 160u8 { *c = ' '; } }
        string_vec
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        match self.mode {
            PrefabMode::RexLevel {template} => self.load_rex_map(&template),
            PrefabMode::Constant{level} => self.load_ascii_map(&level),
            PrefabMode::Sectional{section} => self.apply_sectional(&section),
        }
        self.take_snapshot();

        // Find a starting point; start at the middle and walk left until we find an open tile
        let mut start_index;
        if self.starting_position.x == 0 {
            self.starting_position = Position { x: self.map.width / 2, y: self.map.height / 2 };
            start_index = self.map.xy_index(self.starting_position.x, self.starting_position.y);
            while self.map.tiles[start_index] != TileType::Floor {
                self.starting_position.x -= 1;
                start_index = self.map.xy_index(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();
        }
        let mut has_exit = false;
        for t in self.map.tiles.iter() {
            if *t == TileType::DownStairs { has_exit = true; }
        }

        if !has_exit {
            start_index = self.map.xy_index(self.starting_position.x, self.starting_position.y);

            // Find all tiles we can reach from the starting point
            let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_index);
            self.take_snapshot();

            // Place the stairs
            self.map.tiles[exit_tile] = TileType::DownStairs;
            self.take_snapshot();
        }

    }

    pub fn apply_sectional(&mut self, section: &prefab_sections::PrefabSection) {
        use prefab_sections::*;

        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);

        // Place the new section
        let chunk_x;
        match section.placement.0 {
            HorizontalPlacement::Left => chunk_x = 0,
            HorizontalPlacement::Center => chunk_x = (self.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => chunk_x = (self.map.width - 1) - section.width as i32
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (self.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => chunk_y = (self.map.height - 1) - section.height as i32
        }

        // Build the map
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map();

        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map().clone();
        for e in prev_builder.get_spawn_list().iter() {
            let index = e.0;
            let x = index as i32 % self.map.width;
            let y = index as i32 / self.map.width;
            if x < chunk_x || x > (chunk_x + section.width as i32) ||
                y < chunk_y || y > (chunk_y + section.height as i32) {
                self.spawn_list.push((index, e.1.to_string()))
            }
        }
        self.take_snapshot();

        println!("string_vec len(): {}", string_vec.len());
        println!("section width: {}, section height: {}", section.width, section.height);
        println!("chunk_x: {}, chunk_y: {}", chunk_x, chunk_y);
        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx > 0 && tx < self.map.width as usize - 1 &&
                    ty > 0 && ty < self.map.height as usize - 1 {
                    let index = self.map.xy_index(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map(string_vec[i], index);
                }
                i += 1;
            }
        }
        self.take_snapshot();
    }

    fn char_to_map(&mut self, ch: char, index: usize) {
        match ch {
            ' ' => self.map.tiles[index] = TileType::Floor,
            '#' => self.map.tiles[index] = TileType::Wall,
            '@' => {
                let x = index as i32 % self.map.width;
                let y = index as i32 / self.map.width;
                self.map.tiles[index] = TileType::Floor;
                self.starting_position = Position{ x: x as i32, y: y as i32 };
            }
            '>' => self.map.tiles[index] = TileType::DownStairs,
            'g' => {
                self.map.tiles[index] = TileType::Floor;
                self.spawns.push((index, "Goblin".to_string()));
            }
            'o' => {
                self.map.tiles[index] = TileType::Floor;
                self.spawns.push((index, "Orc".to_string()));
            }
            '^' => {
                self.map.tiles[index] = TileType::Floor;
                self.spawns.push((index, "Bear Trap".to_string()));
            }
            '%' => {
                self.map.tiles[index] = TileType::Floor;
                self.spawns.push((index, "Rations".to_string()));
            }
            '!' => {
                self.map.tiles[index] = TileType::Floor;
                self.spawns.push((index, "Health Potion".to_string()));
            }
            _ => {
                rltk::console::log(format!("Unknown glyph loading map: {}", (ch as u8) as char));
            }
        }
    }
}