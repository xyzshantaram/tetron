use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::error::TetronError;

use super::behaviours::Behaviour;

#[derive(Default, Debug)]
pub struct Entity {
    pub behaviours: HashMap<String, Behaviour>,
    pub tags: HashSet<String>,
}

#[derive(Clone, Debug, Default, rune::Any)]
#[rune(name = Entity)]
pub struct EntityRef(Rc<RefCell<Entity>>);

impl EntityRef {
    pub fn new() -> Self {
        EntityRef(Rc::new(RefCell::new(Entity::default())))
    }

    #[rune::function]
    pub fn tag(&mut self, tag: &str) -> Result<(), TetronError> {
        self.0.try_borrow_mut()?.tags.insert(tag.into());
        Ok(())
    }

    #[rune::function]
    pub fn has_tag(&self, tag: &str) -> Result<bool, TetronError> {
        Ok(self.0.try_borrow()?.tags.contains(tag))
    }

    #[rune::function]
    pub fn attach(&mut self, behaviour: Behaviour) -> Result<(), TetronError> {
        let behaviours = &mut self.0.try_borrow_mut()?.behaviours;
        if behaviours.contains_key(&behaviour.name) {
            Err(TetronError::Runtime(format!(
                "Cannot insert behaviour {}: already exists",
                behaviour.name
            )))
        } else {
            behaviours.insert(behaviour.name.clone(), behaviour);
            Ok(())
        }
    }
}
