use rune::runtime::Object;

use crate::error::TetronError;

use super::{entity::EntityRef, world::WorldRef};

#[derive(Clone, rune::Any)]
pub struct Ctx {
    #[rune(get)]
    world: WorldRef,
    #[rune(get)]
    dt: f64,
}

impl Ctx {
    pub fn new(world: WorldRef, dt: f64) -> Self {
        Self { world, dt }
    }

    #[rune::function]
    fn query(&self, query: Object) -> Result<Vec<EntityRef>, TetronError> {
        let mut result: Vec<EntityRef> = Vec::new();

        if let Some((_, scene)) = self.world.current_scene()? {
            for entity in scene.entities()? {}
        }
        Ok(result)
    }
}
