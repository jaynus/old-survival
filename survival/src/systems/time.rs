#![allow(clippy::module_name_repetitions)]

use crate::components;
use crate::settings::Context;
use amethyst::ecs::{Entities, Entity, ReadExpect, Write, WriteStorage};

#[derive(Default, Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimeState {
    pub current_time: u64,
}

#[derive(Default)]
pub struct System;
impl<'s> amethyst::ecs::System<'s> for System {
    type SystemData = (
        ReadExpect<'s, Context>,
        Write<'s, TimeState>,
        Entities<'s>,
        WriteStorage<'s, components::TimeAvailable>,
    );

    fn run(&mut self, (_, _time_state, _entities, _time_avialables): Self::SystemData) {}
}

pub fn has_time(time: u64, _entity: Entity, time_comp: &mut components::TimeAvailable) -> bool {
    time_comp.has(time)
}

pub fn consume_time(time: u64, _entity: Entity, time_comp: &mut components::TimeAvailable) {
    time_comp.consume(time);
}
