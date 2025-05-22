use super::{entity::EntityRef, world::WorldRef};

pub struct BehaviourContext {
    pub entity: EntityRef,
    pub world: WorldRef,
    pub dt: f64,
}

pub trait Behaviour: std::fmt::Debug {}
