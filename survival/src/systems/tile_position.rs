use amethyst::{
    core::{math::Vector3, Transform},
    ecs::{
        storage::ComponentEvent, BitSet, Entities, Join, ReadExpect, ReadStorage, Resources,
        SystemData, WriteStorage,
    },
    shrev::ReaderId,
};

use crate::{
    components::TilePosition,
    settings::{Config, Context},
    tiles::{TileEntities, Tiles, WriteTiles},
};

use slog::slog_error;

#[derive(Default)]
pub struct System {
    transform_reader: Option<ReaderId<ComponentEvent>>,
    dirty: BitSet,
}
impl<'s> amethyst::ecs::System<'s> for System {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, Context>,
        ReadExpect<'s, Config>,
        ReadExpect<'s, Tiles>,
        WriteTiles<'s, TileEntities>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, TilePosition>,
    );

    fn run(
        &mut self,
        (
            entities,
            context,
            game_settings,
            tiles,
            mut tile_entities_map,
            transforms,
            mut tile_positions,
        ): Self::SystemData,
    ) {
        self.dirty.clear();

        for event in transforms
            .channel()
            .read(self.transform_reader.as_mut().unwrap())
        {
            match event {
                ComponentEvent::Modified(id) | ComponentEvent::Inserted(id) => {
                    //slog_trace!(context.logs.root, "New Transform component");
                    self.dirty.add(*id);
                }
                ComponentEvent::Removed(_) => (),
            }
        }

        // parallel update dirty transforms
        for (entity, transform, tile_position, _) in
            (&entities, &transforms, &mut tile_positions, &self.dirty).join()
        {
            let new_position = tiles.world_to_tile(transform.translation(), &game_settings);

            // Did they actually move tiles? LOL
            if tile_position.coord.xyz() != new_position {
                //slog_trace!(context.logs.root, "TileMove E:{}: ({},{}) -> ({},{}) ", entity.id(),
                //tile_position.coord.x, tile_position.coord.y,
                //new_position.x, new_position.y);

                if let Some(entities_list) =
                    tile_entities_map.get_mut(tiles.id_from_vector(tile_position.coord.xyz()))
                {
                    entities_list.0.remove(&entity);
                } else {
                    slog_error!(
                        context.logs.root,
                        "({}, {}) - E:{} - Invalid tile for a position removal!?",
                        tile_position.coord.x,
                        tile_position.coord.y,
                        entity.id()
                    );
                }

                // Finally, update the tileposition on the entity
                tile_position.coord = Vector3::new(new_position.x as u32, new_position.y as u32, 0);

                if let Some(entities_list) =
                    tile_entities_map.get_mut(tiles.id_from_vector(new_position))
                {
                    entities_list.0.insert(entity);
                } else {
                    slog_error!(
                        context.logs.root,
                        "({}, {}) - E:{} - Invalid tile for a position insertion!?",
                        new_position.x,
                        new_position.y,
                        entity.id()
                    );
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.transform_reader = Some(WriteStorage::<Transform>::fetch(&res).register_reader());
    }
}
