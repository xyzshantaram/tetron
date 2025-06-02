use crate::{
    engine::{
        behaviours::{BehaviourFactory, BehaviourRef},
        entity::EntityRef,
        scene::SceneRef,
        systems::Ctx,
        world::WorldRef,
    },
    utils::Registrable,
};

use rune::{ContextError, Module};

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game"])?;
    WorldRef::register(&mut module)?;
    SceneRef::register(&mut module)?;
    BehaviourRef::register(&mut module)?;
    EntityRef::register(&mut module)?;
    BehaviourFactory::register(&mut module)?;
    Ctx::register(&mut module)?;

    Ok(module)
}
