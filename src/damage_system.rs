use specs::prelude::*;
use super::{ Pools, SufferDamage, Player, Name, GameLog, RunState};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, Pools>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut pools, mut damage) = data;

        for (mut pools, damage) in (&mut pools, &damage).join() {
            pools.hit_points.current -= damage.amount.iter().sum::<i32>();
        }
        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // Using a scope to make the borrow checker happy.
    {
        let pools = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let entities = ecs.entities();
        let mut log = ecs.write_resource::<GameLog>();

        for (entity, pools) in (&entities, &pools).join() {
            if pools.hit_points.current < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("{} is dead.", &victim_name.name))
                        }
                        dead.push(entity)
                    },
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
