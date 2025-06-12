use super::behaviours::{BehaviourFactory, BehaviourRef};
use crate::{
    log_and_die,
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

    let func = move |name: &str, config: &Object| -> BehaviourRef {
        let mut map = std::collections::HashMap::<String, TypedValue>::new();
        for (key, val) in config {
            map.insert(
                key.as_str().to_string(),
                val.try_into()
                    .expect("Engine bug: failed to convert rune value to typed value"),
            );
        }
        map.insert("type".into(), String::from(name).into());
        let shape = shapes.with_map(map);

        // Minor runtime per-type check for stricter shape expectations:
        match name {
            "rect" => {
                if !shape.has("w") || !shape.has("h") {
                    log_and_die!(1, "rect constructor requires fields 'w', 'h'");
                }
            }
            "poly" => {
                if let Some(TypedValue::Array(points)) = shape.get_typed("points") {
                    if points.len() < 3 {
                        log_and_die!(1, "poly shape requires at least 3 points");
                    }
                } else {
                    log_and_die!(1, "poly requires 'points' array");
                }
            }
            "line" => {
                if let Some(TypedValue::Array(points)) = shape.get_typed("points") {
                    if points.len() != 2 {
                        log_and_die!(1, "line requires exactly 2 points");
                    }
                } else {
                    log_and_die!(1, "line requires 'points' array");
                }
            }
            "circle" => {
                if !shape.has("r") {
                    log_and_die!(1, "circle requires field 'r'");
                }
            }
            _ => {
                log_and_die!(1, "Invalid shape type {name} supplied");
            }
        }
        shape
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
