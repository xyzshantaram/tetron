use crate::engine::{behaviours::Behaviour, entity::EntityRef, scene::SceneRef, world::WorldRef};
use rune::{ContextError, Module};

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game"])?;
    module.ty::<WorldRef>()?;
    module.ty::<SceneRef>()?;
    module.ty::<Behaviour>()?;
    module.ty::<EntityRef>()?;

    Ok(module)
}
