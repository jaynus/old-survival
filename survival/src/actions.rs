use amethyst::core::math::Vector3;
use amethyst::ecs::Entity;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Deserialize,
    serde::Serialize,
    strum_macros::EnumString,
    strum_macros::Display,
)]
pub enum Direction {
    N,
    NW,
    NE,
    S,
    SW,
    SE,
    E,
    W,
}
impl Default for Direction {
    fn default() -> Self {
        Direction::N
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum_macros::Display,)]
pub enum Target {
    Entity(Entity),
    Location(Vector3<f32>),
    Direction(Direction),
    Under,
    SelfTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::EnumString, strum_macros::Display, serde::Serialize, serde::Deserialize)]
pub enum Action {
    Move,
    MoveTo,
    Pickup,
    Wait,
}
impl Default for Action {
    fn default() -> Self {
        Action::Wait
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TryAction {
    action: Action,
    target: Target,
}

#[derive(
    Clone,
    Hash,
    Eq,
    PartialEq,
    Copy,
    Debug,
    serde::Deserialize,
    serde::Serialize,
    strum_macros::EnumString,
    strum_macros::Display,
)]
pub enum PlayerInputAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    ZoomIn,
    ZoomOut,
}
