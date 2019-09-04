#![allow(clippy::module_name_repetitions)]
use crate::components;
use crate::game_data::SurvivalState;
use crate::settings::Context;
use amethyst::ecs::{ReadExpect, Resources, SystemData, Write, WriteStorage};

use crate::goap::Planner;

#[derive(Default)]
pub struct System;
impl<'s> amethyst::ecs::System<'s> for System {
    type SystemData = (
        ReadExpect<'s, Context>,
        Write<'s, SurvivalState>,
        WriteExpect<'s, Planner>
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
    }

    fn run(&mut self, (_, mut state, planner): Self::SystemData) {

    }
}
