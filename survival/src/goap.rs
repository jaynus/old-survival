use amethyst::ecs::{
    storage::UnprotectedStorage, world::Index, DenseVecStorage, Entity, World,
};
use hibitset::BitSet;
use indexmap::IndexSet;
use rayon::prelude::*;
use std::collections::HashMap;

use crate::assets::item::Property as ItemProperty;
use crate::actions::Action as ActionEvent;
use crate::components::InteractionType;
use bitflags::*;

bitflags_serial! {
    pub struct TileType: u8 {
        const Water = 1;
        const Magma = 1 << 1;
        const Land  = 1 << 2;
    }
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    strum_macros::EnumString,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ActionCatagory {
    Mining,
    Woodcutting,
    Woodcrafting,
    Woodworking,
    Stonecutting,
    Stonecrafting,
    Hunting,
    Farming,
    Fishing,
    HaulingFood,
    HaulingItems,
    HaulingStone,
    HaulingWood,
    HaulingRefuse,
    Cleaning,
    Doctoring,
    Construction,
}
impl Default for ActionCatagory {
    fn default() -> Self {
        ActionCatagory::Cleaning
    }
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    strum_macros::EnumString,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ConditionType {
    Near(i32),
    Has,
    Me,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    strum_macros::EnumString,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ConditionEquality {
    Is,
    Not,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    strum_macros::EnumString,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ConditionValue {
    Property(ItemProperty),
    Interaction(InteractionType),
    Material { material: String, count: usize },
    Pawn { kind: String, count: usize },
    Location(TileType),
    Tree,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct Condition(ConditionEquality, ConditionType, ConditionValue);

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum ActionSourceType {
    World,
    Region,
    Pawn,
    Attack,
    Ingest,
}
impl Default for ActionSourceType {
    fn default() -> Self {
        ActionSourceType::Pawn
    }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct Action {
    #[serde(skip_serializing, skip_deserializing)]
    id: Index,
    catagory: ActionCatagory,

    event: (ActionEvent, Option<Condition>),

    name: String,
    adjective: String,
    source: ActionSourceType,
    base_time: f32,

    conditions: Vec<Condition>,
    result: Vec<(Condition, bool)>,
}

impl PartialEq for Action {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct PlannerNode {
    pub action_id: Option<Index>,
    pub state: BitSet,
}
impl PlannerNode {
    pub fn new(action_id: Index, state: &BitSet) -> Self {
        Self {
            action_id: Some(action_id),
            state: state.clone(),
        }
    }
}
impl std::hash::Hash for PlannerNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for id in &self.state {
            id.hash(state);
        }
    }
}


pub struct Planner {
    cur_action: Index,
    actions: DenseVecStorage<Action>,
    conditions: IndexSet<Condition>,
    name_table: HashMap<String, Index>,
}

impl Planner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, mut obj: Action) -> Index {
        let action_id = self.cur_action;

        self.cur_action += 1;
        obj.id = action_id;

        self.name_table.insert(obj.name.clone(), action_id);

        // Iterate the conditions and index them
        for condition in &obj.conditions {
            self.conditions.insert(condition.clone());
        }

        unsafe {
            self.actions.insert(action_id, obj);
        }

        action_id
    }

    pub fn lookup(&self, name: &str) -> Option<&Action> {
        if let Some(id) = self.name_table.get(name) {
            return self.get(*id);
        }
        None
    }

    #[allow(mutable_borrow_reservation_conflict)]
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Action> {
        if let Some(id) = self.name_table.get(name) {
            return self.get_mut(*id);
        }
        None
    }

    pub fn get(&self, action: Index) -> Option<&Action> {
        if action > self.cur_action {
            return None;
        }
        unsafe { Some(self.actions.get(action)) }
    }
    pub fn get_mut(&mut self, action: Index) -> Option<&mut Action> {
        if action > self.cur_action {
            return None;
        }
        unsafe { Some(self.actions.get_mut(action)) }
    }

    pub fn get_action_name(&self, action_id: Index) -> Option<&str> {
        if let Some(action) = self.get(action_id) {
            return Some(action.name.as_str());
        }
        None
    }

    pub fn can_occur(&self, action_id: Index, state: &BitSet) -> bool {
        for condition in &self.get(action_id).unwrap().conditions {
            if ! state.contains(self.conditions.get_full(condition).unwrap().0 as u32) {
                log::trace!("\tFailed can: {} because ! {:?}", self.get_action_name(action_id).unwrap(), condition);
                return false;
            }
        }
        true
    }

    pub fn check_condition_live(&self, _condition: &Condition, _entity: Entity, _world: &World) -> bool {
        false
    }

    pub fn check_condition_by_id_live(&self, condition_id: Index, entity: Entity, world: &World) -> bool {
        self.check_condition_live(self.conditions.get_index(condition_id as usize).unwrap(), entity, world)
    }

    pub fn get_condition_set(&self, action_id: Index) -> BitSet {
        let mut set = BitSet::new();
        if let Some(action) = self.get(action_id) {
            action.conditions.iter().for_each(|condition| {
                set.add(self.conditions.get_full(condition).unwrap().0 as u32);
            });
        }
        set
    }

    pub fn get_result_set(&self, action_id: Index) -> BitSet {
        let mut set = BitSet::new();
        if let Some(action) = self.get(action_id) {
            action.result.iter().for_each(|condition| {
                if condition.1 {
                    set.add(self.conditions.get_full(&condition.0).unwrap().0 as u32);
                } else {
                    set.remove(self.conditions.get_full(&condition.0).unwrap().0 as u32);
                }
            });
        }
        set
    }

    pub fn plan_live(&self, available_actions: &BitSet, goal_conditions: &BitSet, entity: Entity, world: &World) -> Option<Vec<Index>> {
        use hibitset::BitSetLike;

        let mut state = BitSet::new();
        available_actions.iter().for_each(|id| {
            if let Some(action) = self.get(id) {
                action.conditions.iter().for_each(|condition| {
                    let condition_id = self.conditions.get_full(condition).unwrap().0 as u32;
                    if self.check_condition_by_id_live(condition_id, entity, world) {
                        state.add(condition_id);
                    }
                });
            }
        });

        self.plan(available_actions, goal_conditions, state)
    }
    pub fn plan(&self, available_actions: &BitSet, goal_conditions: &BitSet, state: BitSet) -> Option<Vec<Index>> {
        use pathfinding::prelude::*;
        use hibitset::BitSetLike;

        let start = PlannerNode {
            action_id: None,
            state,
        };

        if let Some((plan, _)) = astar(
            &start,
            |node| {
                let next = available_actions.par_iter().filter_map(|id| {
                    log::trace!("Checking can={}", id);
                    match self.can_occur(id, &node.state) {
                        true => {
                            log::trace!("\tAdding successor: {}", self.get_action_name(id).unwrap());

                            let mut new_state = node.state.clone();
                            new_state.extend(self.get_result_set(id));

                            Some((PlannerNode {
                                action_id: Some(id),
                                state: new_state.clone(),
                            }, 1))
                        },
                        false => None,
                    }
                }).collect::<Vec<_>>();
                log::trace!("Returning next: {}", next.len());
                next
            },
            |node| {
                let distance = goal_conditions.par_iter().map(|condition| {
                    if ! node.state.contains(condition) {
                        return 1;
                    }
                    0
                }).sum();
                distance
            },
            |node| {
                for condition in goal_conditions {
                    if ! node.state.contains(condition) {
                        return false;
                    }
                }
                log::trace!("matching state");
                return true;
            },
        ) {
            return Some(plan.into_iter().skip(1).map(|ref node| node.action_id.unwrap()).collect());
        }

        None
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self {
            actions: DenseVecStorage::default(),
            name_table: HashMap::new(),
            conditions: IndexSet::new(),
            cur_action: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordered_float::OrderedFloat;

    #[test]
    pub fn goap_test_plan() {
        let _ = env_logger::builder().is_test(true).try_init();

        let actions = gen_test_actions();
        let mut planner = Planner::default();

        let mut goal_action_id: u32 = 0;
        let mut initial_action_id: u32 = 0;
        let mut avialable_actions = BitSet::new();

        for action in actions {
            log::trace!("Adding: {}", action.name);
            let t = if action.name == "Chop Tree" { true } else { false };
            let t2 = if action.name == "Get Axe" { true } else { false };

            let id = planner.insert(action);
            if t { goal_action_id = id; }
            if t2 { initial_action_id = id; }
            avialable_actions.add(id);
        }


        // our  goal action is "chop tree"
        let goal = planner.get_condition_set(goal_action_id);
        let initial_state = planner.get_condition_set(initial_action_id);

        let plan = planner.plan(&avialable_actions, &goal, initial_state);
        log::trace!("Found plan = {:?}", plan);

        let mut res = String::new();
        if let Some(plan) = plan {
            for action in &plan {
                res.push_str(&format!("{} -> ", planner.get_action_name(*action).unwrap()));
            }
        }
        log::trace!("Plan = {}", res);
    }

    pub fn gen_test_actions() -> Vec<Action> {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut actions = Vec::new();

        let mut a = Action::default();
        a.name = "Boil Food".to_string();

        a.conditions.push(Condition(
            ConditionEquality::Is,
            ConditionType::Near(1),
            ConditionValue::Property(ItemProperty::Edible),
        ));
        a.conditions.push(Condition(
            ConditionEquality::Is,
            ConditionType::Near(1),
            ConditionValue::Property(ItemProperty::Cooking(OrderedFloat(5.0))),
        ));
        a.conditions.push(Condition(
            ConditionEquality::Is,
            ConditionType::Near(1),
            ConditionValue::Material {
                material: "Water".to_string(),
                count: 1,
            },
        ));
        actions.push(a);

        let mut a = Action::default();
        a.name = "Get Axe".to_string();
        a.event = (crate::actions::Action::Pickup,
            Some(Condition(
                ConditionEquality::Is,
                ConditionType::Near(1),
                ConditionValue::Property(ItemProperty::Chopping(OrderedFloat(1.0)))
            )));
        a.conditions.push(Condition(
            ConditionEquality::Is,
            ConditionType::Near(1),
            ConditionValue::Property(ItemProperty::Chopping(OrderedFloat(1.0))),
        ));

        a.result.push(
            (Condition(
                ConditionEquality::Is,
                ConditionType::Has,
                ConditionValue::Property(ItemProperty::Chopping(OrderedFloat(1.0)))),
             true)
        );
        actions.push(a);

        let mut a = Action::default();
        a.name = "Chop Tree".to_string();
        a.conditions.push(Condition(
            ConditionEquality::Is,
            ConditionType::Has,
            ConditionValue::Property(ItemProperty::Chopping(OrderedFloat(1.0))),
        ));
        a.conditions.push(Condition(
            ConditionEquality::Is,
            ConditionType::Near(1),
            ConditionValue::Tree,
        ));
        actions.push(a);

        let mut a = Action::default();
        a.name = "Move To Tree".to_string();
        a.result.push(( Condition(
            ConditionEquality::Is,
            ConditionType::Near(1),
            ConditionValue::Tree,
        ), true ));
        actions.push(a);


        actions
    }

    #[test]
    pub fn goap_condition_test_definition() {
        let _ = env_logger::builder().is_test(true).try_init();

        let actions = gen_test_actions();

        let serialized = ron::ser::to_string_pretty(
            &actions,
            ron::ser::PrettyConfig {
                depth_limit: 4,
                separate_tuple_members: false,
                enumerate_arrays: false,
                ..ron::ser::PrettyConfig::default()
            },
        )
        .unwrap();
        log::trace!("{}", serialized);

        let _d: Vec<Action> = ron::de::from_str(serialized.as_str()).unwrap();
    }
}
