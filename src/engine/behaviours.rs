use crate::{
    log_and_die,
    utils::{
        Registrable,
        typed_value::{TypedValue, schema::Schema},
    },
};
use rune::{ContextError, Module, Value, runtime::Object};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

#[derive(rune::Any, Debug)]
pub struct Behaviour {
    pub(crate) name: String,
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

    pub fn with_map(&self, map: HashMap<String, TypedValue>) -> BehaviourRef {
        if let Ok(validated) = self.schema.validate(&TypedValue::Object(map.clone())) {
            let name = if self.internal {
                String::from("tetron:") + &self.name
            } else {
                self.name.clone()
            };
            let config = match validated {
                TypedValue::Object(obj) => obj,
                _ => unreachable!(),
            };
            BehaviourRef::new(Behaviour {
                name,
                config,
                schema: self.schema.clone(),
            })
        } else {
            log_and_die!(
                1,
                "Could not validate {map:?} against schema {:?}",
                self.schema
            )
        }
    }

    #[rune::function(keep)]
    pub fn create(&self, config: &Object) -> BehaviourRef {
        let mut map = HashMap::<String, TypedValue>::new();
        for key in config.keys() {
            if let Some(val) = config.get(key) {
                map.insert(
                    key.as_str().to_string(),
                    val.try_into()
                        .expect("Engine bug: failed to convert rune value to typed value"),
                );
            }
        }
        self.with_map(map)
    }

    pub fn schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }
}

impl Behaviour {
    fn check_field(&self, field: &str) {
        match *self.schema {
            Schema::Object { ref fields } => {
                if !fields.contains_key(field) {
                    log_and_die!(1, "Invalid field {field} accessed on behaviour")
                }
            }
            _ => log_and_die!(
                1,
                "Engine bug: Behaviour schema is not object! This should never happen."
            ),
        }
    }

    fn set(&mut self, field: &str, value: Value) {
        self.check_field(field);
        self.config.insert(
            field.into(),
            TryInto::try_into(&value)
                .expect("engine bug: could not convert rune Value into TypedValue"),
        );
    }

    fn get(&self, field: &str) -> Option<Value> {
        self.config.get(field).map(|val| {
            val.try_into().unwrap_or_else(|_| {
                panic!(
                    "Could not convert value of {field} on behaviour {} ",
                    self.name
                )
            })
        })
    }

    fn get_typed(&self, field: &str) -> Option<TypedValue> {
        self.check_field(field);
        self.config.get(field).cloned()
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
    pub fn set(&mut self, field: &str, value: Value) {
        self.0.borrow_mut().set(field, value);
    }

    #[rune::function(instance, keep, protocol = GET)]
    pub fn get(&self, field: &str) -> Option<Value> {
        self.0.borrow().get(field)
    }

    pub fn has(&self, field: &str) -> bool {
        self.0.borrow().config.contains_key(field)
    }

    pub fn get_typed(&self, field: &str) -> Option<TypedValue> {
        self.0.borrow().get_typed(field)
    }
}
