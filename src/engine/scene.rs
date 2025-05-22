use super::entity::EntityRef;
use std::{cell::RefCell, rc::Rc};

#[derive(Default, Debug)]
pub struct Scene {
    entities: Vec<EntityRef>,
}

#[derive(Default, Clone, Debug)]
pub struct SceneRef(Rc<RefCell<Scene>>);
