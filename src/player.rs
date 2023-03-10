use rltk::{VirtualKeyCode, Point, Rltk};
use specs::prelude::*;
use crate::{TileType, Door, BlocksTile, BlocksVisibility, Renderable, Faction, Attributes, Vendor, VendorMode, HungerClock, HungerState};

use super::{Pools, Position, Player, RunState, State, Map, Viewshed, WantsToMelee, Item, GameLog, WantsToPickupItem, EntityMoved};
use std::cmp::{min, max};
use crate::raws::faction_structs::Reaction;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    println!("try_move_player");
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<Attributes>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let factions = ecs.read_storage::<Faction>();
    let vendors = ecs.read_storage::<Vendor>();

    let mut result = RunState::AwaitingInput;

    let mut swap_entities : Vec<(Entity, i32, i32)> = Vec::new();
    for (entity, _player, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        if pos.x + delta_x < 1 ||
            pos.x + delta_x > map.width - 1 ||
            pos.y + delta_y < 1 ||
            pos.y + delta_y > map.height - 1 {
            return RunState::AwaitingInput;
        }
        let destination_index = map.xy_index(pos.x + delta_x, pos.y + delta_y);

        result = crate::spatial::for_each_tile_content_with_gamemode(destination_index, |potential_target| {
            if let Some(_vendor) = vendors.get(potential_target) {
                return Some(RunState::ShowVendor { vendor: potential_target, mode: VendorMode::Sell });
            }
            
            let mut hostile = true;
            if combat_stats.get(potential_target).is_some() {
                if let Some(faction) = factions.get(potential_target) {
                    let reaction = crate::raws::faction_reaction(
                        &faction.name,
                        "Player",
                        &crate::raws::RAWS.lock().unwrap()
                    );
                    if reaction != Reaction::Attack { hostile = false };
                }
            }
            if !hostile {
                // Note that we want to move the bystander
                swap_entities.push((potential_target, pos.x, pos.y));

                // Move the player
                pos.x = min(map.width - 1, max(0, pos.x + delta_x));
                pos.y = min(map.height - 1, max(0, pos.y + delta_y));
                entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker.");

                viewshed.dirty = true;
                let mut ppos = ecs.write_resource::<Point>();
                ppos.x = pos.x;
                ppos.y = pos.y;
            } else {
                let target = combat_stats.get(potential_target);
                if let Some(_target) = target {
                    wants_to_melee.insert(entity, WantsToMelee { target: potential_target }).expect("Add target failed.");
                    return Some(RunState::Ticking);
                }
            }

            let door = doors.get_mut(potential_target);
            if let Some(door) = door {
                door.open = true;
                blocks_visibility.remove(potential_target);
                blocks_movement.remove(potential_target);
                let glyph = renderables.get_mut(potential_target).unwrap();
                glyph.glyph = rltk::to_cp437('/');
                viewshed.dirty = true;
                return Some(RunState::Ticking);
            }

            None
        });

        if !crate::spatial::is_blocked(destination_index) {
            pos.x = min(map.width - 1, max(0, pos.x + delta_x));
            pos.y = min(map.height - 1, max(0, pos.y + delta_y));
            entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");

            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
            result = RunState::Ticking;
            match map.tiles[destination_index] {
                TileType::DownStairs => result = RunState::NextLevel,
                TileType::UpStairs => result = RunState::PreviousLevel,
                _ => {}
            }
        }
    }

    for m in swap_entities.iter() {
        let their_pos = positions.get_mut(m.0);
        if let Some(their_pos) = their_pos {
            let old_index = map.xy_index(their_pos.x, their_pos.y);
            their_pos.x = m.1;
            their_pos.y = m.2;
            let new_index = map.xy_index(their_pos.x, their_pos.y);
            crate::spatial::move_entity(m.0, old_index, new_index);
            result = RunState::Ticking;
        }
    }

    result
}

pub fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing here to pick up".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem{ collected_by: *player_entity, item }).expect("Unable to insert want to pickup");        }
    }

}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_index = map.xy_index(player_pos.x, player_pos.y);
    if map.tiles[player_index] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way down from here.".to_string());
        false
    }
}

pub fn try_previous_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_index = map.xy_index(player_pos.x, player_pos.y);
    if map.tiles[player_index] == TileType::UpStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way down from here.".to_string());
        false
    }
}

