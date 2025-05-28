use rune::{ContextError, Module, ToValue, runtime::Function};

use crate::TetronError;

use super::{entity::EntityRef, systems::Ctx, world::WorldRef};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Scene {
    world: WorldRef,
    entities: Vec<EntityRef>,
    systems: HashMap<String, Function>,
}

impl Scene {
    pub fn new(world: WorldRef) -> Self {
        Self {
            world,
            entities: Vec::new(),
            systems: HashMap::new(),
        }
    }
    pub fn module() -> Result<Module, ContextError> {
        let mut module = Module::with_crate_item("tetron", ["game"])?;
        module.ty::<SceneRef>()?;
        Ok(module)
    }
}

#[derive(Clone, Debug, rune::Any)]
#[rune(name = Scene)]
pub struct SceneRef(Rc<RefCell<Scene>>);

impl SceneRef {
    #[rune::function(path = Self::new)]
    fn new(world: WorldRef) -> Self {
        SceneRef(Rc::new(RefCell::new(Scene::new(world))))
    }

    #[rune::function(keep)]
    fn spawn(&mut self) -> Result<EntityRef, TetronError> {
        let entity = EntityRef::new();
        self.0.try_borrow_mut()?.entities.push(entity.clone());
        Ok(entity)
    }

    #[rune::function(instance)]
    fn system(&mut self, name: &str, f: Function) -> Result<(), TetronError> {
        self.0.try_borrow_mut()?.systems.insert(name.to_owned(), f);
        Ok(())
    }

    pub fn update(&mut self, dt: f32) -> Result<(), TetronError> {
        let scene = self.0.try_borrow_mut()?;
        let ctx = Ctx::new(scene.world.clone(), dt);
        for system in self.0.try_borrow_mut()?.systems.values() {
            system
                .call::<Result<(), TetronError>>((ctx.clone().to_value()?,))
                .expect("Unrecoverable error updating scene")?;
        }

        Ok(())
    }
}
