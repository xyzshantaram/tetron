use std::{cell::RefCell, rc::Rc};

use conversions::{from_kv_value, rhai_dyn_to_kvkey, to_kv_value};
use rhai::{Dynamic, EvalAltResult, FuncRegistration, Module};
use stupid_simple_kv::Kv;

use crate::TetronError;

mod conversions;

pub fn flags_module(flags: Rc<RefCell<Kv>>) -> Rc<Module> {
    let mut module = Module::new();

    let flags_setter = flags.clone();
    FuncRegistration::new("set_flag")
        .in_internal_namespace()
        .with_purity(false)
        .with_volatility(false)
        .set_into_module(
            &mut module,
            move |k: Dynamic, v: Dynamic| -> Result<(), Box<EvalAltResult>> {
                let key = rhai_dyn_to_kvkey(k.clone())?;
                let value = to_kv_value(&v)?;
                Ok(flags_setter
                    .try_borrow_mut()
                    .map_err(|e| {
                        TetronError::RhaiRuntime(format!("Could not get flags instance: {e}"), None)
                    })?
                    .set(&key, value)
                    .map_err(|e| {
                        TetronError::RhaiRuntime(format!("Could not set flag {k} value: {e}"), None)
                    })?)
            },
        );

    let flags_getter = flags.clone();

    FuncRegistration::new("get_flag")
        .in_internal_namespace()
        .with_purity(false)
        .with_volatility(false)
        .set_into_module(
            &mut module,
            move |k: Dynamic| -> Result<Dynamic, Box<EvalAltResult>> {
                let key = rhai_dyn_to_kvkey(k.clone())?;
                let v = flags_getter
                    .try_borrow()
                    .map_err(|e| {
                        TetronError::RhaiRuntime(format!("Could not get flags instance: {e}"), None)
                    })?
                    .get(&key)
                    .map_err(|e| {
                        TetronError::RhaiRuntime(format!("Could not get flag {k} value: {e}"), None)
                    })?;
                Ok(v.map(|val| from_kv_value(&val)).unwrap_or(Dynamic::UNIT))
            },
        );

    Rc::new(module)
}

pub fn config_module(config: Rc<Kv>) -> Rc<Module> {
    let mut module = Module::new();
    let config_getter = config.clone();

    FuncRegistration::new("get_flag")
        .in_internal_namespace()
        .with_purity(false)
        .with_volatility(false)
        .set_into_module(
            &mut module,
            move |k: Dynamic| -> Result<Dynamic, Box<EvalAltResult>> {
                let key = rhai_dyn_to_kvkey(k.clone())?;
                let v = config_getter.get(&key).map_err(|e| {
                    TetronError::RhaiRuntime(format!("Could not get config value {k}: {e}"), None)
                })?;
                Ok(v.map(|val| from_kv_value(&val)).unwrap_or(Dynamic::UNIT))
            },
        );

    Rc::new(module)
}
