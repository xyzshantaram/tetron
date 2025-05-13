use crate::{TetronError, fs::overlay_fs::OverlayFS};
use kv::{config_module, flags_module};
use log::log_module;
use module_resolver::TetronModuleResolver;
use rhai::{Engine, Module};
use std::{cell::RefCell, rc::Rc};
use stupid_simple_kv::Kv;
use utils::setup_native_module;

mod kv;
mod log;
mod module_resolver;
mod utils;

pub struct TetronScripting {
    rhai: rhai::Engine,
}

type NativeModule = (&'static str, Rc<Module>);
fn tetron_modules(
    engine: &mut Engine,
    flags: Rc<RefCell<Kv>>,
    config: Rc<Kv>,
) -> Result<Vec<NativeModule>, TetronError> {
    let modules: Vec<NativeModule> = vec![
        flags_module(flags),
        config_module(config),
        log_module(engine)?,
    ];
    Ok(modules)
}

impl TetronScripting {
    pub fn new(
        fs: Rc<OverlayFS>,
        flags: Rc<RefCell<Kv>>,
        config: Rc<Kv>,
    ) -> Result<TetronScripting, TetronError> {
        let mut engine = Engine::new();
        let mut global = Module::new();

        let mut resolver = TetronModuleResolver::new(fs.clone());

        let modules = tetron_modules(&mut engine, flags, config)?;
        for (name, module) in modules {
            setup_native_module(&mut global, name, module, &mut resolver)?;
        }

        resolver.register_native_module("tetron", Rc::new(global))?;
        engine.set_module_resolver(resolver);

        Ok(Self { rhai: engine })
    }

    pub fn eval<T: Clone + 'static>(&self, source: &str) -> Result<T, TetronError> {
        self.rhai
            .eval::<T>(source)
            .map_err(|e| TetronError::RhaiRuntime(e.to_string(), None))
    }
}
