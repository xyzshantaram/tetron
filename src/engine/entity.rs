use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use rune::{ContextError, Module};

use super::behaviours::Behaviour;

#[derive(Default, Debug)]
pub struct Entity {
    behaviours: HashMap<String, Box<dyn Behaviour>>,
    tags: HashSet<String>,
}

#[derive(Clone, Debug, Default, rune::Any)]
#[rune(name = Entity)]
pub struct EntityRef(Rc<RefCell<Entity>>);

impl Entity {
    pub fn module() -> Result<Module, ContextError> {
        let mut module = Module::with_crate_item("tetron", ["game"])?;
        module.ty::<EntityRef>()?;
        Ok(module)
    }
}

impl EntityRef {
    pub fn new() -> Self {
        EntityRef(Rc::new(RefCell::new(Entity::default())))
    }
}
