use crate::engine::{
    behaviours::{BehaviourFactory, BehaviourRef},
    entity::EntityRef,
    scene::SceneRef,
    world::WorldRef,
};

use rune::{ContextError, Module};

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game"])?;
    module.ty::<WorldRef>()?;
    module.ty::<SceneRef>()?;
    module.ty::<BehaviourRef>()?;
    module.ty::<EntityRef>()?;
    module.ty::<BehaviourFactory>()?;

    Ok(module)
}
