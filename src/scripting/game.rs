use std::collections::HashSet;

use crate::{
    engine::{
        behaviours::{BehaviourFactory, BehaviourRef},
        entity::EntityRef,
        physics::vec2::Vec2,
        scene::SceneRef,
        world::WorldRef,
    },
    error::TetronError,
};

use rune::{
    ContextError, FromValue, Module, TypeHash, Value,
    alloc::{clone::TryClone, string::TryToString},
    docstring,
    runtime::Object,
};
use rune::{ToValue, alloc::String as RuneString};

fn clone_rune_object(obj: &Object) -> Result<Object, TetronError> {
    let mut copy = Object::new();
    for item in obj.iter() {
        copy.insert(item.0.try_to_string()?, item.1.try_clone()?)?;
    }

    Ok(copy)
}

fn obj_key(s: &str) -> Result<RuneString, rune::alloc::Error> {
    RuneString::try_from(s)
}

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

fn shape_behaviour(module: &mut Module) -> Result<(), ContextError> {
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

    module
        .function(
            "shape",
            move |name: &str, config: &Object| -> Result<BehaviourRef, TetronError> {
                if matches!(name, "rect" | "poly" | "line" | "circle") {
                    if shape_cfg_validator(name)(config) {
                        let mut config = clone_rune_object(config)?;
                        config.insert(obj_key("type")?, obj_key(name)?.to_value()?)?;
                        Ok(shapes.create(config)?)
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
            },
        )
        .build()?
        .docs(docstring! {
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

fn transform_behaviour(module: &mut Module) -> Result<(), ContextError> {
    let transform = BehaviourFactory::new(
        "transform",
        HashSet::from(["x".into(), "y".into(), "rot".into()]),
        true,
    );

    let func = move |obj: &Object| {
        transform.create({
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

            val.insert(obj_key("x")?, x)?;
            val.insert(obj_key("y")?, y)?;
            val.insert(obj_key("rot")?, rot)?;

            val
        })
    };

    module
        .function("transform", func)
        .build()?
        .docs(docstring! {
            /// Create a new transform behaviour. All fields are optional and default to zero if not specified.
            ///
            /// Possible fields:
            /// * x: f64
            /// * y: f64
            /// * rot: f64
        })?;
    Ok(())
}

fn physics_behaviour(module: &mut Module) -> Result<(), ContextError> {
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
                            "physics(): collision behaviour must be specified".into(),
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
                    "physics(): mass must be specified for simulated bodies".into(),
                ));
            } else if collision == "immovable" {
                mass.replace(f64::INFINITY);
            }

            let mut val = Object::new();
            val.insert(obj_key("vel")?, vel.to_value()?)?;
            val.insert(obj_key("collision")?, collision.to_value()?)?;
            val.insert(obj_key("mass")?, mass.unwrap_or(-1.0).into())?;
            val
        })
    };

    module.function("physics", func).build()?.docs(docstring! {
        /// Create a new physics behaviour.
    })?;

    Ok(())
}

fn drawable_behaviour(module: &mut Module) -> Result<(), ContextError> {
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
        let copy = clone_rune_object(obj)?;
        drawable.create(copy)
    };

    module
        .function("drawable", func)
        .build()?
        .docs(docstring! {
            /// Create a new drawable behaviour.
        })?;

    Ok(())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game"])?;
    module.ty::<WorldRef>()?;
    module.ty::<SceneRef>()?;
    module.ty::<BehaviourRef>()?;
    module.ty::<EntityRef>()?;
    module.ty::<BehaviourFactory>()?;

    transform_behaviour(&mut module)?;
    physics_behaviour(&mut module)?;
    shape_behaviour(&mut module)?;
    drawable_behaviour(&mut module)?;

    Ok(module)
}
