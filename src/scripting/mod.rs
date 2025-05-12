use kv::{config_module, flags_module};
use rhai::{Engine, Module};
use std::{cell::RefCell, rc::Rc};
use stupid_simple_kv::Kv;

mod kv;
mod module_resolver;

use crate::{TetronError, fs::overlay_fs::OverlayFS};
use module_resolver::TetronModuleResolver;

pub struct TetronScripting {
    rhai: rhai::Engine,
}

impl TetronScripting {
    pub fn new(
        fs: Rc<OverlayFS>,
        flags: Rc<RefCell<Kv>>,
        config: Rc<Kv>,
    ) -> Result<TetronScripting, TetronError> {
        let mut engine = Engine::new();
        let mut module = Module::new();

        let resolver = TetronModuleResolver::new(fs.clone());
        let flags = flags_module(flags);
        let config = config_module(config);

        module.set_sub_module("flags", flags);
        module.set_sub_module("config", config);

        resolver.register_native_module("tetron", Rc::new(module))?;
        engine.set_module_resolver(resolver);

        Ok(Self { rhai: engine })
    }
}
