use specs::{Entities, Entity, Join, ReadStorage, System, WriteStorage};
use crate::{Faction, Map, MyTurn, Position, ReadExpect, WantsToMelee};
use crate::raws::faction_structs::Reaction;

pub struct AdjacentAI {}

impl<'a> System<'a> for AdjacentAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, WantsToMelee>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        println!("Run Adjacent AI System");

        let (
            mut turns,
            factions,
            positions,
            map,
            mut wants_melee,
            entities,
            player
        ) = data;

        let mut turn_done: Vec<Entity> = Vec::new();
        for (entity, _turn, my_faction, pos) in (&entities, &turns, &factions, &positions).join() {
            if entity != *player {
                let mut reactions: Vec<(Entity, Reaction)> = Vec::new();
                let index = map.xy_index(pos.x, pos.y);
                let w = map.width;
                let h = map.height;

                // Add possible reactions to adjacent for each direction
                if pos.x > 0 { evaluate(index - 1, &map, &factions, &my_faction.name, &mut reactions); }
                if pos.x < w - 1 { evaluate(index + 1, &map, &factions, &my_faction.name, &mut reactions); }
                if pos.y > 0 { evaluate(index - w as usize, &map, &factions, &my_faction.name, &mut reactions); }
                if pos.y < h - 1 { evaluate(index + w as usize, &map, &factions, &my_faction.name, &mut reactions); }
                if pos.y > 0 && pos.x > 0 { evaluate((index + w as usize) - 1, &map, &factions, &my_faction.name, &mut reactions); }
                if pos.y > 0 && pos.x < w - 1 { evaluate((index - w as usize) + 1, &map, &factions, &my_faction.name, &mut reactions); }
                if pos.y < h - 1 && pos.x > 0 { evaluate((index + w as usize) - 1, &map, &factions, &my_faction.name, &mut reactions); }
                if pos.y < h - 1 && pos.x < w - 1 { evaluate((index + w as usize) + 1, &map, &factions, &my_faction.name, &mut reactions); }

                let mut done = false;
                for reaction in reactions.iter() {
                    if let Reaction::Attack = reaction.1 {
                        wants_melee.insert(entity, WantsToMelee { target: reaction.0 }).expect("Error inserting melee");
                        done = true;
                    }
                }

                if done { turn_done.push(entity); }
            }
        }

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}

fn evaluate(index: usize, map: &Map, factions: &ReadStorage<Faction>, my_faction: &str, reactions: &mut Vec<(Entity, Reaction)>) {
    crate::spatial::for_each_tile_content(index, |other_entity| {
        if let Some(faction) = factions.get(other_entity) {
            reactions.push((
                other_entity,
                crate::raws::faction_reaction(my_faction, &faction.name, &crate::raws::RAWS.lock().unwrap())
            ));
        }
    });
}
