use amethyst::{
    core::math::{Vector3, Vector4},
    ecs::{Join, ReadExpect, SystemData, World},
};
use std::collections::HashMap;

use crate::components::{Obstruction, ZTransition};
use crate::tiles::{ReadTiles, Tiles};

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
struct PathCache {
    z_transitions: Vec<(Vector3<u32>, f32)>,
    z_transitions_index: HashMap<u32, Vec<u32>>,
}
impl PathCache {
    pub fn insert_transition(&mut self, coord: Vector3<u32>, speed: f32) {
        let index = self.z_transitions.len() as u32;
        let z = coord.z;
        self.z_transitions.push((coord, speed));

        let z_list = {
            match self.z_transitions_index.get_mut(&z) {
                Some(list) => list,
                None => {
                    self.z_transitions_index.insert(z, Vec::new());
                    self.z_transitions_index.get_mut(&z).unwrap()
                }
            }
        };
        z_list.push(index);
    }

    pub fn clear(&mut self) {
        self.z_transitions.clear();
        self.z_transitions_index.clear();
    }

    pub fn rebuild(&mut self, world: &World, _region: Vector4<u32>) {
        self.clear();

        // Find all ZTransition's, index them + their obstruction value if any
        let tiles: ReadExpect<Tiles> = SystemData::fetch(&world.res);
        let z_transitions: ReadTiles<ZTransition> = SystemData::fetch(&world.res);
        let _obstructions: ReadTiles<Obstruction> = SystemData::fetch(&world.res);

        for (tile_id, _) in (&tiles, &z_transitions).join() {
            let _v = tile_id.vector(tiles.dimensions());
        }
    }
}

pub enum PathfindingType {
    Astar,
}

#[derive(Default, Copy, Clone)]
pub struct Pathfinding;

impl Pathfinding {
    pub fn shortest_path(
        &self,
        obs: ReadTiles<Obstruction>,
        tiles: &Tiles,
        start: &Vector3<u32>,
        goal: &Vector3<u32>,
    ) {
        use ordered_float::NotNan;
        use pathfinding::prelude::*;

        let default_weight = NotNan::new(1.0).unwrap();

        let _result = astar(
            start,
            |point| {
                vec![
                    (*point + Vector3::new(1, 0, 0), default_weight),
                    (*point + Vector3::new(0, 1, 0), default_weight),
                    (*point - Vector3::new(1, 0, 0), default_weight),
                    (*point - Vector3::new(0, 1, 0), default_weight),
                    (*point + Vector3::new(1, 1, 0), default_weight),
                    (*point - Vector3::new(1, 1, 0), default_weight),
                    (
                        *point + Vector3::new(1, 0, 0) - Vector3::new(0, 1, 0),
                        default_weight,
                    ),
                    (
                        *point + Vector3::new(0, 1, 0) - Vector3::new(1, 0, 0),
                        default_weight,
                    ),
                ]
                .into_iter()
                .filter_map(|v| match obs.get(tiles.id_from_vector(v.0)) {
                    Some(o) => match o {
                        Obstruction::Impassable => None,
                        Obstruction::Slow(rate) => Some((v.0, NotNan::new(*rate).unwrap())),
                    },
                    None => Some(v),
                })
                .collect::<Vec<_>>()
            },
            |point| {
                // heuristic
                NotNan::new((absdiff(point.x, goal.x) + absdiff(point.y, goal.y)) as f32).unwrap()
            },
            |point| *point == *goal,
        );
    }
}

type DijstraMap = HashMap<Vector3<u32>, f32>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DijkstraMapType {
    Movement,

}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DijkstraCollection {
    maps: HashMap<DijkstraMapType, DijstraMap>,
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn pathcache_test() {
        println!("Ran");
    }
}
