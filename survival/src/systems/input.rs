#![allow(clippy::module_name_repetitions)]


use crate::actions::PlayerInputAction;
use crate::components;
use crate::game_data::SurvivalState;
use crate::settings::Context;
use amethyst::{
    core::transform::Transform,
    ecs::{
        Entities, Join, Read, ReadExpect, ReadStorage, Resources, SystemData, Write, WriteStorage,
    },
    input::{InputEvent, InputHandler},
    renderer::Camera,
    shrev::{EventChannel, ReaderId},
};

#[derive(Default)]
pub struct System {
    input_reader: Option<ReaderId<InputEvent<PlayerInputAction>>>,
}
impl<'s> amethyst::ecs::System<'s> for System {
    type SystemData = (
        ReadExpect<'s, Context>,
        Write<'s, SurvivalState>,
        Read<'s, InputHandler<PlayerInputAction, PlayerInputAction>>,
        Read<'s, EventChannel<InputEvent<PlayerInputAction>>>,
        Entities<'s>,
        WriteStorage<'s, components::Actionable>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.input_reader = Some(
            Write::<EventChannel<InputEvent<PlayerInputAction>>>::fetch(&res).register_reader(),
        );
    }

    #[allow(clippy::cast_possible_truncation)]
    fn run(
        &mut self,
        (
            _,
            state,
            input,
            input_events,
            entities,
            mut actionables,
            cameras,
            mut transforms, // for debuging
        ): Self::SystemData,
    ) {
        if *state == SurvivalState::Paused {
            for (_, _actionable) in (&entities, &mut actionables).join() {
                let got_input = false;

                // hold-down key actions go here
                if input.action_is_down(&PlayerInputAction::MoveUp).unwrap() {
                    if let Some((_, transform)) = (&cameras, &mut transforms).join().next() {
                        transform.move_up(5.0);
                    }
                }
                if input.action_is_down(&PlayerInputAction::MoveDown).unwrap() {
                    if let Some((_, transform)) = (&cameras, &mut transforms).join().next() {
                        transform.move_down(5.0);
                    }
                }
                if input.action_is_down(&PlayerInputAction::MoveLeft).unwrap() {
                    if let Some((_, transform)) = (&cameras, &mut transforms).join().next() {
                        transform.move_left(5.0);
                    }
                }
                if input.action_is_down(&PlayerInputAction::MoveRight).unwrap() {
                    if let Some((_, transform)) = (&cameras, &mut transforms).join().next() {
                        transform.move_right(5.0);
                    }
                }

                if input.action_is_down(&PlayerInputAction::ZoomIn).unwrap() {
                    if let Some((_, transform)) = (&cameras, &mut transforms).join().next() {
                        *transform.scale_mut() = transform.scale() * 1.1;
                    }
                }
                if input.action_is_down(&PlayerInputAction::ZoomOut).unwrap() {
                    if let Some((_, transform)) = (&cameras, &mut transforms).join().next() {
                        *transform.scale_mut() = transform.scale() * 0.9;
                    }
                }

                // Single shot event actions go here
                if !got_input {
                    for event in input_events.read(self.input_reader.as_mut().unwrap()) {
                        if let InputEvent::ActionPressed(action) = event {
                            match action {
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
