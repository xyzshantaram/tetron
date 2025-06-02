use super::{behaviours::BehaviourFactory, scene::SceneRef};
use crate::{error::TetronError, utils::RuneString};
use rune::{alloc::clone::TryClone, runtime::Object};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

#[derive(rune::Any, Clone, Debug)]
// ok to ignore warning, used in Rune
pub struct BehaviourFactoryRef(#[allow(dead_code)] Arc<BehaviourFactory>);

#[derive(Debug, Default)]
pub struct World {
    #[allow(dead_code)] // used in rune
    scenes: HashMap<String, SceneRef>,
    current_scene: Option<(String, SceneRef)>,
    #[allow(dead_code)] // used in rune
    behaviour_registry: HashMap<String, BehaviourFactoryRef>,
}

#[derive(Clone, Debug, rune::Any, Default)]
#[rune(name = World)]
pub struct WorldRef(Rc<RefCell<World>>);

impl TryClone for WorldRef {
    fn try_clone(&self) -> Result<Self, rune::alloc::Error> {
        Ok(self.clone())
    }
}

use crate::utils::Registrable;
use rune::{ContextError, Module};

impl Registrable for WorldRef {
    fn register(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<WorldRef>()?;
        module.function_meta(WorldRef::define_behaviour)?;
        module.function_meta(WorldRef::behaviour)?;
        module.function_meta(WorldRef::scene)?;
        module.function_meta(WorldRef::insert)?;
        module.function_meta(WorldRef::load_scene)?;
        Ok(())
    }
}

impl WorldRef {
    pub fn new() -> Self {
        Self::default()
    }

    #[rune::function(instance)]
    fn define_behaviour(
        &mut self,
        name: &str,
        keys: Vec<RuneString>,
    ) -> Result<BehaviourFactoryRef, TetronError> {
        let registry = &mut self.0.try_borrow_mut()?.behaviour_registry;
        if name.starts_with("tetron:") {
            Err(TetronError::Runtime(format!(
                "Cannot define behaviour {name}: Behaviour names cannot start with 'tetron:'"
            )))
        } else if registry.contains_key(name) {
            Err(TetronError::Runtime(format!(
                "Cannot define behaviour {name}: a behaviour with the same name already exists"
            )))
        } else {
            let factory = BehaviourFactoryRef(Arc::new(BehaviourFactory::new(
                name,
                HashSet::from_iter(keys.iter().map(|v| v.to_string())),
                false,
            )));
            registry.insert(name.into(), factory.clone());
            Ok(factory)
        }
    }

    #[rune::function(instance)]
    fn behaviour(&self, name: &str) -> Result<Option<BehaviourFactoryRef>, TetronError> {
        Ok(self.0.try_borrow()?.behaviour_registry.get(name).cloned())
    }

    #[rune::function(instance)]
    fn scene(&self, config: Object) -> SceneRef {
        SceneRef::new(self.clone(), config)
    }

    #[rune::function(instance)]
    fn insert(&self, name: &str, scene: SceneRef) -> Result<(), TetronError> {
        self.0
            .try_borrow_mut()
            .map_err(|e| TetronError::Runtime(format!("Could not insert scene \"{name}\": {e}")))?
            .scenes
            .insert(name.into(), scene);

        Ok(())
    }

    #[rune::function(instance)]
    fn load_scene(&self, name: &str) -> Result<(), TetronError> {
        let scene = self
            .0
            .try_borrow()
            .map_err(|e| TetronError::Runtime(format!("Could not load scene \"{name}\": {e}")))?
            .scenes
            .get(name)
            .cloned()
            .ok_or(TetronError::Runtime("Could not clone option".into()))?;

        self.0
            .try_borrow_mut()
            .map_err(|e| TetronError::Runtime(format!("Could not load scene \"{name}\": {e}")))?
            .current_scene = Some((name.to_owned(), scene));

        Ok(())
    }

    pub fn game_loop(&mut self, dt: f64) -> Result<(), TetronError> {
        self.0.try_borrow_mut()?.game_loop(dt)?;
        Ok(())
    }

    pub fn current_scene(&self) -> Result<Option<(String, SceneRef)>, TetronError> {
        Ok(self.0.try_borrow()?.current_scene.clone())
    }
}

impl World {
    fn game_loop(&mut self, dt: f64) -> Result<(), TetronError> {
        if let Some((_, scene)) = &mut self.current_scene {
            scene.update(dt)?;
        }

        Ok(())
    }
}
