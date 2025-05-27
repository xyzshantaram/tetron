use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use rune::{Context, Module, runtime::RuntimeContext};
use stupid_simple_kv::Kv;

use crate::{TetronError, engine, fs::overlay_fs::OverlayFS};

mod kv;
pub mod log;
mod math;

#[derive(Clone)]
pub struct TetronScripting {
    runtime: Rc<RuntimeContext>,
}

fn tetron_modules(flags: Arc<RwLock<Kv>>, config: Arc<Kv>) -> Result<Vec<Module>, TetronError> {
    // custom tetron modules
    let world = engine::world::World::module()?;
    let math = math::module()?;
    let log = log::module()?;
    let flags = kv::flags::module(flags)?;
    let config = kv::config::module(config)?;

    Ok(vec![world, math, log, flags, config])
}

impl TetronScripting {
    pub fn new(
        fs: Rc<OverlayFS>,
        flags: Arc<RwLock<Kv>>,
        config: Arc<Kv>,
    ) -> Result<TetronScripting, TetronError> {
        let mut context = Context::with_config(false)?;
        for module in tetron_modules(flags, config)? {
            context.install(module)?;
        }
        todo!()
    }
}
