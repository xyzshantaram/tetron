use super::behaviours::{BehaviourFactory, BehaviourRef};
use crate::error::TetronError;
use rune::{ContextError, Module, TypeHash, docstring, runtime::Object};
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
            "sprite".into(),
        ]),
        true,
    );

    let func = move |obj: &Object| -> Result<BehaviourRef, TetronError> {
        for field in ["color", "stroke", "fill", "text"] {
            if let Some(v) = obj.get(field) {
                if v.type_hash() != String::HASH {
                    return Err(TetronError::Runtime(
                        "Drawable field '{field}' must be a string!".into(),
                    ));
                }
            }
        }

        if let Some(font_val) = obj.get("font") {
            if font_val.type_hash() != Object::HASH {
                return Err(TetronError::Runtime(
                    "Drawable field 'font' must be an object".into(),
                ));
            }
        }

        drawable
            .create(obj)
            .inspect_err(|e| println!("error building drawable: {e}"))
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
