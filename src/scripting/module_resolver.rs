use crate::{
    TetronError,
    fs::{SimpleFS, normalize_path, overlay_fs::OverlayFS},
};
use rhai::{Engine, EvalAltResult, Module, ModuleResolver, Scope};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct TetronModuleResolver {
    modules: RefCell<HashMap<String, Rc<Module>>>,
    fs: Rc<OverlayFS>,
}

impl TetronModuleResolver {
    pub fn new(fs: Rc<OverlayFS>) -> Self {
        Self {
            fs,
            modules: RefCell::new(HashMap::new()),
        }
    }

    // Helper to get a cached module, propagating borrow errors as module errors
    fn get_cached(
        &self,
        path: &str,
        pos: rhai::Position,
    ) -> Result<Option<Rc<Module>>, Box<EvalAltResult>> {
        Ok(self
            .modules
            .try_borrow()
            .map_err(|e| {
                TetronError::ModuleNotFound(format!("Error getting cached module: {e}"), pos)
            })?
            .get(path)
            .cloned())
    }

    // Helper to insert a module into the cache
    fn cache_module(
        &self,
        path: &str,
        module: Rc<Module>,
        pos: rhai::Position,
    ) -> Result<(), Box<EvalAltResult>> {
        self.modules
            .try_borrow_mut()
            .map_err(|e| {
                TetronError::ModuleNotFound(format!("Error mutably borrowing cache: {e}"), pos)
            })?
            .insert(path.to_string(), module);
        Ok(())
    }

    pub fn register_native_module(
        &self,
        name: &str,
        module: Rc<Module>,
    ) -> Result<(), TetronError> {
        self.modules
            .try_borrow_mut()
            .map_err(|e| format!("Error registering native module: {e}"))?
            .insert(name.into(), module);

        Ok(())
    }
}

impl ModuleResolver for TetronModuleResolver {
    fn resolve(
        &self,
        engine: &Engine,
        _: Option<&str>,
        path: &str,
        pos: rhai::Position,
    ) -> Result<Rc<Module>, Box<EvalAltResult>> {
        // Ensure non-empty path
        let first_char = path.chars().next().ok_or_else(|| {
            TetronError::ModuleNotFound("Invalid import: no module specified".into(), pos)
        })?;

        match first_char {
            '/' => {
                let path = normalize_path(path);

                if let Some(module) = self.get_cached(&path, pos)? {
                    return Ok(module);
                }

                let contents = self.fs.read_text_file(&path).map_err(|e| {
                    TetronError::ModuleNotFound(format!("Error reading file: {e}"), pos)
                })?;

                let ast = engine.compile(&contents).map_err(|e| {
                    TetronError::ModuleNotFound(format!("Error parsing module: {e}"), pos)
                })?;

                let module = Rc::new(Module::eval_ast_as_new(Scope::new(), &ast, engine)?);
                self.cache_module(&path, module.clone(), pos)?;

                Ok(module)
            }
            // Native global module
            'a'..='z' if !path.contains('/') => self.get_cached(path, pos)?.ok_or_else(|| {
                TetronError::ModuleNotFound("Invalid import: Unknown global module".into(), pos)
                    .into()
            }),
            // Malformed global module?
            'a'..='z' => Err(TetronError::ModuleNotFound(
                "Invalid import: global module imports should not contain '/'".into(),
                pos,
            )
            .into()),
            // All other cases
            _ => Err(TetronError::ModuleNotFound(
                "Invalid import: import paths must be absolute (start with '/') or global module names".into(),
                pos,
            ).into()),
        }
    }
}
