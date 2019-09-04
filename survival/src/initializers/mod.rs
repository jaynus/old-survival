use crate::assets;
use crate::components;
use amethyst::{
    core::{
        components::Parent,
        math::{Vector3},
    },
    ecs::{Builder, Entity, World},
};

#[derive(Copy, Clone, Debug, strum_macros::Display)]
pub enum SpawnType {
    TilePosition(Vector3<u32>),
    TransformPosition(Vector3<f32>),
    Parent(Entity),
}

pub fn spawn_item(
    world: &mut World,
    spawn_type: SpawnType,
    name: &str,
    properties: Option<Vec<crate::assets::item::Property>>,
) -> Entity {
    let (details_handle, is_container) = {
        let item_storage = world.res.fetch::<assets::ItemStorage>();
        let item_details = item_storage.read().unwrap();

        (
            item_details.handles.get(name).unwrap().clone(),
            item_details
                .data
                .get(name)
                .unwrap()
                .flags
                .contains(assets::item::ItemFlag::Container),
        )
    };

    let mut builder = world.create_entity().with(components::Item {
        handle: details_handle,
        properties: match properties {
            Some(p) => p,
            None => Vec::new(),
        },
    });

    if is_container {
        builder = builder.with(components::Container {});
    }

    match spawn_type {
        SpawnType::TilePosition(_pos) => unimplemented!("Not implemented"),
        SpawnType::TransformPosition(_pos) => unimplemented!("Not implemented"),
        SpawnType::Parent(parent_entity) => {
            builder = builder.with(Parent {
                entity: parent_entity,
            });
        }
    }

    builder.build()
}
