#![warn(clippy::pedantic, clippy::all)]
#![allow(
    clippy::type_complexity,
    clippy::empty_enum,
    clippy::default_trait_access
)]
#![allow(
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::similar_names
)] // TODO: revisit these
#![allow(non_upper_case_globals)]
#![feature(custom_attribute, core_intrinsics, futures_api)]
#![allow(dead_code)]

#[macro_use]
pub mod bitflags_serial;

pub mod goap;
pub mod mapgen;
pub mod pathfinding;
pub mod system_chain;

pub mod assets;
pub mod components;
pub mod render;
pub mod settings;
pub mod states;
pub mod systems;
pub mod tiles;
pub mod utils;

pub mod actions;

pub mod inventory;

pub mod game_data;
pub mod initializers;

pub mod map;

pub use game_data::{SurvivalData, SurvivalDataBuilder, SurvivalState};

use amethyst::{
    assets::{HotReloadBundle, PrefabLoaderSystem},
    core::{frame_limiter::FrameRateLimitStrategy, TransformBundle},
    input::InputBundle,
    prelude::*,
    renderer::{DisplayConfig, DrawFlat2D, Pipeline, PosNormTex, RenderBundle, Stage},
    ui::UiBundle,
    utils::application_root_dir,
    utils::{fps_counter::FPSCounterBundle, scene::BasicScenePrefab},
};

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

pub fn run(root_logger: &slog::Logger) -> amethyst::Result<()> {
    let root = application_root_dir()?.join("resources");

    let display_config = DisplayConfig::load(root.join("display_config.ron"));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.1, 0.1, 0.1, 1.0], 1.0)
            .with_pass(crate::render::tiles::Pass::new())
            .with_pass(DrawFlat2D::new())
            .with_pass(amethyst::ui::DrawUi::new())
            //.with_pass(amethyst_imgui::DrawUi::default().docking()),
    );

    let game_config = crate::settings::Config::load(root.join("game_settings.ron"));
    let game_context = crate::settings::Context {
        logs: crate::settings::Logs {
            root: root_logger.clone(),
        },
        spritesheet: None,
    };

    let game_data = SurvivalDataBuilder::new(game_context, display_config.clone(), game_config)
        .with_core_bundle(TransformBundle::new())?
        .with_core_bundle(
            InputBundle::<actions::PlayerInputAction, actions::PlayerInputAction>::new()
                .with_bindings_from_file(root.join("input.ron"))?,
        )?
        .with_core(
            systems::ImguiBeginFrameSystem::default(),
            "imgui_begin_frame",
            &[],
        )
        .with_core(
            systems::DebugSystem::default(),
            "debug",
            &["imgui_begin_frame"],
        )
        .with_core_bundle(HotReloadBundle::default())?
        .with_core_bundle(UiBundle::<
            actions::PlayerInputAction,
            actions::PlayerInputAction,
        >::new())?
        .with_core(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_core_bundle(FPSCounterBundle::default())?
        .with_core_bundle(
            RenderBundle::new(pipe, Some(display_config.clone()))
                .with_sprite_sheet_processor()
                .with_sprite_visibility_sorting(&[])
                .with_hide_hierarchy_system(),
        )?
        .with_core(
            systems::UiSystem::default(),
            "ui",
            &["imgui_begin_frame", "ui_loader"],
        )
        .with_core(
            systems::ui::InventoryWindowSystem::default(),
            "inventory_window_system",
            &["ui"],
        )
        .with_core(
            systems::ImguiEndFrameSystem::default(),
            "imgui_end_frame",
            &["imgui_begin_frame", "ui", "debug"],
        ) // All systems which use imgui must be here.
        .with_level(systems::WearingSystem::default(), "wearing", &[])
        .with_level(systems::InputSystem::default(), "input", &[])
        .with_level(systems::TilePositionSystem::default(), "tile_position", &[])
        .with_level(systems::MovementSystem::default(), "movement", &[])
        .with_level(systems::TimeSystem::default(), "time", &[])
        .with_level(systems::InitiativeSystem::default(), "initiative", &[]);

    let mut game = Application::build(root, crate::states::FirstLoad::new(root_logger.clone()))?
        .with_frame_limit(FrameRateLimitStrategy::Unlimited, 9999)
        .build(game_data)?;

    game.run();

    Ok(())
}
