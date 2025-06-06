use super::behaviours::{BehaviourFactory, BehaviourRef};
use crate::error::TetronError;
use crate::utils::typed_value::schema::Schema;
use rune::{ContextError, Module, docstring, runtime::Object};

fn register_factory(module: &mut Module) -> Result<(), ContextError> {
    // font must be an Object with required size (number), optional face (string)
    let font_schema = Schema::object()
        .field("size", Schema::number())
        .optional_field("face", Schema::string(), None)
        .build();

    let schema = Schema::object()
        .optional_field("color", Schema::string(), None)
        .optional_field("text", Schema::string(), None)
        .optional_field("font", font_schema, None)
        .build();

    let drawable = BehaviourFactory::new("drawable", schema, true);

    let func = move |obj: &Object| -> Result<BehaviourRef, TetronError> {
        drawable
            .create(obj)
            .inspect_err(|e| println!("error building drawable: {e}"))
    };

    module.function("create", func).build()?.docs(docstring! {
        /// Create a new drawable behaviour.
        ///
        /// Fields:
        /// * color: string
        /// * text: string
        /// * font: object with size (number) and optional face (string)
    })?;
    Ok(())
}

pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["game", "drawable"])?;
    register_factory(&mut module)?;
    Ok(module)
}
