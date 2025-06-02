use crate::{
    error::TetronError,
    utils::{Registrable, RuneString},
};
use rune::{ContextError, Module, Value, runtime::Object};
use std::{cell::RefCell, collections::HashSet, rc::Rc, sync::Arc};

enum BehaviourError {
    InvalidProperty(String),
}

impl From<BehaviourError> for TetronError {
    fn from(value: BehaviourError) -> Self {
        match value {
            BehaviourError::InvalidProperty(name) => TetronError::Runtime(format!(
                "tetron::behaviours: attempt to access invalid property {name}"
            )),
        }
    }
}

#[derive(rune::Any, Debug)]
pub struct Behaviour {
    pub(crate) name: String,
    #[allow(dead_code)] // used in impl Behaviour
    pub(crate) config: Object,
    #[allow(dead_code)] // used in impl Behaviour
    pub(crate) fields: Arc<HashSet<String>>,
}

#[derive(rune::Any, Clone, Debug)]
pub struct BehaviourFactory {
    name: String,
    keys: Arc<HashSet<String>>,
    internal: bool,
}

impl BehaviourFactory {
    pub fn new(name: &str, keys: HashSet<String>, internal: bool) -> Self {
        Self {
            name: name.to_owned(),
            keys: Arc::new(keys),
            internal,
        }
    }

    #[rune::function(keep)]
    pub fn create(&self, config: Object) -> Result<BehaviourRef, TetronError> {
        for name in config.keys() {
            if !self.keys.contains(name.as_str()) {
                return Err(BehaviourError::InvalidProperty(name.to_string()).into());
            }
        }

        let name = if self.internal {
            String::from("tetron:") + &self.name
        } else {
            self.name.clone()
        };

        Ok(BehaviourRef::new(Behaviour {
            name,
            config,
            fields: self.keys.clone(),
        }))
    }
}

impl Behaviour {
    #[allow(dead_code)] // used on the Rune side
    fn check_field(&self, field: &str) -> Result<(), TetronError> {
        if !self.fields.contains(field) {
            Err(BehaviourError::InvalidProperty(field.to_owned()).into())
        } else {
            Ok(())
        }
    }

    #[allow(dead_code)]
    fn set(&mut self, field: &str, value: Value) -> Result<(), TetronError> {
        self.check_field(field)?;
        self.config.insert(RuneString::try_from(field)?, value)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn get(&self, field: &str) -> Result<Option<Value>, TetronError> {
        self.check_field(field)?;
        Ok(self.config.get(field).cloned())
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(rune::Any, Debug, Clone)]
pub struct BehaviourRef(Rc<RefCell<Behaviour>>);

impl Registrable for BehaviourRef {
    fn register(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<BehaviourRef>()?;
        module.function_meta(BehaviourRef::name__meta)?;
        module.function_meta(BehaviourRef::set__meta)?;
        module.function_meta(BehaviourRef::get__meta)?;
        Ok(())
    }
}

impl Registrable for BehaviourFactory {
    fn register(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<BehaviourFactory>()?;
        module.function_meta(BehaviourFactory::create__meta)?;
        Ok(())
    }
}

impl BehaviourRef {
    fn new(behaviour: Behaviour) -> Self {
        Self(Rc::new(RefCell::new(behaviour)))
    }

    #[rune::function(keep)]
    pub fn name(&self) -> Result<String, TetronError> {
        Ok(self.0.try_borrow()?.name())
    }

    #[rune::function(instance, keep, protocol = SET)]
    pub fn set(&mut self, field: &str, value: Value) -> Result<(), TetronError> {
        self.0.try_borrow_mut()?.set(field, value)
    }

    #[rune::function(instance, keep, protocol = GET)]
    pub fn get(&self, field: &str) -> Result<Option<Value>, TetronError> {
        self.0.try_borrow()?.get(field)
    }
}
