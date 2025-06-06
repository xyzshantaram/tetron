use crate::{
    error::TetronError,
    system_log,
    utils::{
        Registrable,
        typed_value::{
            TypedValue,
            schema::{Schema, SchemaError},
        },
    },
};
use rune::{ContextError, Module, Value, runtime::Object};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

enum BehaviourError {
    InvalidProperty(String),
    Validation(SchemaError),
}

impl From<BehaviourError> for TetronError {
    fn from(value: BehaviourError) -> Self {
        match value {
            BehaviourError::InvalidProperty(name) => TetronError::Runtime(format!(
                "tetron::behaviours: attempt to access invalid property {name}"
            )),
            BehaviourError::Validation(e) => {
                TetronError::Runtime(format!("validation failed: {e:?}"))
            }
        }
    }
}

#[derive(rune::Any, Debug)]
pub struct Behaviour {
    pub(crate) name: String,
    #[allow(dead_code)] // used in impl Behaviour
    pub(crate) config: HashMap<String, TypedValue>,
    pub(crate) schema: Arc<Schema>,
}

#[derive(rune::Any, Clone, Debug)]
pub struct BehaviourFactory {
    name: String,
    schema: Arc<Schema>,
    internal: bool,
}

impl BehaviourFactory {
    pub fn new(name: &str, schema: Schema, internal: bool) -> Self {
        Self {
            name: name.to_owned(),
            schema: Arc::new(schema),
            internal,
        }
    }

    pub fn with_map(&self, map: HashMap<String, TypedValue>) -> Result<BehaviourRef, TetronError> {
        let validated = self
            .schema
            .validate(&TypedValue::Object(map))
            .inspect_err(|e| system_log!("BehaviourFactory::with_map validation error: {e:?}"))
            .map_err(BehaviourError::Validation)?;
        let name = if self.internal {
            String::from("tetron:") + &self.name
        } else {
            self.name.clone()
        };
        let config = match validated {
            TypedValue::Object(obj) => obj,
            _ => unreachable!(),
        };
        Ok(BehaviourRef::new(Behaviour {
            name,
            config,
            schema: self.schema.clone(),
        }))
    }

    #[rune::function(keep)]
    pub fn create(&self, config: &Object) -> Result<BehaviourRef, TetronError> {
        let mut map = HashMap::<String, TypedValue>::new();
        for key in config.keys() {
            if let Some(val) = config.get(key) {
                map.insert(
                    key.as_str().to_string(),
                    val.try_into().inspect_err(|e| {
                        system_log!("BehaviourFactory::create (key {key}) error: {e:?}")
                    })?,
                );
            }
        }
        self.with_map(map)
            .inspect_err(|e| system_log!("BehaviourFactory::create -> with_map error: {e:?}"))
    }

    pub fn schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }
}

impl Behaviour {
    #[allow(dead_code)] // used on the Rune side
    fn check_field(&self, field: &str) -> Result<(), TetronError> {
        match *self.schema {
            Schema::Object { ref fields } => {
                if !fields.contains_key(field) {
                    Err(BehaviourError::InvalidProperty(field.to_owned()).into())
                } else {
                    Ok(())
                }
            }
            _ => Err(TetronError::Runtime(
                "Behaviour schema is not object".to_string(),
            )),
        }
    }

    #[allow(dead_code)]
    fn set(&mut self, field: &str, value: Value) -> Result<(), TetronError> {
        self.check_field(field)
            .inspect_err(|e| system_log!("Behaviour::set check_field error: {e:?}"))?;
        self.config.insert(
            field.into(),
            TryInto::try_into(&value)
                .inspect_err(|e| system_log!("Behaviour::set TryInto error: {e:?}"))?,
        );
        Ok(())
    }

    #[allow(dead_code)]
    fn get(&self, field: &str) -> Result<Option<Value>, TetronError> {
        self.check_field(field)
            .inspect_err(|e| system_log!("Behaviour::get check_field error: {e:?}"))?;
        if let Some(val) = self.config.get(field) {
            Ok(Some(val.try_into().inspect_err(|e| {
                system_log!("Behaviour::get TryInto error: {e:?} (field: {field})")
            })?))
        } else {
            Ok(None)
        }
    }

    fn get_typed(&self, field: &str) -> Result<Option<TypedValue>, TetronError> {
        self.check_field(field)
            .inspect_err(|e| system_log!("Behaviour::get_typed check_field error: {e:?}"))?;
        if let Some(val) = self.config.get(field) {
            Ok(Some(val.to_owned()))
        } else {
            Ok(None)
        }
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    pub fn schema(&self) -> Arc<Schema> {
        self.schema.clone()
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
    pub fn name(&self) -> String {
        self.0.borrow().name()
    }

    #[rune::function(instance, keep, protocol = SET)]
    pub fn set(&mut self, field: &str, value: Value) -> Result<(), TetronError> {
        self.0.borrow_mut().set(field, value)
    }

    #[rune::function(instance, keep, protocol = GET)]
    pub fn get(&self, field: &str) -> Result<Option<Value>, TetronError> {
        self.0.borrow().get(field)
    }

    pub fn has(&self, field: &str) -> bool {
        self.0.borrow().config.contains_key(field)
    }

    pub fn get_typed(&self, field: &str) -> Result<Option<TypedValue>, TetronError> {
        self.0.borrow().get_typed(field)
    }
}
