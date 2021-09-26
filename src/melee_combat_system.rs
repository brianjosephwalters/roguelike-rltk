use specs::prelude::*;
use super::{Pools, WantsToMelee, Name, SufferDamage, GameLog};
use crate::{Attributes, Skills, Position, Skill};
use crate::gamesystem::skill_bonus;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Pools>,
        WriteExpect<'a, rltk::RandomNumberGenerator>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut log,
            mut wants_melee,
            names,
            attributes,
            skills,
            mut inflict_damage,
            positions,
            pools,
            mut rng,
        ) = data;

        for (entity, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools)
            in (&entities, &wants_melee, &names, &attributes, &skills, &pools).join()
        {
            // Are the attacker and defender alive? Only attack if they are
            println!("Target: {}", names.get(wants_melee.target).unwrap().name);
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();

            if attacker_pools.hit_points.current > 0 && target_pools.hit_points.current > 0  {
                let target_name = names.get(wants_melee.target).unwrap();
                let natural_roll = rng.roll_dice(1, 20);
                let attribute_hit_bonus = attacker_attributes.might.bonus;
                let skill_hit_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                let weapon_hit_bonus = 0; // TODO: Once weapons support this
                let mut status_hit_bonus = 0;
                let modified_hit_roll = natural_roll + attribute_hit_bonus + skill_hit_bonus + weapon_hit_bonus + status_hit_bonus;

                let base_armor_class = 10;
                let armor_quickness_bonus = target_attributes.quickness.bonus;
                let armor_skill_bonus = skill_bonus(Skill::Defense, &*target_skills);
                let armor_item_bonus = 0; // TODO: Once armor supports this
                let armor_class = base_armor_class + armor_quickness_bonus + armor_skill_bonus + armor_item_bonus;

                if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armor_class) {
                    // Target hit! Until we support weapons, we're going with 1d4
                    let base_damage = rng.roll_dice(1, 4);
                    let attr_damage_bonus = attacker_attributes.might.bonus;
                    let skill_damage_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                    let weapon_damage_bonus = 0;

                    let damage = i32::max(0, base_damage + attr_damage_bonus + skill_hit_bonus + skill_damage_bonus + weapon_damage_bonus);
                    SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    log.entries.push(format!("{} hits {}, for {} hp.", &name.name, &target_name.name, damage));
                } else if natural_roll == 1 {
                    // Natural 1 miss
                    log.entries.push(format!("{} considers attacking {}, but misjudges the timing.", name.name, target_name.name));
                } else {
                    // Miss
                    log.entries.push(format!("{} attacks {}, but can't connect.", name.name, target_name.name));
                }
            }
        }

        wants_melee.clear();
    }
}
