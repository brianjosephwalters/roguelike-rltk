use specs::prelude::*;
use super::{Viewshed, Position, Map, Player};

use rltk::{field_of_view, Point, RandomNumberGenerator};
use crate::gamelog::GameLog;
use crate::{BlocksVisibility, Name, Hidden};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, Player>,
                        WriteStorage<'a, Hidden>,
                        WriteExpect<'a, RandomNumberGenerator>,
                        WriteExpect<'a, GameLog>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, BlocksVisibility>
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map,
            entities,
            mut viewshed,
            pos,
            player,
            mut hidden,
            mut rng,
            mut log,
            names,
            blocks_visibility
        ) = data;

        map.view_blocked.clear();
        for (block_pos, _block) in (&pos, &blocks_visibility).join() {
            let index = map.xy_index(block_pos.x, block_pos.y);
            map.view_blocked.insert(index);
        }
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // If this is the player, reveal what they can see.
                let _p: Option<&Player> = player.get(ent);
                if let Some(_p) = _p {
                    for t in map.visible_tiles.iter_mut() { *t = false };
                    for vis in viewshed.visible_tiles.iter() {
                        if vis.x > 0 && vis.x < map.width - 1 && vis.y > 0 && vis.y < map.height - 1 {
                            let index = map.xy_index(vis.x, vis.y);
                            map.revealed_tiles[index] = true;
                            map.visible_tiles[index] = true;

                            // Chance to reveal Hidden things
                            crate::spatial::for_each_tile_content(index, |e| {
                                let maybe_hidden = hidden.get(e);
                                if let Some(_maybe_hidden) = maybe_hidden {
                                    if rng.roll_dice(1, 24) == 1 {
                                        let name = names.get(e);
                                        if let Some(name) = name {
                                            log.entries.push(format!("You spotted a {}!", &name.name));
                                        }
                                        hidden.remove(e);
                                    }
                                }
                            });
                        }
                    }
                }    
            }
        }
    }
}
