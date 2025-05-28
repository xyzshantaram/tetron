use std::{cell::RefCell, collections::HashMap, rc::Rc};

use rune::{ContextError, Module};

use crate::TetronError;

use super::scene::SceneRef;

#[derive(Debug, Default)]
pub struct World {
    scenes: HashMap<String, SceneRef>,
    current_scene: Option<(String, SceneRef)>,
}

#[derive(Clone, Debug, rune::Any, Default)]
#[rune(name = World)]
pub struct WorldRef(Rc<RefCell<World>>);

impl WorldRef {
    pub fn new() -> Self {
        Self::default()
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

    pub fn game_loop(&mut self, dt: f32) -> Result<(), TetronError> {
        self.0.try_borrow_mut()?.game_loop(dt)?;
        Ok(())
    }
}

impl World {
    pub fn module() -> Result<Module, ContextError> {
        let mut module = Module::with_crate_item("tetron", ["game", "world"])?;
        module.ty::<WorldRef>()?;

        Ok(module)
    }

    fn game_loop(&mut self, dt: f32) -> Result<(), TetronError> {
        if let Some((_, scene)) = &mut self.current_scene {
            scene.update(dt)?;
        }

        Ok(())
    }
}