pub fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let factions = ecs.read_storage::<Faction>();

    let worldmap_resource = ecs.fetch::<Map>();
    
    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let index = worldmap_resource.xy_index(tile.x, tile.y);
        crate::spatial::for_each_tile_content(index, |entity_id| {
            let faction = factions.get(entity_id);
            match faction {
                None => {}
                Some(faction) => {
                    let reaction = crate::raws::faction_reaction(
                        &faction.name,
                        "Player",
                        &crate::raws::RAWS.lock().unwrap()
                    );
                    if reaction == Reaction::Attack {
                        can_heal = false;
                    }
                }
            }
        });
    }

    let hunger_clocks = ecs.read_storage::<HungerClock>();
    let hc = hunger_clocks.get(*player_entity);
    if let Some(hc) = hc {
        match hc.state {
            HungerState::Hungry => can_heal = false,
            HungerState::Starving => can_heal = false,
            _ => {}
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<Pools>();
        let pools = health_components.get_mut(*player_entity).unwrap();
        pools.hit_points.current = i32::min(pools.hit_points.current  + 1, pools.hit_points.max);
    }

    RunState::Ticking
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {

    // Hotkeys
    if ctx.shift & &ctx.key.is_some() {
        let key : Option<i32> = match ctx.key.unwrap() {
            VirtualKeyCode::Key1 => Some(1),
            VirtualKeyCode::Key2 => Some(2),
            VirtualKeyCode::Key3 => Some(3),
            VirtualKeyCode::Key4 => Some(4),
            VirtualKeyCode::Key5 => Some(5),
            VirtualKeyCode::Key6 => Some(6),
            VirtualKeyCode::Key7 => Some(7),
            VirtualKeyCode::Key8 => Some(8),
            VirtualKeyCode::Key9 => Some(9),
            _ => None
        };
        if let Some(key) = key {
            return use_consumable_hotkey(gs, key-1);
        }
    }
    match ctx.key {
        None => { return RunState::AwaitingInput }
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => return try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => return try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up | 
            VirtualKeyCode::Numpad8 | 
            VirtualKeyCode::K => return try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down | 
            VirtualKeyCode::Numpad2 | 
            VirtualKeyCode::J => return try_move_player(0, 1, &mut gs.ecs),

            // Diagonals
            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::Y => return try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::U => return try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::N => return try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::B => return try_move_player(-1, 1, &mut gs.ecs),

            VirtualKeyCode::G => get_item(&mut gs.ecs),

            VirtualKeyCode::I => return RunState::ShowInventory,

            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }
            VirtualKeyCode::Comma => {
                if try_previous_level(&mut gs.ecs) {
                    return RunState::PreviousLevel;
                }
            }
            VirtualKeyCode::R => return RunState::ShowRemoveItem,
            
            // Skip Turn
            VirtualKeyCode::Numpad5 => return skip_turn(&mut gs.ecs),
            VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),
            
            // Save and Quit
            VirtualKeyCode::Escape => return RunState::SaveGame,

            // Cheating!
            VirtualKeyCode::Backslash => return RunState::ShowCheatMenu,
            
            _ => { return RunState::AwaitingInput }
        }
    }
    RunState::Ticking
}

fn use_consumable_hotkey(gs: &mut State, key: i32) -> RunState {
    use super::{Consumable, InBackpack, WantsToUseItem};

    let consumables = gs.ecs.read_storage::<Consumable>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let player_entity = gs.ecs.fetch::<Entity>();
    let entities = gs.ecs.entities();
    let mut carried_consumables = Vec::new();
    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity {
            carried_consumables.push(entity);
        }
    }

    if (key as usize) < carried_consumables.len() {
        use crate::components::Ranged;
        if let Some(ranged) = gs.ecs.read_storage::<Ranged>().get(carried_consumables[key as usize]) {
            return RunState::ShowTargeting {range: ranged.range, item: carried_consumables[key as usize]};
        }

        let mut intent = gs.ecs.write_storage::<WantsToUseItem>();
        intent.insert(
            *player_entity,
            WantsToUseItem { item: carried_consumables[key as usize], target: None }
        ).expect("Unable to insert intent)");
        return RunState::Ticking;
    }
    RunState::Ticking
}
