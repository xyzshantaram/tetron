use std::sync::Arc;

use rune::{ContextError, Module, Value};
use stupid_simple_kv::Kv;

use super::utils::{kv_value_to_rune, rune_vec_to_kv_key};
use crate::TetronError;

pub fn module(config: Arc<Kv>) -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["config"])?;
    let getter = config.clone();

    module
        .function(
            "get",
            move |key_array: Vec<Value>| -> Result<Option<Value>, TetronError> {
                let kv_key = rune_vec_to_kv_key(key_array)?;
                let val = getter.get(&kv_key)?;
                Ok(if let Some(value) = val {
                    Some(kv_value_to_rune(&value)?)
                } else {
                    None
                })
            },
        )
        .build()?;

    Ok(module)
}
