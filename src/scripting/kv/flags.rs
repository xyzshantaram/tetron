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
        .function("clear", move || -> Result<(), TetronError> {
            clearer
                .try_write()
                .map_err(|e| TetronError::KvError(format!("mutable borrow failed: {e}")))?
                .clear()?;
            Ok(())
        })
        .build()?;

    module
        .function(
            "delete",
            move |key_array: Vec<Value>| -> Result<(), TetronError> {
                let kv_key = rune_vec_to_kv_key(key_array)?;
                remover
                    .try_write()
                    .map_err(|e| TetronError::KvError(format!("borrow failed: {e}")))?
                    .delete(&kv_key)?;
                Ok(())
            },
        )
        .build()?;

    module
        .function(
            "get",
            move |key_array: Vec<Value>| -> Result<Option<Value>, TetronError> {
                let kv_key = rune_vec_to_kv_key(key_array)?;
                let val = getter
                    .try_read()
                    .map_err(|e| TetronError::KvError(format!("borrow failed: {e}")))?
                    .get(&kv_key)?;
                Ok(if let Some(value) = val {
                    Some(kv_value_to_rune(&value)?)
                } else {
                    None
                })
            },
        )
        .build()?;

    module
        .function(
            "set",
            move |key_array: Vec<Value>, value: Value| -> Result<(), TetronError> {
                let kv_value = rune_value_to_kv(value)?;
                let kv_key = rune_vec_to_kv_key(key_array)?;
                setter
                    .try_write()
                    .map_err(|e| TetronError::KvError(format!("mutable borrow failed: {e}")))?
                    .set(&kv_key, kv_value)
                    .map_err(|e| TetronError::KvError(format!("setting kv value failed: {e}")))?;
                Ok(())
            },
        )
        .build()?;

    Ok(module)
}
