use specs::prelude::*;
use crate::{EquipmentChanged, particle_system::ParticleBuilder, ProvidesFood, HungerClock, HungerState};

use super::{
    WantsToPickupItem, 
    Name, InBackpack, 
    Position, 
    gamelog::GameLog, 
    Pools, WantsToDropItem,
    Consumable, WantsToUseItem, ProvidesHealing,
    InflictsDamage, SufferDamage, Map,
    AreaOfEffect, Confusion, Equippable, Equipped,
    WantsToRemoveItem
};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity, 
            mut gamelog, 
            mut wants_pickup, 
            mut positions, 
            names, 
            mut backpack,
            mut dirty) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack {
                owner: pickup.collected_by,
            }).expect("Unable to insert backpack entry");
            dirty.insert(pickup.collected_by, EquipmentChanged {  }).expect("Unable to insert");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            }
        }
        wants_pickup.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity, 
            mut gamelog, 
            entities, 
            mut wants_drop, 
            names, 
            mut positions, 
            mut backpack,
            mut dirty) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position{ x: 0, y:0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }

            positions.insert(to_drop.item, Position { x: dropper_pos.x, y: dropper_pos.y }).expect("Unabled to insert position.");
            backpack.remove(to_drop.item);
            dirty.insert(entity, EquipmentChanged {}).expect("Unable to insert");
            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }

        wants_drop.clear();
    }
}

pub struct ItemRemoveSystem {}
impl<'a> System<'a> for ItemRemoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( 
                        Entities<'a>,
                        WriteStorage<'a, WantsToRemoveItem>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, InBackpack>
                      );
    fn run(&mut self, data : Self::SystemData) {
        let (entities, 
             mut wants_remove, 
             mut equipped, 
             mut backpack
        ) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack.insert(to_remove.item, InBackpack{ owner: entity }).expect("Unable to insert backpack");
        }

        wants_remove.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Entity>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SufferDamage>,
        WriteStorage<'a, Pools>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, ProvidesFood>,
        WriteStorage<'a, HungerClock>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            player_entity,
            entities, 
            mut gamelog,
            mut wants_use, 
            consumables,
            healing,
            inflict_damage,
            mut suffer_damage,
            mut pools,
            names,
            aoe,
            mut confused,
            equippable, 
            mut equipped, 
            mut backpack,
            mut particle_builder,
            positions,
            mut dirty,
            provides_food,
            mut hunger_clocks
        ) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty.insert(entity, EquipmentChanged {  });
            let mut used_item = true;
            let mut targets : Vec<Entity> = Vec::new();
            match useitem.target {
                None => { targets.push( *player_entity )},
                Some(target) => {
                    let area_of_effect = aoe.get(useitem.item);
                    match area_of_effect {
                        None => {
                            let index = map.xy_index(target.x, target.y);
                            crate::spatial::for_each_tile_content(index, |mob| targets.push(mob));
                        }
                        Some(area_of_effect) => {
                            let mut blast_tiles = rltk::field_of_view(target, area_of_effect.radius, &*map);
                            blast_tiles.retain( |p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
                            for tile_index in blast_tiles.iter() {
                                let index = map.xy_index(tile_index.x, tile_index.y);
                                crate::spatial::for_each_tile_content(index, |mob| targets.push(mob));
                                particle_builder.request(tile_index.x, tile_index.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('???'), 200.0);
                            }
                        }
                    }
                }
            }

            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            let item_equippable = equippable.get(useitem.item);
            match item_equippable {
                None => {}
                Some (can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    // Remove any items the target has in the item's slot
                    let mut to_unequip: Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                        if already_equipped.owner == target && already_equipped.slot == target_slot {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.push(format!("You unequiped {}.", name.name));
                            }
                        }
                    }
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack.insert(*item, InBackpack { owner: target }).expect("Unable to insert backpack entry.");
                    }

                    // Wield the item
                    equipped.insert(useitem.item, Equipped { owner: target, slot: target_slot }).expect("Unable to insert equipped component");
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!("You equip {}.", names.get(useitem.item).unwrap().name));
                    }
                }
            }
            
            let item_edible = provides_food.get(useitem.item);
            match item_edible {
                None => {}
                Some(_) => {
                    used_item = true;
                    let target = targets[0];
                    let hc = hunger_clocks.get_mut(target);
                    if let Some(hc) = hc {
                        hc.state = HungerState::WellFed;
                        hc.duration = 20;
                        gamelog.entries.push(format!("You eat the {}.", names.get(useitem.item).unwrap().name));
                    }
                }
            }


            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {},
                Some(healer) => {
                    used_item = false;
                    for target in targets.iter() {
                        let pools = pools.get_mut(*target);
                        if let Some(pools) = pools {
                            pools.hit_points.current = i32::min(pools.hit_points.max, pools.hit_points.current + healer.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.push(format!("You drink the {}, healing {} hp.", names.get(useitem.item).unwrap().name, healer.heal_amount));
                            }
                        }
                        used_item = true;

                        let pos = positions.get(*target);
                        if let Some(pos) = pos {
                            particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::GREEN), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('???'), 200.0);
                        }
                    }  
                }
            }

            let item_damages = inflict_damage.get(useitem.item);
            match item_damages {
                None => {},
                Some(damage) => {
                    used_item = false;
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage, false);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!("You use {} on {}, inflicting {} hp.", item_name.name, mob_name.name, damage.damage));
                            
                            let pos = positions.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::RED), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('???'), 200.0);
                            }
                        }
                        used_item = true;
                    }
                }
            }

            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confused.get(useitem.item);
                match causes_confusion {
                    None => {},
                    Some(confusion) => {
                        used_item = false;
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(useitem.item).unwrap();
                                gamelog.entries.push(format!("You use {} on {}, confusing them.", item_name.name, mob_name.name));
                            
                                let pos = positions.get(*mob);
                                if let Some(pos) = pos {
                                    particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::MAGENTA), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('?'), 200.0);
                                }
                            }
                            used_item = true;
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused.insert(mob.0, Confusion {turns: mob.1}).expect("Unable to insert status");
            }

            if used_item {
                let consumable = consumables.get(useitem.item);
                match consumable {
                    None => {},
                    Some(_) => {
                        entities.delete(useitem.item).expect("Delete failed");
                    }
                }
            }
        }

        wants_use.clear();
    }
}
