use super::{
    behaviours::{BehaviourFactory, BehaviourRef},
    physics::vec2::Vec2,
};
use crate::{error::TetronError, utils};
use rune::{ContextError, FromValue, Module, ToValue, docstring, runtime::Object};
use std::collections::HashSet;

#[rune::function(keep)]
pub fn rotate(b: &mut BehaviourRef, angle: f64) -> Result<(), TetronError> {
    let old = if let Some(value) = b.get("rot")? {
        f64::from_value(value)?
    } else {
        0.0
    };
    b.set("rot", (old + angle).to_value()?)?;
    Ok(())
}

#[rune::function(keep)]
pub fn translate(b: &mut BehaviourRef, delta: Vec2) -> Result<(), TetronError> {
    let current_pos = if let Some(value) = b.get("pos")? {
        Vec2::from_value(value)?
    } else {
        Vec2::zero()
    };
    let new_pos = current_pos + delta;
    b.set("pos", new_pos.to_value()?)?;
    Ok(())
}

fn register_factory(module: &mut Module) -> Result<(), ContextError> {
    let transform = BehaviourFactory::new(
        "transform",
        HashSet::from(["pos".into(), "rot".into()]),
        true,
    );

    let func = move |obj: &Object| -> Result<BehaviourRef, TetronError> {
        let pos = obj
            .get("pos")
            .cloned()
            .unwrap_or(Vec2::new(0.0, 0.0).to_value()?);
        let rot = obj
            .get("rot")
            .and_then(|v| v.as_float().ok())
            .unwrap_or(0.0);

        let mut val = Object::new();
        val.insert(utils::rune::obj_key("pos")?, pos)?;
        val.insert(utils::rune::obj_key("rot")?, rot.into())?;

        transform.create(val)
    };

    module.function("create", func).build()?.docs(docstring! {
        /// Create a new transform behaviour. All fields are optional and default to zero if not specified.
        ///
        /// Possible fields:
        /// * pos: Vec2
        /// * rot: f64
    })?;
    Ok(())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game", "transform"])?;
    register_factory(&mut module)?;
    module.function_meta(translate__meta)?;
    module.function_meta(rotate__meta)?;
    Ok(module)
}
