
use rltk::{RGB, Rltk, Algorithm2D, BaseMap, Point, SmallVec};
use specs::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use specs::Entity;

mod tiletype;
pub use tiletype::{TileType, tile_walkable, tile_opaque};
use crate::map::tiletype::tile_cost;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub view_blocked: HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {

    /// Generates an empty map, consisting entirely of solid walls
    pub fn new(new_depth : i32, width: i32, height: i32) -> Map {
        let map_tile_count = (width * height) as usize;
        Map {
            tiles : vec![TileType::Wall; map_tile_count],
            width,
            height,
            revealed_tiles : vec![false; map_tile_count],
            visible_tiles : vec![false; map_tile_count],
            blocked : vec![false; map_tile_count],
            tile_content : vec![Vec::new(); map_tile_count],
            depth: new_depth,
            view_blocked: HashSet::new(),
        }
    }

    pub fn xy_index(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 { return false; }
        let index = self.xy_index(x, y);
        !self.blocked[index]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = !tile_walkable(*tile)
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
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
}

// pub fn draw_map(map: &Map, ctx: &mut Rltk) {
//     let mut y = 0;
//     let mut x = 0;
//     for (index, tile) in map.tiles.iter().enumerate() {
//         if map.revealed_tiles[index] {
//             let glyph;
//             let mut fg;
//             match tile {
//                 TileType::Floor => {
//                     glyph = rltk::to_cp437('.');
//                     fg = RGB::from_f32(0.5, 0.5, 0.5);
//                 }
//                 TileType::Wall => {
//                     glyph = wall_glyph(&*map, x, y);
//                     fg = RGB::from_f32(0.0, 1.0, 0.0);
//                 }
//                 TileType::DownStairs => {
//                     glyph = rltk::to_cp437('>');
//                     fg = RGB::from_f32(0., 1.0, 1.0);
//                 }
//             }
//             if !map.visible_tiles[index] { fg = fg.to_greyscale() }
//             ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
//         }
//
//         x += 1;
//         if x > (map.width * map.height) as i32 {
//             x = 0;
//             y += 1;
//         }
//     }
// }

// fn wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
//     if x < 1 || x > map.width-2 || y < 1 || y > map.height-2 as i32 { return 35; }
//     let mut mask : u8 = 0;
//
//     if is_revealed_and_wall(map, x, y - 1) { mask +=1; }
//     if is_revealed_and_wall(map, x, y + 1) { mask +=2; }
//     if is_revealed_and_wall(map, x - 1, y) { mask +=4; }
//     if is_revealed_and_wall(map, x + 1, y) { mask +=8; }
//
//     match mask {
//         0 => { 9 } // Pillar because we can't see neighbors
//         1 => { 186 } // Wall only to the north
//         2 => { 186 } // Wall only to the south
//         3 => { 186 } // Wall to the north and south
//         4 => { 205 } // Wall only to the west
//         5 => { 188 } // Wall to the north and west
//         6 => { 187 } // Wall to the south and west
//         7 => { 185 } // Wall to the north, south and west
//         8 => { 205 } // Wall only to the east
//         9 => { 200 } // Wall to the north and east
//         10 => { 201 } // Wall to the south and east
//         11 => { 204 } // Wall to the north, south and east
//         12 => { 205 } // Wall to the east and west
//         13 => { 202 } // Wall to the east, west, and south
//         14 => { 203 } // Wall to the east, west, and north
//         15 => { 206 }  // ╬ Wall on all sides
//         _ => { 35 } // We missed one?
//     }
// }

// fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
//     let idx = map.xy_index(x, y);
//     map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
// }

