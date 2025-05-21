use crate::{TetronError, fs::overlay_fs::OverlayFS};
use game::game_module;
use kv::{config_module, flags_module};
use log::log_module;
use math::math_module;
use resolver::TetronModuleResolver;
use rhai::{Engine, Module};
use std::{cell::RefCell, rc::Rc};
use stupid_simple_kv::Kv;
use utils::setup_native_module;

mod game;
mod kv;
mod log;
mod math;
mod resolver;
pub mod utils;

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
        math_module(engine),
        game_module(engine),
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
        engine.set_fast_operators(false);
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
