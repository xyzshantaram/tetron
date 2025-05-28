use super::world::WorldRef;

#[derive(Clone, rune::Any)]
pub struct SystemCtx {
    world: WorldRef,
    dt: f32,
}

impl SystemCtx {
    pub fn new(world: WorldRef, dt: f32) -> Self {
        Self { world, dt }
    }

    #[rune::function]
    fn query(&self) {
        todo!()
    }
}
