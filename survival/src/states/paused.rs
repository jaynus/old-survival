use amethyst::{assets::ProgressCounter, StateData, StateEvent, Trans};

use slog::slog_trace;

use crate::game_data::SurvivalState;
use crate::SurvivalData;

pub struct State {
    progress_counter: ProgressCounter,
    log: slog::Logger,
}
impl State {
    pub fn new(root_logger: slog::Logger) -> Self {
        Self {
            progress_counter: ProgressCounter::default(),
            log: root_logger,
        }
    }
}
impl<'a, 'b> amethyst::State<SurvivalData<'a, 'b>, StateEvent> for State {
    fn on_start(&mut self, _: StateData<'_, SurvivalData<'_, '_>>) {
        slog_trace!(self.log, "Changed state to Paused");
    }

    fn on_pause(&mut self, _: StateData<'_, SurvivalData<'_, '_>>) {}

    fn handle_event(
        &mut self,
        data: StateData<'_, SurvivalData<'_, '_>>,
        event: StateEvent,
    ) -> Trans<SurvivalData<'a, 'b>, StateEvent> {
        //slog_trace!(self.log, "Event Paused");
        //amethyst_imgui::handle_imgui_events(data.world, &event);

        // Wait for player input and trans if we get it.

        Trans::None
    }

    fn update(
        &mut self,
        data: StateData<'_, SurvivalData<'_, '_>>,
    ) -> Trans<SurvivalData<'a, 'b>, StateEvent> {
        if data.data.update(&data.world, SurvivalState::Paused) != SurvivalState::Paused {
            return Trans::Push(Box::new(super::Running::new(self.log.clone())));
        }

        Trans::None
    }
}
