use super::behaviours::{BehaviourFactory, BehaviourRef};
use crate::{
    error::TetronError,
    system_log,
    utils::typed_value::{TypedValue, schema::Schema},
};
use rune::{ContextError, Module, docstring, runtime::Object};

fn register_factory(module: &mut Module) -> Result<(), ContextError> {
    let schema = Schema::object()
        .field("type", Schema::string())
        .optional_field("w", Schema::number(), None)
        .optional_field("h", Schema::number(), None)
        .optional_field("r", Schema::number(), None)
        .optional_field("points", Schema::array(Schema::vec2()).min(2), None)
        .build();

    let shapes = BehaviourFactory::new("shape", schema, true);

    let func = move |name: &str, config: &Object| -> Result<BehaviourRef, TetronError> {
        let mut map = std::collections::HashMap::<String, TypedValue>::new();
        for (key, val) in config {
            map.insert(
                key.as_str().to_string(),
                val.try_into()
                    .inspect_err(|e| system_log!("shape::create (key {key}) error: {e:?}"))?,
            );
        }
        map.insert("type".into(), String::from(name).into());
        let shape = shapes
            .with_map(map)
            .inspect_err(|e| system_log!("shape::create shapes.with_map error: {e:?}"))?;

        // Minor runtime per-type check for stricter shape expectations:
        match name {
            "rect" => {
                if !shape.has("w") || !shape.has("h") {
                    return Err(TetronError::Runtime("rect requires fields 'w', 'h'".into()));
                }
            }
            "poly" => {
                if let Some(TypedValue::Array(points)) = shape.get_typed("points")? {
                    if points.len() < 3 {
                        return Err(TetronError::Runtime(
                            "poly requires at least 3 points".into(),
                        ));
                    }
                } else {
                    return Err(TetronError::Runtime("poly requires 'points' array".into()));
                }
            }
            "line" => {
                if let Some(TypedValue::Array(points)) = shape.get_typed("points")? {
                    if points.len() != 2 {
                        return Err(TetronError::Runtime(
                            "line requires exactly 2 points".into(),
                        ));
                    }
                } else {
                    return Err(TetronError::Runtime("line requires 'points' array".into()));
                }
            }
            "circle" => {
                if !shape.has("r") {
                    return Err(TetronError::Runtime("circle requires field 'r'".into()));
                }
            }
            _ => {
                return Err(TetronError::Runtime(format!(
                    "Invalid shape type {name} supplied"
                )));
            }
        }
        Ok(shape)
    };

    module.function("create", func).build()?.docs(docstring! {
        /// Create a new shape. Valid values for "type":
        ///
        /// * rect - a simple rectangle. Supply `w`, `h` in the options object.
        /// * poly - a polygon comprised of an arbitrary number of points. Must be convex.
        ///   Supply `points: [Vec2...]` in the options object. There must be at least 3 points.
        /// * line - a line with exactly 2 points. Supply `points: [vec2, vec2]` in the options object.
        /// * circle - a circle of radius `r`. Supply `r` in the options object.
    })?;
    Ok(())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game", "shape"])?;
    register_factory(&mut module)?;
    Ok(module)
}
