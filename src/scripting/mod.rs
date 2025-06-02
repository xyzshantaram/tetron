use crate::{engine::input, fs::SimpleFs};
use crate::{engine::input::KeyState, error::TetronError};
use rune::{
    Context, Diagnostics, Module, Source, Sources, ToTypeHash, Vm,
    runtime::RuntimeContext,
    termcolor::{ColorChoice, StandardStream},
};
use source_loader::SimpleFsSourceLoader;
use std::{
    path::Path,
    rc::Rc,
    sync::{Arc, RwLock},
};
use stupid_simple_kv::Kv;

mod game;
mod kv;
pub mod log;
mod math;
mod source_loader;

use crate::engine::drawable;
use crate::engine::physics;
use crate::engine::shape;
use crate::engine::transform;

pub struct TetronScripting {
    context: Arc<Context>,
    runtime: Arc<RuntimeContext>,
    loader: SimpleFsSourceLoader,
    fs: Rc<dyn SimpleFs>,
}

fn tetron_modules(
    flags: Arc<RwLock<Kv>>,
    config: Arc<Kv>,
    input: Arc<RwLock<KeyState>>,
) -> Result<Vec<Module>, TetronError> {
    // custom tetron modules
    let math = math::module()?;
    let log = log::module()?;
    let flags = kv::flags::module(flags)?;
    let config = kv::config::module(config)?;
    let game = game::module()?;
    let physics = physics::module()?;
    let shape = shape::module()?;
    let drawable = drawable::module()?;
    let transform = transform::module()?;
    let input = input::module(input)?;

    Ok(vec![
        math, log, flags, config, game, shape, drawable, transform, physics, input,
    ])
}

pub fn tetron_context(
    flags: Arc<RwLock<Kv>>,
    config: Arc<Kv>,
    input: Arc<RwLock<KeyState>>,
) -> Result<Context, TetronError> {
    let mut context = Context::with_config(false)?;
    for module in tetron_modules(flags, config, input.clone())? {
        context.install(module)?;
    }

    Ok(context)
}

impl TetronScripting {
    pub fn new(
        fs: Rc<dyn SimpleFs>,
        flags: Arc<RwLock<Kv>>,
        config: Arc<Kv>,
        input: Arc<RwLock<KeyState>>,
    ) -> Result<TetronScripting, TetronError> {
        let context = tetron_context(flags, config, input)?;
        let runtime = context.runtime()?;
        let loader = SimpleFsSourceLoader::new(fs.clone());

        Ok(Self {
            fs,
            context: Arc::new(context),
            runtime: Arc::new(runtime),
            loader,
        })
    }

    pub fn execute(
        &mut self,
        path: &str,
        func: impl ToTypeHash,
        args: impl rune::runtime::Args,
    ) -> Result<(), TetronError> {
        let p = Path::new(path);
        let filename = p
            .file_name()
            .ok_or(TetronError::ModuleNotFound(path.into()))?
            .to_str()
            .ok_or(TetronError::Runtime(
                "Unable to convert filename of path".into(),
            ))?;

        let contents = self.fs.read_text_file(path)?;
        let mut sources = Sources::new();
        sources.insert(Source::new(filename, contents)?)?;

        let mut diagnostics = Diagnostics::new();
        let result = rune::prepare(&mut sources)
            .with_context(&self.context)
            .with_diagnostics(&mut diagnostics)
            .with_source_loader(&mut self.loader)
            .build();

        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &sources)?;
        }

        let unit = result?;
        let mut vm = Vm::new(self.runtime.clone(), Arc::new(unit));
        vm.execute(func, args)?.complete().into_result()?;
        Ok(())
    }
}
