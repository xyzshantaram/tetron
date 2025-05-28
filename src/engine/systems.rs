use super::world::WorldRef;

#[derive(Clone, rune::Any)]
pub struct Ctx {
    world: WorldRef,
    dt: f32,
}

impl Ctx {
    pub fn new(world: WorldRef, dt: f32) -> Self {
        Self { world, dt }
    }

    #[rune::function]
    fn query(&self) {
        todo!()
    }
}
