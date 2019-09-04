extern crate amethyst;
extern crate amethyst_imgui;

use amethyst::{
    assets::{AssetLoaderSystemData, HotReloadBundle},
    core::{Transform, TransformBundle},
    ecs::{Resources, SystemData},
    prelude::*,
    renderer::{
        ActiveCamera, Camera, DisplayConfig, DrawShaded, Light, Material, MaterialDefaults, Mesh,
        Pipeline, PointLight, PosNormTex, Projection, RenderBundle, Rgba, Shape, Stage,
        Texture,
    },
    utils::application_root_dir,
};

use amethyst_imgui::{imgui, imgui::im_str};
use survival::mapgen::{CellData, Generator, GeneratorSettings, IslandGeneratorSettings};

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
pub struct DrawGenerationUiSystem {
    state: UiState,
    height: f32,
    sharpness: f32,
    radius: f32,
    world_pixels: f32,
    num_points: i32,
    num_lloyd: i32,
}
impl<'s> amethyst::ecs::System<'s> for DrawGenerationUiSystem {
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
        let StateData { world, .. } = data;
        let mat_defaults = world.read_resource::<MaterialDefaults>().0.clone();

        let mesh = world.exec(|loader: AssetLoaderSystemData<'_, Mesh>| {
            loader.load_from_data(Shape::Cube.generate::<Vec<PosNormTex>>(None), ())
        });
        let mtl = world.exec(|loader: AssetLoaderSystemData<'_, Texture>| {
            let albedo = loader.load_from_data([0.0, 0.0, 1.0, 0.0].into(), ());
            Material {
                albedo,
                ..mat_defaults
            }
        });

        let mut trans = Transform::default();
        trans.set_translation_xyz(-5.0, 0.0, 0.0);
        world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .build();

        initialise_lights(world);
        initialise_camera(world);
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

fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, -20.0, 10.0);
    transform.prepend_rotation_x_axis(1.3257521);

    let camera = world
        .create_entity()
        .with(Camera::from(Projection::perspective(
            1.0,
            std::f32::consts::FRAC_PI_3,
        )))
        .with(transform)
        .build();

    world.add_resource(ActiveCamera {
        entity: Some(camera),
    });
}

fn initialise_lights(world: &mut World) {
    let light: Light = PointLight {
        intensity: 100.0,
        radius: 1.0,
        color: Rgba::white(),
        ..Default::default()
    }
    .into();

    use amethyst::core::ecs::Component;

    let mut transform = Transform::default();
    transform.set_translation_xyz(5.0, -20.0, 15.0);

    // Add point light.
    world.create_entity().with(light).with(transform).build();
}S

fn generate_new_map(
    seed: &[u8; 32],
    config: &GeneratorSettings,
    settings: &IslandGeneratorSettings,
) -> amethyst::Result<()> {
    use rand::SeedableRng;
    use survival::map::WorldMap;

    let mut generator = Generator::new(rand::rngs::StdRng::from_seed(*seed));

    let mut cells = generator.gen_voronoi::<CellData>(&config);
    generator.create_island(config, settings, &mut cells);

    let mut worldmap = WorldMap::new(&config);
    worldmap.heightmap = generator.generate_height_map(&config, &cells).unwrap();
    worldmap.moisture = generator.generate_moisture_map(&config, &cells).unwrap();

    let _region = worldmap.generate_chunk(0);

    Ok(())
}

fn main() -> amethyst::Result<()> {
    use slog::Drain;

    amethyst::start_logger(amethyst::LoggerConfig::default());

    // Make sure to save the guard, see documentation for more information
    let decorator = slog_term::TermDecorator::new().force_color().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let async_drain = slog_async::Async::new(drain).build().fuse();
    let root_log = slog::Logger::root(
        async_drain,
        slog::o!("@" =>
         slog::FnValue(move |info| {
             format!("{}({}:{})",
                     info.module(),
                     info.file(),
                     info.line(),
                     )
         })
        ),
    );
    let _guard = slog_scope::set_global_logger(root_log);

    let resources = application_root_dir()?.join("tools/region_generator/resources");
    let config = DisplayConfig::load(resources.join("display_config.ron"));
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.1, 0.1, 0.1, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new())
            .with_pass(amethyst_imgui::DrawUi::default()),
    );

    let game_data = GameDataBuilder::default()
        .with(
            survival::systems::ImguiBeginFrameSystem::default(),
            "imgui_begin_frame",
            &[],
        )
        .with(
            DrawGenerationUiSystem::default(),
            "draw_ui",
            &["imgui_begin_frame"],
        )
        .with(
            survival::systems::ImguiEndFrameSystem::default(),
            "imgui_end_frame",
            &["imgui_begin_frame", "draw_ui"],
        )
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with_bundle(HotReloadBundle::default())?;

    let mut game = Application::build(resources, Example)?.build(game_data)?;
    game.run();

    Ok(())
}
