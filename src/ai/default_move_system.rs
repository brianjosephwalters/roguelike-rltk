use std::convert::TryInto;
use rltk::RandomNumberGenerator;
use specs::{Entities, Entity, Join, System, WriteExpect, WriteStorage};
use crate::{EntityMoved, Map, Movement, MoveMode, MyTurn, Position, tile_walkable, Viewshed};

pub struct DefaultMoveAI {}

impl<'a> System<'a> for DefaultMoveAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, MoveMode>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, EntityMoved>,
        WriteExpect<'a, RandomNumberGenerator>,
        Entities<'a>
    );

    fn run(&mut self, data: Self::SystemData) {
        println!("Run DefaultMove System");

        let (
            mut turns,
            mut move_mode,
            mut positions,
            mut map,
            mut viewsheds,
            mut entity_moved,
            mut rng,
            entities
        ) = data;

        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, mut pos, mut mode, mut viewshed, _myturn)
            in (&entities, &mut positions, &mut move_mode, &mut viewsheds, &turns).join()
        {
            println!("x: {}, y: {}", pos.x, pos.y);
            turn_done.push(entity);
            match &mut mode.mode {
                Movement::Static => {
                    println!("  Static");
                },
                Movement::Random => {
                    println!("  Random");

                    let mut x = pos.x;
                    let mut y = pos.y;
                    let move_roll = rng.roll_dice(1, 5);
                    match move_roll {
                        1 => x -= 1,
                        2 => x += 1,
                        3 => y -= 1,
                        4 => y += 1,
                        _ => {}
                    }

                    if x > 0 && x < map.width - 1 && y > 0 && y < map.height - 1 {
                        let dest_index = map.xy_index(x, y);
                        if !map.blocked[dest_index] {
                            let index = map.xy_index(pos.x, pos.y);
                            map.blocked[index] = false;
                            pos.x = x;
                            pos.y = y;
                            entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                            map.blocked[dest_index] = true;
                            viewshed.dirty = true;
                        }
                    }
                },
                Movement::RandomWaypoint {path} => {
                    println!("  RandomWaypoint:", );
                    if let Some(path) = path {
                        println!("  Path Length: {}", path.len());
                        // We have a target, go there
                        let mut index = map.xy_index(pos.x, pos.y);
                        if path.len() > 1 {
                            if !map.blocked[path[1] as usize] {
                                map.blocked[index] = false;
                                pos.x = path[1] as i32 % map.width;
                                pos.y = path[1] as i32 / map.width;
                                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                                index = map.xy_index(pos.x, pos.y);
                                map.blocked[index] = true;
                                viewshed.dirty = true;
                                path.remove(0); // Remove first step of the path
                            }
                        } else {
                            mode.mode = Movement::RandomWaypoint { path: None };
                        }
                    } else {
                        println!("  No Path");

                        let target_x = rng.roll_dice(1, map.width - 2);
                        let target_y = rng.roll_dice(1, map.height - 2);
                        let index = map.xy_index(target_x, target_y);
                        if tile_walkable(map.tiles[index]) {
                            println!("    tile_walkable");
                            println!("    start: {}, end: {}",
                                     map.xy_index(pos.x, pos.y),
                                     map.xy_index(target_x, target_y));
                            let path = rltk::a_star_search(
                                map.xy_index(pos.x, pos.y),
                                map.xy_index(target_x, target_y),
                                &mut *map
                            );
                            println!("    tile_walkable: after a*");
                            if path.success && path.steps.len() > 1 {
                                println!("    tile_walkable: success");
                                mode.mode = Movement::RandomWaypoint {
                                    path: Some(path.steps)
                                };
                            }
                            println!("    tile_walkable: done");
                        }
                        println!("  Done with No Path");
                    }
                }
            }
        }
        println!("Done with Entities");

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
