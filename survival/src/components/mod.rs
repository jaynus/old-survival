use crate::utils::HasChannel;
use amethyst::{
    assets::Handle,
    core::{components::Transform, math::Vector3},
    ecs::prelude::*,
    renderer::SpriteSheetHandle,
    shrev::EventChannel,
};
use bitflags::*;
use serde::{Deserialize, Serialize};
use specs_derive::Component;

pub mod ai;

#[derive(Component, Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[storage(NullStorage)]
pub struct IsTurn;

#[derive(Default, Serialize, Deserialize)]
pub struct Actionable {
    #[serde(skip_serializing, skip_deserializing)]
    pub channel: EventChannel<crate::actions::Action>,
}
impl Component for Actionable {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
impl HasChannel<crate::actions::Action> for Actionable {
    fn channel(&self) -> &EventChannel<crate::actions::Action> {
        &self.channel
    }

    fn channel_mut(&mut self) -> &mut EventChannel<crate::actions::Action> {
        &mut self.channel
    }
}

#[derive(
    Clone, Debug, PartialEq, Deserialize, Serialize, strum_macros::EnumString, strum_macros::Display,
)]
pub enum TreeFamily {
    Deciduous,
    Coniferous,
}

#[derive(
    Clone, Debug, PartialEq, Deserialize, Serialize, strum_macros::EnumString, strum_macros::Display,
)]
pub enum TreeKind {
    Pine,   //(TreeFamily::Coniferous),
    Fur,    //(TreeFamily::Coniferous),
    Spruce, //(TreeFamily::Coniferous),
    Cedar,  //(TreeFamily::Coniferous),

    Oak,    //(TreeFamily::Deciduous),
    Elm,    //(TreeFamily::Deciduous),
    Maple,  //(TreeFamily::Deciduous),
    Birch,  //(TreeFamily::Deciduous),
    Willow, //(TreeFamily::Deciduous)
}

#[derive(
    Component,
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    strum_macros::EnumString,
    strum_macros::Display,
)]
pub enum Obstruction {
    Impassable,
    Slow(f32),
}
impl Default for Obstruction {
    fn default() -> Self {
        Obstruction::Impassable
    }
}

#[derive(Component, Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[storage(NullStorage)]
pub struct ZTransition;

#[derive(Component, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[storage(DenseVecStorage)]
pub struct Tree {
    kind: TreeKind,
    size: f32,
    branches: f32,
}

#[derive(Component, Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[storage(DenseVecStorage)]
pub struct TimeAvailable(pub u64);
impl TimeAvailable {
    pub fn has(&self, time: u64) -> bool {
        self.0 >= time
    }
    pub fn consume(&mut self, time: u64) {
        self.0 -= time;
    }
    pub fn add(&mut self, time: u64) {
        self.0 += time;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlaggedSpriteRender {
    /// Handle to the sprite sheet of the sprite
    pub sprite_sheet: SpriteSheetHandle,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
}

impl Component for FlaggedSpriteRender {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[storage(DenseVecStorage)]
pub struct Container;

#[derive(Component, Clone, Debug)]
#[storage(DenseVecStorage)]
pub struct Item {
    pub handle: Handle<crate::assets::item::Details>,
    pub properties: Vec<crate::assets::item::Property>,
}
impl PartialEq<Item> for Item {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
#[storage(DenseVecStorage)]
pub struct TilePosition {
    pub coord: Vector3<u32>,
}
impl Default for TilePosition {
    fn default() -> Self {
        Self {
            coord: Vector3::new(0, 0, 0),
        }
    }
}
impl TilePosition {
    pub fn new(coord: Vector3<u32>) -> Self {
        Self { coord }
    }
    pub fn from_transform(
        transform: &Transform,
        tiles: crate::tiles::Tiles,
        game_settings: &crate::settings::Config,
    ) -> Self {
        let position = tiles.world_to_tile(transform.translation(), &game_settings);;
        Self {
            coord: Vector3::new(position.x as u32, position.y as u32, 0),
        }
    }
}

bitflags_serial! {
    pub struct InteractionType: u64 {
        const None =                0;
        const Chop =                1 << 1;
        const Pickup =              1 << 2;
        const Dig  =                1 << 3;
        const Hit =                 1 << 4;
        const LightFire =           1 << 5;
        const Cut =                 1 << 6;
        const Hammer =              1 << 7;

    }
}

#[derive(Component, Default, Copy, Clone, Debug, Serialize, Deserialize)]
#[storage(DenseVecStorage)]
pub struct Interactable(InteractionType);

#[derive(Component, Default, Clone, Debug, Serialize, Deserialize)]
pub struct MaterialStatus {
    // TODO: Deterioration? Damage? HP?
}

#[derive(Component, Clone, Debug)]
#[storage(DenseVecStorage)]
pub struct TileMaterial {
    material: Handle<crate::assets::material::Material>,
    status: MaterialStatus,

}
