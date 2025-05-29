use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::error::TetronError;

use super::behaviours::BehaviourRef;

#[derive(Default, Debug)]
pub struct Entity {
    pub behaviours: HashMap<String, BehaviourRef>,
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

    #[rune::function(keep)]
    pub fn has_tag(&self, tag: &str) -> Result<bool, TetronError> {
        Ok(self.0.try_borrow()?.tags.contains(tag))
    }

    #[rune::function(keep)]
    pub fn attach(&mut self, behaviour: BehaviourRef) -> Result<(), TetronError> {
        let behaviours = &mut self.0.try_borrow_mut()?.behaviours;
        let name = behaviour.name()?;

        #[allow(clippy::map_entry)]
        if behaviours.contains_key(&name) {
            Err(TetronError::Runtime(format!(
                "Cannot insert behaviour {}: already exists",
                name
            )))
        } else {
            behaviours.insert(name, behaviour);
            Ok(())
        }
    }

    #[rune::function]
    pub fn has_behaviour(&self, name: &str) -> Result<bool, TetronError> {
        Ok(self.0.try_borrow()?.behaviours.contains_key(name))
    }

    #[rune::function]
    pub fn behaviour(&self, name: &str) -> Result<Option<BehaviourRef>, TetronError> {
        Ok(self.0.try_borrow()?.behaviours.get(name).cloned())
    }
}
