use super::behaviours::{BehaviourFactory, BehaviourRef};
use crate::error::TetronError;
use rune::{ContextError, FromValue, Module, ToValue, Value, docstring, runtime::Object};
use std::collections::{HashMap, HashSet};
use vec2::Vec2;

pub mod vec2;

fn register_factory(module: &mut Module) -> Result<(), ContextError> {
    let physics = BehaviourFactory::new(
        "physics",
        HashSet::from(["vel".into(), "collision".into(), "mass".into()]),
        true,
    );

    let func = move |obj: &Object| -> Result<BehaviourRef, TetronError> {
        physics.with_map({
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

            let mut map = HashMap::<String, Value>::new();
            map.insert("vel".into(), vel.to_value()?);
            map.insert("collision".into(), collision.to_value()?);
            map.insert("mass".into(), mass.unwrap_or(-1.0).into());

            map
        })
    };

    module.function("create", func).build()?.docs(docstring! {
        /// Create a new physics behaviour.
    })?;

    Ok(())
}

#[rune::function]
fn vec2(x: f64, y: f64) -> Vec2 {
    Vec2::new(x, y)
}

#[rune::function(keep)]
pub fn apply_force(b: &mut BehaviourRef, force: Vec2) -> Result<(), TetronError> {
    let vel = if let Some(val) = b.get("vel")? {
        Vec2::from_value(val)?
    } else {
        Vec2::zero()
    };
    b.set("vel", (vel + force).to_value()?)?;
    Ok(())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game", "physics"])?;
    register_factory(&mut module)?;
    module.function_meta(vec2)?;
    module.function_meta(apply_force__meta)?;
    Ok(module)
}
