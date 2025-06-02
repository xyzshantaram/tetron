use super::behaviours::{BehaviourFactory, BehaviourRef};
use crate::{error::TetronError, utils};
use rune::{ContextError, Module, docstring, runtime::Object};
use std::collections::HashSet;

fn register_factory(module: &mut Module) -> Result<(), ContextError> {
    let drawable = BehaviourFactory::new(
        "drawable",
        HashSet::from([
            "color".into(),
            "anim".into(),
            "font".into(),
            "text".into(),
            "stroke".into(),
            "fill".into(),
        ]),
        true,
    );

    let func = move |obj: &Object| -> Result<BehaviourRef, TetronError> {
        let copy = utils::rune::clone_obj(obj)?;
        drawable.create(copy)
    };

    module.function("create", func).build()?.docs(docstring! {
        /// Create a new drawable behaviour.
    })?;
    Ok(())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game", "drawable"])?;
    register_factory(&mut module)?;
    Ok(module)
}
