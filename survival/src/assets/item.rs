use bitflags::*;

use amethyst::{
    assets::{Asset, Handle},
    ecs::VecStorage,
};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct ItemFlag: u64 {
        const Container = 1;
        const Tool      = 1 << 1;
    }
}
impl Default for ItemFlag {
    fn default() -> Self {
        Self { bits: 0 }
    }
}

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct ContainerCanHold: u8 {
        const Liquid = 1 ;
        const Solid  = 1 << 1;
    }
}
impl Default for ContainerCanHold {
    fn default() -> Self {
        Self { bits: 0 }
    }
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Deserialize,
    Serialize,
    strum_macros::EnumString,
    strum_macros::Display,
)]
pub enum Property {
    Container { can_hold: ContainerCanHold },
    Chopping(OrderedFloat<f32>),
    Cutting(OrderedFloat<f32>),
    Hitting(OrderedFloat<f32>),
    Hammering(OrderedFloat<f32>),
    Cooking(OrderedFloat<f32>),
    Boiling(OrderedFloat<f32>),
    Edible,
    None,
}
impl Default for Property {
    fn default() -> Self {
        Property::None
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, strum_macros::EnumString, strum_macros::Display,)]
pub enum Catagory {
    Furniture,
    Weapon,
    Armor,
    Tool,
    Stone,
    Wood,
    Other
}
impl Default for Catagory {
    fn default() -> Self {
        Catagory::Other
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Details {
    // general information
    pub size: (f32, f32, f32),
    pub weight: f32,
    pub flags: ItemFlag,

    // UI information
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub catagory: Catagory,
    pub sprite_sheet_number: usize,
    pub sprite_number: usize,

    pub properties: Vec<Property>,
    pub interactions: crate::components::InteractionType,
}
impl PartialEq for Details {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Asset for Details {
    const NAME: &'static str = "survival::Item";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Storage {
    tag: u32,
    items: HashMap<String, Arc<Details>>,
}

#[test]
pub fn write_test_collection() {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::Path;

    let mut collection = Storage::default();

    collection.items.insert(
        "test_collection_item_1".to_string(),
        Arc::new(Details {
            name: "Test Collection Item 1".to_owned(),
            short_description: "Test Collection Item 1".to_owned(),
            long_description: "Test Collection Item 1".to_owned(),
            ..Default::default()
        }),
    );
    collection.items.insert(
        "test_collection_item_2".to_string(),
        Arc::new(Details {
            name: "Test Collection Item 2".to_owned(),
            short_description: "Test Collection Item 2".to_owned(),
            long_description: "Test Collection Item 2".to_owned(),
            flags: ItemFlag::Container,
            properties: vec![Property::Container {
                can_hold: ContainerCanHold::Solid,
            }],
            ..Default::default()
        }),
    );

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(Path::new("resources/data/test.items.ron"))
        .unwrap();
    let serialized = ron::ser::to_string_pretty(
        &collection,
        ron::ser::PrettyConfig {
            depth_limit: 4,
            separate_tuple_members: false,
            enumerate_arrays: false,
            ..ron::ser::PrettyConfig::default()
        },
    )
    .unwrap();
    file.write_all(serialized.as_bytes()).unwrap();
}
