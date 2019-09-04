use amethyst::{assets::ProgressCounter, StateData, StateEvent, Trans};

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
        //slog_trace!(self.log, "Changed state to Running");
    }

    fn on_pause(&mut self, _: StateData<'_, SurvivalData<'_, '_>>) {}

    fn handle_event(
        &mut self,
        data: StateData<'_, SurvivalData<'_, '_>>,
        event: StateEvent,
    ) -> Trans<SurvivalData<'a, 'b>, StateEvent> {
        //slog_trace!(self.log, "Event Running");
       // amethyst_imgui::handle_imgui_events(data.world, &event);
        Trans::None
    }

    fn update(
        &mut self,
        data: StateData<'_, SurvivalData<'_, '_>>,
    ) -> Trans<SurvivalData<'a, 'b>, StateEvent> {
        if data.data.update(&data.world, SurvivalState::Running) != SurvivalState::Running {
            return Trans::Pop;
        }

        Trans::None
    }
}
