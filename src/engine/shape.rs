use super::behaviours::{BehaviourFactory, BehaviourRef};
use crate::{engine::physics::vec2::Vec2, error::TetronError, utils};
use rune::{
    ContextError, Module, ToValue, TypeHash, Value, alloc::clone::TryClone, docstring,
    runtime::Object,
};
use std::collections::{HashMap, HashSet};

fn is_points_array_invalid<'a>(
    count: usize,
    compare: impl Fn(&usize, &usize) -> bool + 'a,
) -> impl Fn(&rune::runtime::Vec) -> bool + 'a {
    move |vec: &rune::runtime::Vec| {
        compare(&vec.len(), &count) && vec.iter().any(|val| val.type_hash() != Vec2::HASH)
    }
}

fn shape_cfg_validator(name: &str) -> Box<dyn Fn(&Object) -> bool> {
    let value_can_be_float = |v: &Value| {
        let hash = v.type_hash();
        hash == u64::HASH || hash == f64::HASH
    };

    match name {
        "rect" => Box::new(move |obj: &Object| {
            obj.get("w").is_some_and(value_can_be_float)
                && obj.get("h").is_some_and(value_can_be_float)
        }),
        "poly" => {
            let checker = is_points_array_invalid(3, usize::ge);
            Box::new(move |obj: &Object| {
                obj.get("points").is_some_and(|points| {
                    points
                        .borrow_ref::<rune::runtime::Vec>()
                        .ok()
                        .is_some_and(|v| !checker(&v))
                })
            })
        }
        "line" => {
            let checker = is_points_array_invalid(2, usize::eq);
            Box::new(move |obj: &Object| {
                obj.get("points").is_some_and(|points| {
                    points
                        .borrow_ref::<rune::runtime::Vec>()
                        .ok()
                        .is_some_and(|v| !checker(&v))
                })
            })
        }
        "circle" => Box::new(move |obj: &Object| obj.get("r").is_some_and(value_can_be_float)), // Placeholder
        _ => panic!("Unknown type name encountered finding validator"),
    }
}

fn register_factory(module: &mut Module) -> Result<(), ContextError> {
    let shapes = BehaviourFactory::new(
        "shape",
        HashSet::from([
            "type".into(),
            "w".into(),
            "h".into(),
            "r".into(),
            "points".into(),
        ]),
        true,
    );

    let func = move |name: &str, config: &Object| -> Result<BehaviourRef, TetronError> {
        if matches!(name, "rect" | "poly" | "line" | "circle") {
            if shape_cfg_validator(name)(config) {
                let mut map = HashMap::<String, Value>::new();
                for (key, val) in config {
                    map.insert(key.as_str().to_string(), val.try_clone()?);
                }
                map.insert("type".into(), String::from(name).to_value()?);
                shapes.with_map(map)
            } else {
                Err(TetronError::Runtime(format!(
                    "Invalid shape initializer supplied for shape '{name}'"
                )))
            }
        } else {
            Err(TetronError::Runtime(format!(
                "Invalid shape type {name} supplied"
            )))
        }
    };

    module.function("create", func).build()?.docs(docstring! {
        /// Create a new shape. Valid values for "type":
        ///
        /// * rect - a simple rectangle. Supply `w`, `h` in the options object.
        /// * poly - a polygon comprised of an arbitrary number of points. Must be convex.
        ///   Supply `points: [Vec2...]` in the options object. There must be atleast 3 points.
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
