use amethyst::{
    core::math::{Point3, Vector2, Vector3, Vector4},
    ecs::{BitSet, Component, DenseVecStorage, Entity, Join, ParJoin, Read, Write},
};

use specs_derive::Component;
use specs_static::{Id, Storage};
use std::collections::HashSet;

#[derive(Component, Clone, Debug, Default)]
#[storage(DenseVecStorage)]
pub struct TileEntities(pub HashSet<Entity>);

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct TileId(u32);

impl TileId {
    #[inline]
    // (z * xMax * yMax) + (y * xMax) + x;
    pub fn coords(self, dimensions: Vector3<u32>) -> (f32, f32, f32) {
        let z = self.0 / (dimensions.x * dimensions.y);
        let idx = self.0 - (z * dimensions.x * dimensions.y);

        (
            (idx % dimensions.y) as f32,
            (idx / dimensions.y) as f32,
            z as f32,
        )
    }

    #[inline]
    pub fn vector(&self, dimensions: Vector3<u32>) -> Vector3<f32> {
        let coords = self.coords(dimensions);
        Vector3::new(coords.0, coords.1, coords.2)
    }
}

impl Id for TileId {
    fn from_u32(value: u32) -> Self {
        Self(value)
    }

    fn id(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Tiles {
    dimensions: Vector3<u32>,
}

impl Tiles {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self {
            dimensions: Vector3::new(x, y, z),
        }
    }

    pub fn id(self, x: u32, y: u32, z: u32) -> TileId {
        TileId((z * self.dimensions.x * self.dimensions.y) + (y * self.dimensions.x) + x)
    }

    pub fn id_from_vector(&self, vector: Vector3<u32>) -> TileId {
        //(z * xMax * yMax) + (y * xMax) + x;
        TileId(
            (vector.z * self.dimensions.x * self.dimensions.y)
                + (vector.y * self.dimensions.x)
                + vector.x,
        )
    }

    pub fn id_from_point(&self, vector: Point3<u32>) -> TileId {
        //(z * xMax * yMax) + (y * xMax) + x;
        TileId(
            (vector.z * self.dimensions.x * self.dimensions.y)
                + (vector.y * self.dimensions.x)
                + vector.x,
        )
    }

    pub fn world_to_tile(
        self,
        vector: &Vector3<amethyst::core::Float>,
        game_settings: &crate::settings::Config,
    ) -> Vector3<u32> {
        Vector3::new(
            (vector.x / 20. / game_settings.graphics.scale) as u32,
            (vector.y / 20. / game_settings.graphics.scale).abs() as u32,
            (vector.z / 20. / game_settings.graphics.scale).abs() as u32,
        )
    }

    pub fn world_to_id(
        self,
        vector: &Vector3<f32>,
        game_settings: &crate::settings::Config,
    ) -> TileId {
        self.id_from_vector(Vector3::new(
            (vector.x / 20. / game_settings.graphics.scale) as u32,
            (vector.y / 20. / game_settings.graphics.scale).abs() as u32,
            (vector.z / 20. / game_settings.graphics.scale).abs() as u32,
        ))
    }

    pub fn iter_all(self) -> impl Iterator<Item = TileId> {
        (0..self.dimensions.x * self.dimensions.y - 1).map(TileId)
    }

    pub fn iter_region(self, region: Vector4<u32>, z_level: u32) -> impl Iterator<Item = TileId> {
        RegionIter::new(self, region, z_level)
    }

    pub fn dimensions(self) -> Vector3<u32> {
        self.dimensions
    }
}

impl<'a> Join for &'a Tiles {
    type Type = TileId;
    type Value = Self;
    type Mask = BitSet;

    unsafe fn open(self) -> (Self::Mask, Self) {
        (BitSet::new(), self)
    }

    unsafe fn get(_v: &mut &'a Tiles, idx: u32) -> TileId {
        TileId::from_u32(idx)
    }
}

unsafe impl<'a> ParJoin for &'a Tiles {}

pub struct RegionIter {
    region: Vector4<u32>,
    tiles: Tiles,
    cur: Vector2<u32>,
    z_level: u32,
    stride: u32,
}
impl RegionIter {
    pub fn new(tiles: Tiles, region: Vector4<u32>, z_level: u32) -> Self {
        Self {
            stride: 1,
            region,
            tiles,
            cur: Vector2::new(region.x, region.y),
            z_level,
        }
    }
}
impl Iterator for RegionIter {
    type Item = TileId;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur.x += self.stride;
        if self.cur.x > self.region.z {
            self.cur.x = self.region.x;
            self.cur.y += self.stride;
        }

        if self.cur.y > self.region.w {
            return None;
        }

        Some(
            self.tiles
                .id_from_vector(Vector3::new(self.cur.x, self.cur.y, self.z_level)),
        )
    }
}

#[allow(clippy::module_name_repetitions)]
pub type ReadTiles<'a, C> = Read<'a, Storage<C, <C as Component>::Storage, TileId>>;
#[allow(clippy::module_name_repetitions)]
pub type WriteTiles<'a, C> = Write<'a, Storage<C, <C as Component>::Storage, TileId>>;
