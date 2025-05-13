use std::rc::Rc;

use rhai::{FnNamespace, FuncRegistration, Module, RhaiNativeFunc, Variant};

use crate::TetronError;

use super::module_resolver::TetronModuleResolver;

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
    pvg: Option<(bool, bool, bool)>,
) where
    R: Variant + Clone,
    FN: RhaiNativeFunc<A, N, X, R, F> + 'static,
{
    let (pure, volatile, global) = pvg.unwrap_or((false, false, false));
    FuncRegistration::new(name)
        .with_purity(pure)
        .with_namespace(if global {
            FnNamespace::Global
        } else {
            FnNamespace::Internal
        })
        .with_volatility(volatile)
        .set_into_module(module, fun);
}
