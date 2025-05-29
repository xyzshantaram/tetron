use std::collections::HashSet;
use std::sync::Arc;

use crate::error::TetronError;
use rune::alloc::String as RuneString;
use rune::{Value, runtime::Object};

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
    pub(crate) config: Object,
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
    pub fn create(&self, config: Object) -> Result<Behaviour, TetronError> {
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

        Ok(Behaviour {
            name,
            config,
            fields: self.keys.clone(),
        })
    }
}

impl Behaviour {
    fn check_field(&self, field: &str) -> Result<(), TetronError> {
        if !self.fields.contains(field) {
            Err(BehaviourError::InvalidProperty(field.to_owned()).into())
        } else {
            Ok(())
        }
    }

    #[rune::function(instance, protocol = SET, vm_result)]
    fn set_field(&mut self, field: &str, value: Value) -> Result<(), TetronError> {
        self.check_field(field)?;
        self.config.insert(RuneString::try_from(field)?, value).vm?;
        Ok(())
    }

    #[rune::function(instance, keep, protocol = GET, vm_result)]
    fn get_field(&self, field: &str) -> Result<Option<Value>, TetronError> {
        self.check_field(field)?;
        Ok(self.config.get(field).cloned())
    }

    #[rune::function]
    fn name(&self) -> String {
        self.name.clone()
    }
}
