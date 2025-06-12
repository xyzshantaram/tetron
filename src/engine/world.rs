use super::{behaviours::BehaviourFactory, scene::SceneRef};
use crate::{error::TetronError, log_and_die, utils::typed_value::schema::Schema};
use rune::{alloc::clone::TryClone, runtime::Object};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

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
        module.function_meta(WorldRef::load_scene)?;
        Ok(())
    }
}

impl WorldRef {
    pub fn new() -> Self {
        Self::default()
    }

    #[rune::function(instance)]
    fn define_behaviour(&mut self, name: &str, schema: Schema) -> BehaviourFactoryRef {
        let registry = &mut self
            .0
            .try_borrow_mut()
            .expect("Engine bug: world lock poisoned")
            .behaviour_registry;
        if name.starts_with("tetron:") {
            log_and_die!(
                1,
                "Engine bug: Cannot define behaviour {name}: Behaviour names cannot start with 'tetron:'"
            );
        } else if registry.contains_key(name) {
            log_and_die!(
                1,
                "Engine bug: Cannot define behaviour {name}: a behaviour with the same name already exists"
            );
        } else {
            let factory = BehaviourFactoryRef(Arc::new(BehaviourFactory::new(name, schema, false)));
            registry.insert(name.into(), factory.clone());
            factory
        }
    }

    #[rune::function(instance)]
    fn behaviour(&self, name: &str) -> Option<BehaviourFactoryRef> {
        self.0.borrow().behaviour_registry.get(name).cloned()
    }

    #[rune::function(instance)]
    fn scene(&self, name: &str, config: Object) -> SceneRef {
        let mut world = self.0.borrow_mut();
        if world.scenes.contains_key(name) {
            log_and_die!(
                1,
                "Engine bug: Could not create scene {name} - a scene with that name already exists"
            );
        }

        let scene = SceneRef::new(self.clone(), config);
        world.scenes.insert(name.into(), scene.clone());

        scene
    }

    #[rune::function(instance)]
    fn load_scene(&self, name: &str) {
        if let Some(scene) = self.0.borrow().scenes.get(name).cloned() {
            self.0.borrow_mut().current_scene = Some((name.to_owned(), scene));
        }
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
