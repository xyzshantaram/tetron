use super::behaviours::{BehaviourFactory, BehaviourRef};
use crate::{error::TetronError, utils};
use rune::{ContextError, FromValue, Module, ToValue, docstring, runtime::Object};
use std::collections::HashSet;
use vec2::Vec2;

pub mod vec2;

fn register_factory(module: &mut Module) -> Result<(), ContextError> {
    let physics = BehaviourFactory::new(
        "physics",
        HashSet::from(["vel".into(), "collision".into(), "mass".into()]),
        true,
    );

    let func = move |obj: &Object| -> Result<BehaviourRef, TetronError> {
        physics.create({
            let vel = match obj.get("vel") {
                Some(value) => Vec2::from_value(value.clone())?,
                None => Vec2::zero(),
            };

            let collision = String::from_value(
                obj.get("collision")
                    .ok_or_else(|| {
                        TetronError::Runtime(
                            "physics::create(): collision behaviour must be specified".into(),
                        )
                    })?
                    .clone(),
            )?;

            if !(["simulate", "immovable", "none"].contains(&collision.as_str())) {
                return Err(TetronError::Runtime(format!(
                    "Invalid collision type {collision} specified"
                )));
            }

            let mut mass = obj.get("mass").and_then(|v| v.as_float().ok());

            if collision == "simulate" && mass.is_none() {
                return Err(TetronError::Runtime(
                    "physics::create(): mass must be specified for simulated bodies".into(),
                ));
            } else if collision == "immovable" {
                mass.replace(f64::INFINITY);
            }

            let mut val = Object::new();
            val.insert(utils::rune::obj_key("vel")?, vel.to_value()?)?;
            val.insert(utils::rune::obj_key("collision")?, collision.to_value()?)?;
            val.insert(utils::rune::obj_key("mass")?, mass.unwrap_or(-1.0).into())?;
            val
        })
    };

    module.function("create", func).build()?.docs(docstring! {
        /// Create a new physics behaviour.
    })?;

    Ok(())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game", "physics"])?;
    register_factory(&mut module)?;
    Ok(module)
}
