use super::TypedValue;
use crate::utils::Registrable;
use rune::{ContextError, Module, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, rune::Any)]
pub enum Schema {
    Null,
    Bool,
    Number,
    String,
    Vec2,
    Array {
        item: Box<Schema>,
        min: Option<usize>,
        max: Option<usize>,
    },
    Object {
        fields: HashMap<String, SchemaField>,
    },
    Optional(Box<Schema>),
    Default {
        schema: Box<Schema>,
        default: TypedValue,
    },
}

#[derive(Debug, Clone, rune::Any)]
pub struct SchemaField {
    pub schema: Schema,
    pub optional: bool,
    pub default: Option<TypedValue>,
}

#[derive(Debug, Clone, rune::Any)]
pub struct ObjectBuilder {
    fields: Vec<(String, SchemaField)>,
}

impl Schema {
    #[rune::function(keep, path = Schema::string)]
    pub fn string() -> Self {
        Schema::String
    }
    #[rune::function(keep, path = Schema::number)]
    pub fn number() -> Self {
        Schema::Number
    }
    #[rune::function(keep, path = Schema::bool)]
    pub fn bool() -> Self {
        Schema::Bool
    }
    #[rune::function(keep, path = Schema::vec2)]
    pub fn vec2() -> Self {
        Schema::Vec2
    }
    #[rune::function(keep, path = Schema::object)]
    pub fn object() -> ObjectBuilder {
        ObjectBuilder { fields: Vec::new() }
    }
    #[rune::function(keep, path = Schema::array)]
    pub fn array(item: Schema) -> Self {
        Schema::Array {
            item: Box::new(item),
            min: None,
            max: None,
        }
    }
    #[rune::function(instance, keep)]
    pub fn min(&self, n: usize) -> Self {
        let mut new = self.clone();
        if let Schema::Array { ref mut min, .. } = new {
            *min = Some(n);
        }
        new
    }
    #[rune::function(instance, keep)]
    pub fn max(&self, n: usize) -> Self {
        let mut new = self.clone();
        if let Schema::Array { ref mut max, .. } = new {
            *max = Some(n);
        }
        new
    }
    #[rune::function(instance, keep)]
    pub fn optional(&self) -> Self {
        Schema::Optional(Box::new(self.clone()))
    }

    pub fn default(&self, default: TypedValue) -> Self {
        Schema::Default {
            schema: Box::new(self.clone()),
            default,
        }
    }
    // Rune-visible, accepts Value/default Value, does the conversion
    #[rune::function(instance, keep, path = Self::default)]
    pub fn default_rune(&self, default: Value) -> Self {
        let def: TypedValue = match default.try_into() {
            Ok(val) => val,
            Err(_) => return self.clone(),
        };
        self.default(def)
    }
    // Internal validation
    pub fn validate(&self, value: &TypedValue) -> Result<TypedValue, SchemaError> {
        match (self, value) {
            (Schema::String, TypedValue::String(_)) => Ok(value.clone()),
            (Schema::Number, TypedValue::Number(_)) => Ok(value.clone()),
            (Schema::Bool, TypedValue::Bool(_)) => Ok(value.clone()),
            (Schema::Vec2, TypedValue::Vector(_)) => Ok(value.clone()),
            (Schema::Null, TypedValue::Array(_)) => Err(SchemaError::TypeMismatch {
                expected: "Null".into(),
                found: "Array".into(),
            }),
            (Schema::Array { item, min, max }, TypedValue::Array(items)) => {
                if let Some(min) = min {
                    if items.len() < *min {
                        return Err(SchemaError::ArrayMinViolation {
                            min: *min,
                            found: items.len(),
                        });
                    }
                }
                if let Some(max) = max {
                    if items.len() > *max {
                        return Err(SchemaError::ArrayMaxViolation {
                            max: *max,
                            found: items.len(),
                        });
                    }
                }
                let mut validated = Vec::with_capacity(items.len());
                for item_val in items {
                    validated.push(item.validate(item_val)?);
                }
                Ok(TypedValue::Array(validated))
            }
            (Schema::Object { fields }, TypedValue::Object(obj)) => {
                let mut out = HashMap::new();
                for (key, field_schema) in fields {
                    match obj.get(key) {
                        Some(v) => {
                            out.insert(key.clone(), field_schema.schema.validate(v)?);
                        }
                        None => {
                            if field_schema.optional {
                                if let Some(default) = &field_schema.default {
                                    out.insert(key.clone(), default.clone());
                                }
                            } else {
                                return Err(SchemaError::MissingField(key.clone()));
                            }
                        }
                    }
                }
                Ok(TypedValue::Object(out))
            }
            (Schema::Optional(sub), v) => sub.validate(v),
            (Schema::Default { schema, default }, v) => match schema.validate(v) {
                Ok(valid) => Ok(valid),
                Err(_) => Ok(default.clone()),
            },
            (expected, found) => Err(SchemaError::TypeMismatch {
                expected: format!("{:?}", expected),
                found: format!("{:?}", found),
            }),
        }
    }
    #[rune::function(instance, keep, path = Schema::validate)]
    pub fn validate_rune(&self, value: Value) -> Result<Value, SchemaError> {
        let tv: TypedValue = value
            .try_into()
            .map_err(|e| SchemaError::Validation(format!("{e}")))?;
        let validated = self.validate(&tv)?;
        validated
            .try_into()
            .map_err(|e| SchemaError::Validation(format!("{e}")))
    }
}

