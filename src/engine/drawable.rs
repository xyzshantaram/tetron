use super::behaviours::{BehaviourFactory, BehaviourRef};
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
        .optional_field("sprite", Schema::string(), None)
        .optional_field("anim", Schema::string(), None)
        .build();

    let drawable = BehaviourFactory::new("drawable", schema, true);

    let func = move |obj: &Object| -> BehaviourRef { drawable.create(obj) };

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
