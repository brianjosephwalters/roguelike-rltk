use specs::prelude::*;
use crate::{spatial, Pools};

use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        ReadStorage<'a, Pools>,
        Entities<'a>
    );

    #[allow(unused_variables, dead_code)]
    fn run(&mut self, data: Self::SystemData) {
        let (mut map,
            position,
            blockers,
            pools,
            entities) = data;
        spatial::clear();
        spatial::populate_blocked_from_map(&*map);
        for (entity, position) in (&entities, &position).join() {
            let mut alive = true;
            if let Some(pools) = pools.get(entity) {
                if pools.hit_points.current < 1 {
                    alive = false;
                }
            }
            if alive {
                let index = map.xy_index(position.x, position.y);
                spatial::index_entity(entity, index, blockers.get(entity).is_some());
            }
            
        }
    }
}
