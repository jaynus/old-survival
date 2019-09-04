#![allow(clippy::module_name_repetitions)]

use crate::actions::PlayerInputAction;
use crate::components;
use crate::settings::Context;
use amethyst::{
    assets::AssetStorage,
    core::ParentHierarchy,
    ecs::{
        Entities, Entity, Read, ReadExpect, Resources, SystemData,
        WriteStorage,
    },
    input::InputEvent,
    renderer::HiddenPropagate,
    shrev::{EventChannel, ReaderId},
    ui::{UiFinder, UiText},
};
use crate::assets;

#[derive(Default)]
pub struct System {
    main_ui: Option<Entity>,
    inventory: Option<Entity>,
    input_reader_id: Option<ReaderId<InputEvent<PlayerInputAction>>>,
}

impl<'s> amethyst::ecs::System<'s> for System {
    type SystemData = (
        ReadExpect<'s, Context>,
        Entities<'s>,
        Read<'s, EventChannel<InputEvent<PlayerInputAction>>>,
        ReadExpect<'s, ParentHierarchy>,
        WriteStorage<'s, components::Item>,
        WriteStorage<'s, components::Container>,
        WriteStorage<'s, HiddenPropagate>,
        WriteStorage<'s, UiText>,
        Read<'s, AssetStorage<assets::Item>>,
        UiFinder<'s>,
    );

    fn run(
        &mut self,
        (
            _,
            _entities,
            _input_events,
            _hierarchy,
            _item_storage,
            _container_storage,
            _hidden_storage,
            _text_storage,
            _item_details,
            _finder,
        ): Self::SystemData,
    ) {

    }
    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        //    let mut creator: UiCreator<'_> = SystemData::fetch(res);
        //   self.inventory = Some(creator.create("ui/inventory.ron", ()));
    }
}