impl ObjectBuilder {
    pub fn field(&self, name: &str, schema: Schema) -> Self {
        let mut clone = self.clone();
        clone.fields.push((
            name.into(),
            SchemaField {
                schema,
                optional: false,
                default: None,
            },
        ));
        clone
    }

    pub fn optional_field(&self, name: &str, schema: Schema, default: Option<TypedValue>) -> Self {
        let mut clone = self.clone();
        clone.fields.push((
            name.into(),
            SchemaField {
                schema,
                optional: true,
                default,
            },
        ));
        clone
    }

    #[rune::function(instance, path = ObjectBuilder::field)]
    pub fn field_rune(&self, name: &str, schema: Schema) -> Self {
        self.field(name, schema)
    }

    #[rune::function(instance, path = ObjectBuilder::optional_field)]
    pub fn optional_field_rune(&self, name: &str, schema: Schema, default: Option<Value>) -> Self {
        let default_tv = match default {
            Some(ref v) => v.clone().try_into().ok(),
            None => None,
        };
        self.optional_field(name, schema, default_tv)
    }

    #[rune::function(instance, keep)]
    pub fn build(&self) -> Schema {
        Schema::Object {
            fields: self.fields.clone().into_iter().collect(),
        }
    }
}

impl Registrable for Schema {
    fn register(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<Schema>()?;
        module.function_meta(Schema::string__meta)?;
        module.function_meta(Schema::number__meta)?;
        module.function_meta(Schema::bool__meta)?;
        module.function_meta(Schema::vec2__meta)?;
        module.function_meta(Schema::object__meta)?;
        module.function_meta(Schema::array__meta)?;
        module.function_meta(Schema::min__meta)?;
        module.function_meta(Schema::max__meta)?;
        module.function_meta(Schema::optional__meta)?;
        module.function_meta(Schema::default_rune__meta)?;
        module.function_meta(Schema::validate_rune__meta)?;
        Ok(())
    }
}
impl Registrable for SchemaField {
    fn register(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<SchemaField>()?;
        Ok(())
    }
}
impl Registrable for ObjectBuilder {
    fn register(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<ObjectBuilder>()?;
        module.function_meta(ObjectBuilder::field_rune)?;
        module.function_meta(ObjectBuilder::optional_field_rune)?;
        module.function_meta(ObjectBuilder::build__meta)?;
        Ok(())
    }
}
impl Registrable for SchemaError {
    fn register(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<SchemaError>()?;
        Ok(())
    }
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_crate_item("tetron", ["validation"])?;
    Schema::register(&mut m)?;
    SchemaField::register(&mut m)?;
    ObjectBuilder::register(&mut m)?;
    SchemaError::register(&mut m)?;
    Ok(m)
}

#[derive(Debug, Clone, PartialEq, rune::Any)]
pub enum SchemaError {
    TypeMismatch { expected: String, found: String },
    MissingField(String),
    ArrayMinViolation { min: usize, found: usize },
    ArrayMaxViolation { max: usize, found: usize },
    Validation(String),
}

#[cfg(test)]
mod tests {
    use crate::error::TetronError;

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_primitive_builders() {
        assert!(matches!(Schema::string(), Schema::String));
        assert!(matches!(Schema::number(), Schema::Number));
        assert!(matches!(Schema::bool(), Schema::Bool));
        assert!(matches!(Schema::vec2(), Schema::Vec2));
    }

    #[test]
    fn test_array_schema_min_max() {
        let arr = Schema::array(Schema::number()).min(2).max(5);
        if let Schema::Array { min, max, .. } = arr {
            assert_eq!(min, Some(2));
            assert_eq!(max, Some(5));
        } else {
            panic!("Should be array schema");
        }
    }

    #[test]
    fn test_object_builder_and_validate() -> Result<(), TetronError> {
        let schema = Schema::object()
            .field("n", Schema::number())
            .optional_field("msg", Schema::string(), Some("ok".into()))
            .build();

        let mut input = HashMap::new();
        input.insert("n".into(), TypedValue::Number(42.0));
        let validated = schema.validate(&TypedValue::Object(input)).unwrap();
        if let TypedValue::Object(obj) = validated {
            assert_eq!(obj["n"], TypedValue::Number(42.0));
            assert_eq!(obj["msg"], TypedValue::String("ok".into()));

            Ok(())
        } else {
            panic!("not object");
        }
    }

    #[test]
    fn test_validate_array_length() {
        let s = Schema::array(Schema::number()).min(2).max(3);
        assert!(
            s.validate(&TypedValue::Array(vec![TypedValue::Number(1.0)]))
                .is_err()
        );
        assert!(
            s.validate(&TypedValue::Array(vec![
                TypedValue::Number(1.0),
                TypedValue::Number(2.0)
            ]))
            .is_ok()
        );
        assert!(
            s.validate(&TypedValue::Array(vec![
                TypedValue::Number(1.0),
                TypedValue::Number(2.0),
                TypedValue::Number(3.0),
                TypedValue::Number(4.0)
            ]))
            .is_err()
        );
    }

    #[test]
    fn test_default_validation() {
        let schema = Schema::number().default(TypedValue::Number(7.0));
        // Supply nothing or wrong, get default
        let got = schema.validate(&TypedValue::String("bad".into())).unwrap();
        assert_eq!(got, TypedValue::Number(7.0));
    }
}
