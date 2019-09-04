use amethyst::{
    ecs::{Component, DenseVecStorage, world::Index, },
};

use specs_derive::Component;
use crossbeam::queue::SegQueue;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::BinaryHeap;

use crate::goap::ActionCatagory;

use crate::pathfinding::DijkstraCollection;

#[derive(Component, Default, Debug)]
#[storage(DenseVecStorage)]
pub struct Pawn {
    pub requested_action: Index,
    pub labor_priorities: HashMap<ActionCatagory, f32>,
}

#[derive(Component, Default, Debug)]
#[storage(DenseVecStorage)]
pub struct AI {
    pub action_set: Index,
    pub action_queue: SegQueue<Index>,
    pub dijkstra_maps: DijkstraCollection,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Thought {
    duration: f32,
    value: f32,
    impacts: f32,
}
impl PartialEq for Thought {
    fn eq(&self, other: &Self) -> bool {
        self.duration.eq(&other.duration)
    }
}
impl PartialOrd for Thought {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.duration.partial_cmp(&other.duration)
    }
}

pub struct Personality {

    thoughts: BinaryHeap<Thought>,
}