use crate::mapgen::GeneratorSettings;
use crate::tiles::{TileId, Tiles};
use amethyst::core::math::{Vector3, Vector4};
use specs_static::Id;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Tile {
    pub sprite_number: u32,
    pub sprite_sheet_number: u32,
    pub filled: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldMap {
    pub heightmap: Vec<u8>,
    pub moisture: Vec<u8>,
    pub seed: String,
    pub settings: GeneratorSettings,

    inner: Tiles,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Region {
    pub id: u32,
    pub tiles: Vec<Tile>,
}

const z_depth: usize = 20;

impl WorldMap {
    pub fn new(settings: &GeneratorSettings) -> Self {
        Self {
            settings: settings.clone(),
            heightmap: Vec::new(),
            seed: String::new(),
            moisture: Vec::new(),
            inner: Tiles::new(
                settings.world_pixels as u32,
                settings.world_pixels as u32,
                z_depth as u32,
            ),
        }
    }

    pub fn coord_to_region_id(&self, coord: Vector3<u32>) -> TileId {
        let region_coord = amethyst::core::math::convert::<Vector3<u32>, Vector3<f32>>(coord)
            / self.settings.region_pixels as f32;
        let absolute = Vector3::<u32>::new(
            region_coord.x as u32,
            region_coord.y as u32,
            region_coord.z as u32,
        );
        // Now round the coordinate to a region id
        self.inner.id_from_vector(absolute)
    }

    pub fn generate_chunk(&self, id: u32) -> Region {
        use rbf_interp::{DistanceFunction, PtValue, Rbf};

        let _seed = self.region_seed(id);

        let mut region = Region::default();
        region.tiles.resize(
            self.settings.region_size * self.settings.region_size * z_depth,
            Tile::default(),
        );

        // Collect the points in the region
        let region_tiles = Tiles::new(
            self.settings.region_size as u32,
            self.settings.region_size as u32,
            z_depth as u32,
        );
        let world_tiles = Tiles::new(
            self.settings.world_pixels as u32,
            self.settings.world_pixels as u32,
            z_depth as u32,
        );

        // load the heightmap into an image
        let imgbuf = image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_raw(
            self.settings.world_pixels as u32,
            self.settings.world_pixels as u32,
            self.heightmap.clone(),
        )
        .unwrap();
        slog::slog_trace!(slog_scope::logger(), "Dimensions={:?}", imgbuf.dimensions());

        let region_range = Vector4::new(0, 0, 10, 10);

        let mut points = Vec::new();
        world_tiles.iter_region(region_range, 1).for_each(|id| {
            let coord = id.vector(region_tiles.dimensions());
            //slog::slog_trace!(slog_scope::logger(), "Collected coord: {:?}", coord);
            let height = imgbuf.get_pixel(coord.x as u32, coord.y as u32);

            points.push(PtValue::new(coord.x, coord.y, (height[0] as f32) / 255.));
        });

        let rbf = Rbf::new(&points, DistanceFunction::Linear, None);
        region_tiles.iter_all().for_each(|id| {
            let coord = id.vector(region_tiles.dimensions());
            let z = rbf.interp_point((coord.x, coord.y));
            if (coord.z as f32) / z_depth as f32 > z {
                if let Some(tile) = region.tiles.get_mut(id.id() as usize) {
                    *tile = Tile {
                        sprite_number: 1,
                        sprite_sheet_number: 1,
                        filled: true,
                    };
                }
            } else {
                if let Some(tile) = region.tiles.get_mut(id.id() as usize) {
                    *tile = Tile {
                        sprite_number: 1,
                        sprite_sheet_number: 1,
                        filled: false,
                    };
                }
            }
        });

        Region::default()
    }

    pub fn save_chunk() {

    }

    pub fn load_chunk() {

    }

    fn region_seed(&self, id: u32) -> Vec<u8> {
        crate::mapgen::seed_from_string(&format!("{}{}", id, self.seed))
    }
}
