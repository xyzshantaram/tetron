use super::behaviours::{BehaviourFactory, BehaviourRef};
use crate::error::TetronError;
use crate::utils;
use rune::{ContextError, Module, ToValue, docstring, runtime::Object};
use std::collections::HashSet;

fn register_factory(module: &mut Module) -> Result<(), ContextError> {
    let transform = BehaviourFactory::new(
        "transform",
        HashSet::from(["x".into(), "y".into(), "rot".into()]),
        true,
    );

    let func = move |obj: &Object| -> Result<BehaviourRef, TetronError> {
        let x = obj
            .get("x")
            .and_then(|v| v.as_float().ok())
            .unwrap_or(0.0)
            .to_value()?;
        let y = obj
            .get("y")
            .and_then(|v| v.as_float().ok())
            .unwrap_or(0.0)
            .to_value()?;
        let rot = obj
            .get("rot")
            .and_then(|v| v.as_float().ok())
            .unwrap_or(0.0)
            .to_value()?;

        let mut val = Object::new();
        val.insert(utils::rune::obj_key("x")?, x)?;
        val.insert(utils::rune::obj_key("y")?, y)?;
        val.insert(utils::rune::obj_key("rot")?, rot)?;

        transform.create(val)
    };

    module.function("create", func).build()?.docs(docstring! {
        /// Create a new transform behaviour. All fields are optional and default to zero if not specified.
        ///
        /// Possible fields:
        /// * x: f64
        /// * y: f64
        /// * rot: f64
    })?;
    Ok(())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game", "transform"])?;
    register_factory(&mut module)?;
    Ok(module)
}
