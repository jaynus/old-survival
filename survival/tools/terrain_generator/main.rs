extern crate amethyst;
extern crate amethyst_imgui;

use amethyst::{
    assets::{AssetStorage, HotReloadBundle, Loader},
    core::{Transform, TransformBundle},
    ecs::{Entity, ReadExpect, Resources, SystemData, Write},
    prelude::*,
    renderer::{
        Camera, DisplayConfig, DrawFlat2D, Pipeline, PngFormat, Projection, RenderBundle, Stage,
        Texture, TextureHandle, TextureMetadata,
    },
    utils::application_root_dir,
};

use amethyst_imgui::{imgui, imgui::im_str, ImguiState};
use survival::mapgen::{CellData, Generator, GeneratorSettings, IslandGeneratorSettings};

#[derive(Default)]
pub struct ImguiBeginFrameSystem;
impl ImguiBeginFrameSystem {
    pub fn open_frame<'ui>(
        &mut self,
        dimensions: &amethyst::renderer::ScreenDimensions,
        time: &amethyst::core::timing::Time,
        imgui_state: &mut Option<ImguiState>,
    ) -> Option<&'ui imgui::Ui<'ui>> {
        let dimensions: &amethyst::renderer::ScreenDimensions = &dimensions;
        let time: &amethyst::core::timing::Time = &time;

        if dimensions.width() <= 0. || dimensions.height() <= 0. {
            return None;
        }

        let imgui = match imgui_state {
            Some(x) => &mut x.imgui,
            _ => return None,
        };

        let frame = imgui.frame(
            imgui::FrameSize::new(
                f64::from(dimensions.width()),
                f64::from(dimensions.height()),
                1.,
            ),
            time.delta_seconds(),
        );
        std::mem::forget(frame);
        unsafe { imgui::Ui::current_ui() }
    }
}
impl<'s> amethyst::ecs::System<'s> for ImguiBeginFrameSystem {
    type SystemData = (
        ReadExpect<'s, amethyst::renderer::ScreenDimensions>,
        ReadExpect<'s, amethyst::core::timing::Time>,
        Write<'s, Option<ImguiState>>,
    );

    fn run(&mut self, (dimensions, time, mut imgui_state): Self::SystemData) {
        self.open_frame(&dimensions, &time, &mut imgui_state);
    }
}

struct UiState {
    seed: imgui::ImString,
}
impl Default for UiState {
    fn default() -> Self {
        Self {
            seed: "balls".to_string().into(),
        }
    }
}

#[derive(Default)]
pub struct ImguiEndFrameSystem {
    state: UiState,
    height: f32,
    sharpness: f32,
    radius: f32,
    world_pixels: f32,
    num_points: i32,
    num_lloyd: i32,
}
impl<'s> amethyst::ecs::System<'s> for ImguiEndFrameSystem {
    type SystemData = ();

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        let g_d = GeneratorSettings::default();
        let i_d = IslandGeneratorSettings::default();
        self.height = i_d.height as f32;
        self.sharpness = i_d.sharpness as f32;
        self.radius = i_d.radius as f32;
        self.world_pixels = g_d.world_pixels as f32;
        self.num_points = g_d.num_points as i32;
        self.num_lloyd = g_d.num_lloyd as i32;
    }

    fn run(&mut self, _: Self::SystemData) {
        if let Some(ui) = unsafe { imgui::Ui::current_ui() } {
            unsafe {
                (ui as *const imgui::Ui).read_volatile();
                //let root_dock = ui.dockspace_over_viewport(None, imgui::ImGuiDockNodeFlags::PassthruDockspace );
                //ui.show_demo_window(&mut true);
            }

            ui.window(imgui::im_str!("Generate Terrain"))
                .size((300.0, 100.0), imgui::ImGuiCond::FirstUseEver)
                .build(|| {
                    if ui.button(im_str!("Regenerate Island"), (0.0, 0.0)) {
                        let seed = survival::mapgen::seed_from_string(self.state.seed.to_str());

                        let settings = IslandGeneratorSettings {
                            height: f64::from(self.height),
                            sharpness: f64::from(self.sharpness),
                            radius: f64::from(self.radius),
                        };

                        let config = GeneratorSettings {
                            world_pixels: f64::from(self.world_pixels),
                            num_points: self.num_points as usize,
                            num_lloyd: self.num_lloyd as usize,
                            ..GeneratorSettings::default()
                        };

                        generate_new_map(arrayref::array_ref![seed, 0, 32], &config, &settings)
                            .unwrap();
                    }
                    ui.input_text(im_str!("Seed"), &mut self.state.seed).build();
                    ui.separator();
                    ui.slider_float(im_str!("Box Size"), &mut self.world_pixels, 1.0, 5000.0)
                        .build();
                    ui.slider_int(im_str!("Points #"), &mut self.num_points, 1, 20000)
                        .build();
                    ui.slider_int(im_str!("Lloyd Reductions"), &mut self.num_lloyd, 1, 20)
                        .build();
                    ui.separator();
                    ui.slider_float(im_str!("Start Height"), &mut self.height, 0.1, 1.0)
                        .build();
                    ui.slider_float(im_str!("Radius"), &mut self.radius, 0.1, 0.99999)
                        .build();
                    ui.slider_float(im_str!("Sharpness"), &mut self.sharpness, 0.1, 2.0)
                        .build();
                });
        }
    }
}

struct Example;
impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        let texture_handle = load_texture(world, "map.png");
        let _image = init_image(world, &texture_handle);

        init_camera(world);
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> Trans<GameData<'static, 'static>, StateEvent> {
        //amethyst_imgui::handle_imgui_events(data.world, &event);

        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let resources = application_root_dir()?.join("tools/terrain_generator/resources");
    let config = DisplayConfig::load(resources.join("display_config.ron"));
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.1, 0.1, 0.1, 1.0], 1.0)
            .with_pass(DrawFlat2D::new())
            .with_pass(amethyst_imgui::DrawUi::default().docking()),
    );

    let game_data = GameDataBuilder::default()
        .with(ImguiBeginFrameSystem::default(), "imgui_begin_frame", &[])
        .with(
            ImguiEndFrameSystem::default(),
            "imgui_end_frame",
            &["imgui_begin_frame"],
        )
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with_bundle(HotReloadBundle::default())?;

    let mut game = Application::build(resources, Example)?.build(game_data)?;
    game.run();

    Ok(())
}

fn generate_new_map(
    seed: &[u8; 32],
    config: &GeneratorSettings,
    settings: &IslandGeneratorSettings,
) -> amethyst::Result<()> {
    use rand::SeedableRng;

    let mut generator = Generator::new(rand::rngs::StdRng::from_seed(*seed));

    let mut cells = generator.gen_voronoi::<CellData>(&config);
    generator.create_island(config, settings, &mut cells);

    generator
        .save_heightmap_image(
            &config,
            &application_root_dir()?.join("tools/terrain_generator/resources/map.png"),
            &cells,
        )
        .unwrap();

    Ok(())
}

fn init_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_z(1.0);
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            -250.0, 250.0, -250.0, 250.0,
        )))
        .with(transform)
        .build();
}

fn init_image(world: &mut World, texture: &TextureHandle) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_x(0.0);
    transform.set_translation_y(0.0);

    world
        .create_entity()
        .with(transform)
        .with(texture.clone())
        .build()
}

fn load_texture(world: &mut World, png_path: &str) -> TextureHandle {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    loader.load(
        png_path,
        PngFormat,
        TextureMetadata::srgb_scale(),
        (),
        &texture_storage,
    )
}
