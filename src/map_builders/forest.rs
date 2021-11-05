use crate::map_builders::BuilderChain;
use crate::map_builders::cellular_automata::CellularAutomataBuilder;
use crate::map_builders::area_starting_points::{AreaStartingPosition, XStart, YStart};
use crate::map_builders::cull_unreachable::CullUnreachable;
use crate::map_builders::voronoi_spawning::VoronoiSpawning;
use crate::map_builders::distant_exit::DistantExit;

pub fn forest_builder(new_depth: i32, _rng: &mut rltk::RandomNumberGenerator, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into the Woods");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER));

    // Setup an exit and spawn mobs
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain
}
