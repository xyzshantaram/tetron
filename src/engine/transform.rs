use super::{
    behaviours::{BehaviourFactory, BehaviourRef},
    physics::vec2::Vec2,
};
use crate::utils::typed_value::{TypedValue, schema::Schema};
use rune::{ContextError, FromValue, Module, ToValue, docstring, runtime::Object};

#[rune::function(keep)]
pub fn rotate(b: &mut BehaviourRef, angle: f64) {
    let old = if let Some(value) = b.get("rot") {
        f64::from_value(value).expect("Engine bug: failed to convert rotation value")
    } else {
        0.0
    };
    b.set(
        "rot",
        (old + angle)
            .to_value()
            .expect("Engine bug: failed to convert rotation to rune value"),
    );
}

#[rune::function(keep)]
pub fn translate(b: &mut BehaviourRef, delta: Vec2) {
    let current_pos = if let Some(value) = b.get("pos") {
        Vec2::from_value(value).expect("Engine bug: failed to convert position value")
    } else {
        Vec2::zero()
    };
    let new_pos = current_pos + delta;
    b.set(
        "pos",
        new_pos
            .to_value()
            .expect("Engine bug: failed to convert position to rune value"),
    );
}

fn register_factory(module: &mut Module) -> Result<(), ContextError> {
    let schema = Schema::object()
        .optional_field(
            "pos",
            Schema::vec2(),
            Some(TypedValue::Vector(Vec2::zero())),
        )
        .optional_field("rot", Schema::number(), Some(TypedValue::Number(0.0)))
        .build();

    let transform = BehaviourFactory::new("transform", schema, true);

    let func = move |obj: &Object| -> BehaviourRef { transform.create(obj) };

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
