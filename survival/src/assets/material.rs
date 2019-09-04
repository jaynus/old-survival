use std::collections::HashMap;

#[derive(
    strum_macros::EnumString,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum MaterialState {
    Solid,
    Liquid,
    Gas,
    Powder,
    Paste,
    Pressed,
    AllSolid,
    SolidPowder,
    SolidPaste,
    SolidPressed,
}

#[derive(
    strum_macros::EnumString,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum MaterialCatagory {
    IgneousRock,
    MetamorphicRock,
    SedimentaryRock,
    Other,
}
impl Default for MaterialCatagory {
    fn default() -> Self {
        MaterialCatagory::Other
    }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct Material {
    name: String,
    catagory: MaterialCatagory,

    state_names: HashMap<MaterialState, String>,
    state_sprites: HashMap<MaterialState, (u32, u32)>, // sheet, sprite

    permeable: f64,

    impact_yeild: f64,
    impact_fracture: f64,
    impact_elasticity: f64,

    compressive_yeild: f64,
    compressive_fracture: f64,
    compressive_elasticity: f64,

    tensile_yeild: f64,
    tensile_fracture: f64,
    tensile_elasticity: f64,

    torsion_yeild: f64,
    torsion_fracture: f64,
    torsion_elasticity: f64,

    bend_yeild: f64,
    bend_fracture: f64,
    bend_elasticity: f64,

    max_edge: Option<f64>,

    heat_accumulator: Option<f64>,
    melt_point: Option<f64>,
    boil_point: Option<f64>,
    ignite_point: Option<f64>,

    densities: HashMap<MaterialState, f64>,
}


#[derive(Clone, Default, Debug, serde::Deserialize, serde::Serialize)]
pub struct Layer {
    name: String,
    material: String,
    depth: f64,
}
impl Layer {
    pub fn new(name: &str, material: &str, depth: f64) -> Self {
        Self {
            name: name.to_string(),
            material: material.to_string(),
            depth
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_serialize() {
        let mat = Material::default();

        let serialized = ron::ser::to_string_pretty(
            &mat,
            ron::ser::PrettyConfig {
                depth_limit: 4,
                separate_tuple_members: false,
                enumerate_arrays: false,
                ..ron::ser::PrettyConfig::default()
            },
        )
        .unwrap();
        println!("{}", serialized);
    }
}
