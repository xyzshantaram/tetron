use super::{entity::EntityRef, systems::Ctx, world::WorldRef};
use crate::{error::TetronError, utils::Registrable};
use rune::{
    ContextError, Module, ToValue,
    runtime::{Function, Object},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Scene {
    world: WorldRef,
    entities: Vec<EntityRef>,
    systems: HashMap<String, Function>,
    config: Object,
}

impl Scene {
    pub fn new(world: WorldRef, config: Object) -> Self {
        Self {
            world,
            entities: Vec::new(),
            systems: HashMap::new(),
            config,
        }
    }
}

#[derive(Clone, Debug, rune::Any)]
#[rune(name = Scene)]
pub struct SceneRef(Rc<RefCell<Scene>>);

impl Registrable for SceneRef {
    fn register(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<SceneRef>()?;
        module.function_meta(SceneRef::spawn__meta)?;
        module.function_meta(SceneRef::system)?;
        Ok(())
    }
}

impl SceneRef {
    pub fn new(world: WorldRef, config: Object) -> Self {
        SceneRef(Rc::new(RefCell::new(Scene::new(world, config))))
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

    pub fn update(&mut self, dt: f64) -> Result<(), TetronError> {
        let scene = self.0.try_borrow_mut()?;
        let ctx = Ctx::new(scene.world.clone(), dt);
        for system in self.0.try_borrow_mut()?.systems.values() {
            system
                .call::<Result<(), TetronError>>((ctx.clone().to_value()?,))
                .expect("Unrecoverable error updating scene")?;
        }

        Ok(())
    }

    pub fn entities(&self) -> Result<Vec<EntityRef>, TetronError> {
        Ok(self.0.try_borrow()?.entities.clone())
    }
}
