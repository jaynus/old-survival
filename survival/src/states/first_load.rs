use amethyst::{
    assets::ProgressCounter,
    assets::{AssetStorage, Loader},
    ecs::World,
    renderer::{
        PngFormat, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle, Texture, TextureMetadata,
    },
    StateData, StateEvent, Trans,
};
use specs_static::WorldExt;

use slog::slog_trace;

use crate::settings;
use crate::SurvivalData;

fn load_sprite_sheet(
    world: &mut World,
    png_path: &str,
    ron_path: &str,
    progress_counter: &mut ProgressCounter,
) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            png_path,
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat,
        texture_handle,
        progress_counter,
        &sprite_sheet_store,
    )
}

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
    fn on_start(&mut self, data: StateData<'_, SurvivalData<'_, '_>>) {
        let world = data.world;

        slog_trace!(self.log, "Changed state to first_load");

        let default_sprite_sheet = load_sprite_sheet(
            world,
            "spritesheets/Bisasam_16x16.png",
            "spritesheets/Bisasam_16x16.ron",
            &mut self.progress_counter,
        );

        // How do we pass this along?
        world.res.fetch_mut::<settings::Context>().spritesheet = Some(default_sprite_sheet);

        crate::assets::StorageSource::<crate::assets::Item>::apply(
            &std::path::Path::new("resources/data/items.ron"),
            world,
        )
        .unwrap();

        // Register tile components
        world.register_tile_comp::<crate::components::FlaggedSpriteRender, crate::tiles::TileId>();
        world.register_tile_comp::<amethyst::renderer::Flipped, crate::tiles::TileId>();
        world.register_tile_comp::<amethyst::renderer::Rgba, crate::tiles::TileId>();
        world
            .register_tile_comp::<amethyst::core::transform::Transform, crate::tiles::TileId>(
            );
        world.register_tile_comp::<crate::tiles::TileEntities, crate::tiles::TileId>();

        world.register_tile_comp::<crate::components::Obstruction, crate::tiles::TileId>();
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, SurvivalData<'_, '_>>,
        _: StateEvent,
    ) -> Trans<SurvivalData<'a, 'b>, StateEvent> {
        slog_trace!(self.log, "Event First Load");
        Trans::None
    }

    fn update(
        &mut self,
        _: StateData<'_, SurvivalData<'_, '_>>,
    ) -> Trans<SurvivalData<'a, 'b>, StateEvent> {
        //if self.progress_counter.num_assets() == self.progress_counter.num_finished() {
        println!("Transition away from load");
        return Trans::Switch(Box::new(super::Level::new(self.log.clone())));
        //}
        //Trans::None
    }
}
