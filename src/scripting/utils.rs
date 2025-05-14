use std::rc::Rc;

use rhai::{FnNamespace, FuncRegistration, Module, RhaiNativeFunc, Variant};

use crate::TetronError;

use super::module_resolver::TetronModuleResolver;

#[derive(Debug, Clone)]
pub struct FnOpts {
    pure: bool,
    volatile: bool,
    global: bool,
}

impl Default for FnOpts {
    fn default() -> Self {
        Self::new()
    }
}

impl FnOpts {
    pub fn new() -> Self {
        Self {
            pure: true,
            volatile: false,
            global: false,
        }
    }

    /** A function marked global will not need namespacing to be called. See __tetron_logger_internal. */
    pub fn global(&mut self) -> &mut Self {
        self.global = true;
        self
    }

    /** A volatile function may return different results for the same set of arguments. */
    pub fn volatile(&mut self) -> &mut Self {
        self.volatile = true;
        self
    }

    /** An impure function is one that modifies its arguments. */
    pub fn impure(&mut self) -> &mut Self {
        self.pure = false;
        self
    }
}

pub fn setup_native_module(
    global: &mut Module,
    name: &str,
    module: Rc<Module>,
    resolver: &mut TetronModuleResolver,
) -> Result<(), TetronError> {
    global.set_sub_module(name, module.clone());
    resolver.register_native_module(&format!("tetron:{name}"), module)?;
    Ok(())
}

pub fn register_fn<A: 'static, const N: usize, const X: bool, R, const F: bool, FN>(
    module: &mut Module,
    name: &str,
    fun: FN,
    opts: &FnOpts,
) where
    R: Variant + Clone,
    FN: RhaiNativeFunc<A, N, X, R, F> + 'static,
{
    let FnOpts {
        pure,
        volatile,
        global,
    } = opts;
    FuncRegistration::new(name)
        .with_purity(*pure)
        .with_namespace(if *global {
            FnNamespace::Global
        } else {
            FnNamespace::Internal
        })
        .with_volatility(*volatile)
        .set_into_module(module, fun);
}
