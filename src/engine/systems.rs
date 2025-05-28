use crate::TetronError;

use super::{entity::EntityRef, world::WorldRef};

#[derive(Clone, rune::Any)]
pub struct Ctx {
    #[rune(get)]
    world: WorldRef,
    #[rune(get)]
    dt: f32,
}

impl Ctx {
    pub fn new(world: WorldRef, dt: f32) -> Self {
        Self { world, dt }
    }

    #[rune::function]
    fn query(&self) -> Result<Vec<EntityRef>, TetronError> {
        let mut result: Vec<EntityRef> = Vec::new();
        if let Some((_, scene)) = self.world.current_scene()? {
            for entity in scene.entities()? {}
        }
        Ok(result)
    }
}
