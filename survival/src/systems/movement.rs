#![allow(clippy::module_name_repetitions)]
use crate::settings::Context;
use amethyst::{
    core::components::Transform,
    ecs::{Entities, Join, Read, ReadExpect, Resources, SystemData, WriteStorage},
};

use crate::actions::{Action};
use crate::components;
use crate::utils::ComponentEventReader;



use crate::settings::Config;
use crate::tiles::{ReadTiles, Tiles};

#[derive(Default)]
pub struct System {
    action_reader: ComponentEventReader<components::Actionable, Action>,
}
impl<'s> amethyst::ecs::System<'s> for System {
    type SystemData = (
        ReadExpect<'s, Context>,
        Read<'s, Config>,
        ReadExpect<'s, Tiles>,
        Entities<'s>,
        WriteStorage<'s, components::TimeAvailable>,
        WriteStorage<'s, components::Actionable>,
        WriteStorage<'s, Transform>,
        // Tile storages
        ReadTiles<'s, components::Obstruction>,
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.action_reader.setup(res);
    }

    fn run(
        &mut self,
        (
            _context,
            _game_config,
            _tiles,
            entities,
            mut times,
            mut actionables,
            mut transforms,
            _tile_impassable,
        ): Self::SystemData,
    ) {
        self.action_reader.maintain(&entities, &mut actionables);

        // Read components...
        for (entity, _time_comp, actionable, _transform) in
            (&entities, &mut times, &mut actionables, &mut transforms).join()
        {
            for _event in self.action_reader.read(entity, actionable) {
                /*
                if let Action::Move(direction) = event {
                    if crate::systems::time::has_time(1, entity, time_comp) {
                        //slog_trace!(context.logs.root, "Moving entity: {}", direction);
                        // Once its confirmed they can do it, run it
                        crate::systems::time::consume_time(1, entity, time_comp);

                        // And finally, move one tile in the given direction
                        let mut target = transform.clone();

                        match direction {
                            Direction::N => {
                                target.move_up(5.0);
                            }
                            Direction::S => {
                                target.move_down(5.0);
                            }
                            Direction::E => {
                                target.move_right(5.0);
                            }
                            Direction::W => {
                                target.move_left(5.0);
                            }
                            _ => {
                                slog_error!(context.logs.root, "Unsupported direction");
                            }
                        }

                        // Can we actually go to the target?
                        let target_tile = tiles.world_to_tile(target.translation(), &game_config);

                        if tile_impassable
                            .get(tiles.id_from_vector(target_tile))
                            .is_some()
                        {
                            // We cant do the move!
                            slog_warn!(
                                context.logs.root,
                                "Attempted to move to impassable tile: ({},{})",
                                target_tile.x,
                                target_tile.y
                            );
                            continue;
                        }

                        *transform = target;
                    }
                }
                */
            }
        }
    }
}
