use std::sync::{Arc, RwLock};

use rune::{ContextError, Module, Value};
use stupid_simple_kv::Kv;

use super::utils::{kv_value_to_rune, rune_value_to_kv, rune_vec_to_kv_key};
use crate::error::TetronError;

pub fn module(flags: Arc<RwLock<Kv>>) -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["flags"])?;

    let setter = flags.clone();
    let getter = flags.clone();
    let remover = flags.clone();
    let clearer = flags.clone();

    module
        .function("clear", move || {
            clearer
                .try_write()
                .expect("Engine bug: flags lock poisoned")
                .clear()
                .expect("Engine bug: failed to clear flags");
        })
        .build()?;

    module
        .function(
            "delete",
            move |key_array: Vec<Value>| {
                let kv_key = rune_vec_to_kv_key(key_array).expect("Engine bug: failed to convert key array");
                remover
                    .try_write()
                    .expect("Engine bug: flags lock poisoned")
                    .delete(&kv_key)
                    .expect("Engine bug: failed to delete from flags");
            },
        )
        .build()?;

    module
        .function(
            "get",
            move |key_array: Vec<Value>| -> Option<Value> {
                let kv_key = rune_vec_to_kv_key(key_array).expect("Engine bug: failed to convert key array");
                let val = getter
                    .try_read()
                    .expect("Engine bug: flags lock poisoned")
                    .get(&kv_key)
                    .expect("Engine bug: failed to get from flags");
                if let Some(value) = val {
                    Some(kv_value_to_rune(&value).expect("Engine bug: failed to convert value to rune"))
                } else {
                    None
                }
            },
        )
        .build()?;

    module
        .function(
            "set",
            move |key_array: Vec<Value>, value: Value| {
                let kv_value = rune_value_to_kv(value).expect("Engine bug: failed to convert value to kv");
                let kv_key = rune_vec_to_kv_key(key_array).expect("Engine bug: failed to convert key array");
                setter
                    .try_write()
                    .expect("Engine bug: flags lock poisoned")
                    .set(&kv_key, kv_value)
                    .expect("Engine bug: failed to set flags value");
            },
        )
        .build()?;

    Ok(module)
}
