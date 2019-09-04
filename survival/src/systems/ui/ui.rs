#![allow(clippy::module_name_repetitions)]

use crate::settings::Context;
use amethyst::{
    ecs::{Entity, LazyUpdate, Read, ReadExpect, Resources, SystemData, Write, WriteStorage},
    input::InputEvent,
    renderer::HiddenPropagate,
    shrev::{EventChannel, ReaderId},
    ui::{UiFinder, UiText},
    utils::fps_counter::FPSCounter,
};

use crate::actions::PlayerInputAction;
use amethyst_imgui::imgui;

use super::ImGuiDraw;

#[derive(Default)]
pub struct System {
    draw_call_reader_id: Option<ReaderId<ImGuiDraw>>,
    main_ui: Option<Entity>,
    inventory: Option<Entity>,
    input_reader_id: Option<ReaderId<InputEvent<PlayerInputAction>>>,
}

impl<'s> amethyst::ecs::System<'s> for System {
    type SystemData = (
        ReadExpect<'s, Context>,
        Read<'s, EventChannel<ImGuiDraw>>,
        Read<'s, EventChannel<InputEvent<PlayerInputAction>>>,
        Read<'s, FPSCounter>,
        WriteStorage<'s, HiddenPropagate>,
        WriteStorage<'s, UiText>,
        UiFinder<'s>,
        Read<'s, LazyUpdate>,
    );

    fn run(
        &mut self,
        (_, imgui_draw_events, _input_events, fps, _hidden_storage, mut texts, finder, lazy): Self::SystemData,
    ) {
        if let Some(fps_entity) = finder.find("fps") {
            if let Some(fps_display) = texts.get_mut(fps_entity) {
                fps_display.text = format!("FPS: {:.2}", fps.sampled_fps());
            }
        }

        // Get the current ui
        let ui = unsafe { imgui::Ui::current_ui() };
        if let Some(ui) = ui {
            for draw_call in imgui_draw_events.read(self.draw_call_reader_id.as_mut().unwrap()) {
                (draw_call)(ui, &lazy)
            }
        }
    }
    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.input_reader_id = Some(
            Write::<EventChannel<InputEvent<PlayerInputAction>>>::fetch(&res).register_reader(),
        );

        self.draw_call_reader_id =
            Some(res.fetch_mut::<EventChannel<ImGuiDraw>>().register_reader());

        //let mut creator: UiCreator<'_> = SystemData::fetch(res);
        //let mut hidden: WriteStorage<'_, HiddenPropagate> = SystemData::fetch(res);

        //self.main_ui = Some(creator.create("ui/main_ui.ron", ()));
    }
}
