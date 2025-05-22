use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use rhai::Module;

use super::behaviours::Behaviour;

#[derive(Default, Debug)]
pub struct Entity {
    behaviours: HashMap<String, Box<dyn Behaviour>>,
    tags: HashSet<String>,
}

#[derive(Clone, Debug, Default)]
pub struct EntityRef(Rc<RefCell<Entity>>);

impl Entity {
    fn create() -> EntityRef {
        EntityRef(Rc::new(RefCell::new(Self::default())))
    }

    pub fn register(module: &mut Module) {
        module.set_custom_type::<EntityRef>("Entity");
        module.set_sub_module("Entity", {
            let mut sub = Module::new();
            sub.set_native_fn("create", || Ok(Self::create()));
            sub
        });
    }
}
