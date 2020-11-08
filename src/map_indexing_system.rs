extern crate specs;
use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>
    );

    #[allow(unused_variables, dead_code)]
    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for (entity, position) in (&entities, &position).join() {
            let index = map.xy_index(position.x, position.y);
            let _p: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked[index] = true;
            }
            map.tile_content[index].push(entity);
        }
    }
}