use super::behaviours::{BehaviourFactory, BehaviourRef};
use crate::{
    error::TetronError,
    utils::typed_value::{TypedValue, schema::Schema},
};
use rune::{ContextError, FromValue, Module, ToValue, docstring, runtime::Object};
use vec2::Vec2;

pub mod vec2;

fn register_factory(module: &mut Module) -> Result<(), ContextError> {
    let schema = Schema::object()
        .optional_field(
            "vel",
            Schema::vec2(),
            Some(TypedValue::Vector(Vec2::zero())),
        )
        .field("collision", Schema::string())
        .optional_field("mass", Schema::number(), None)
        .optional_field("friction", Schema::number(), None)
        .build();

    let physics = BehaviourFactory::new("physics", schema, true);

    let func =
        move |obj: &Object| -> Result<BehaviourRef, TetronError> {
            let behaviour = physics.create(obj)?;
            let collision = match behaviour.get_typed("collision")? {
                Some(TypedValue::String(s)) => s,
                _ => unreachable!(),
            };

            match collision.as_str() {
                "simulate" => match behaviour.get_typed("mass")? {
                    Some(TypedValue::Number(m)) if m > 0.0 => {}
                    _ => return Err(TetronError::Runtime(
                        "physics::create(): mass must be specified and > 0 for simulated bodies"
                            .into(),
                    )),
                },
                "immovable" | "none" => {}
                _ => {
                    return Err(TetronError::Runtime(format!(
                        "Invalid collision type {collision} specified"
                    )));
                }
            }
            Ok(behaviour)
        };

    module.function("create", func).build()?.docs(docstring! {
        /// Create a new physics behaviour.
        ///
        /// Fields:
        /// * collision: string ("simulate", "immovable", or "none")
        /// * vel: Vec2 (optional, default (0,0))
        /// * mass: number (optional, required if collision=="simulate")
        /// * friction: number (optional)
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
