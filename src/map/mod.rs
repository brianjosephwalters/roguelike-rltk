
use rltk::{Algorithm2D, BaseMap, Point, RGB, SmallVec};
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use specs::Entity;

mod tiletype;
mod themes;
pub mod dungeon;
pub use dungeon::{MasterDungeonMap, level_transition, freeze_level_entities, thaw_level_entities};

pub use tiletype::{TileType, tile_walkable, tile_opaque};
pub use themes::*;
use crate::map::tiletype::tile_cost;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,
    pub view_blocked: HashSet<usize>,
    pub name : String,
    pub outdoors: bool,
    pub light: Vec<RGB>,
}

impl Map {

    /// Generates an empty map, consisting entirely of solid walls
    pub fn new<S: ToString>(new_depth : i32, width: i32, height: i32, name: S) -> Map {
        let map_tile_count = (width * height) as usize;
        crate::spatial::set_size(map_tile_count);
        Map {
            tiles : vec![TileType::Wall; map_tile_count],
            width,
            height,
            revealed_tiles : vec![false; map_tile_count],
            visible_tiles : vec![false; map_tile_count],
            depth: new_depth,
            bloodstains: HashSet::new(),
            view_blocked: HashSet::new(),
            name: name.to_string(),
            outdoors: true,
            light: vec![RGB::from_f32(0.0, 0.0, 0.0); map_tile_count]
        }
    }

    pub fn xy_index(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 { return false; }
        let index = self.xy_index(x, y);
        !crate::spatial::is_blocked(index)
    }

    pub fn populate_blocked(&mut self) {
        crate::spatial::populate_blocked_from_map(self);
    }

    pub fn clear_content_index(&mut self) {
        crate::spatial::clear();
    }

}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, index: usize) -> bool {
        if index > 0 && index < self.tiles.len() {
            tile_opaque(self.tiles[index]) || self.view_blocked.contains(&index)
        } else {
            true
        }
    }

    fn get_available_exits(&self, index: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();

        let x = index as i32 % self.width;
        let y = index as i32 / self.width;
        let w = self.width as usize;

        let tt = self.tiles[index as usize];

        //Cardinal Directions
        if self.is_exit_valid(x-1, y) { exits.push((index-1, tile_cost(tt))) };
        if self.is_exit_valid(x+1, y) { exits.push((index+1, tile_cost(tt))) };
        if self.is_exit_valid(x, y-1) { exits.push((index-w, tile_cost(tt))) };
        if self.is_exit_valid(x, y+1) { exits.push((index+w, tile_cost(tt))) };

        // Diagonals
        if self.is_exit_valid(x-1, y-1) { exits.push(((index-w)-1, tile_cost(tt) * 1.45)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((index-w)+1, tile_cost(tt) * 1.45)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((index+w)-1, tile_cost(tt) * 1.45)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((index+w)+1, tile_cost(tt) * 1.45)); }

        exits
    }

    fn get_pathing_distance(&self, index1: usize, index2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(index1 % w, index1 / w);
        let p2 = Point::new(index2 % w, index2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}


