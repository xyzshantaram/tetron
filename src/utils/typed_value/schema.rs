use super::TypedValue;
use crate::utils::Registrable;
use rune::{ContextError, Module, Value};
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Clone, rune::Any)]
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

fn format_typed_value_for_display(value: &TypedValue) -> String {
    format!("{:?}", value) // Assumes TypedValue has a Debug implementation
}

// Custom Debug implementation for Schema (one-line)
impl fmt::Debug for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Schema::Null => write!(f, "Schema::Null"),
            Schema::Bool => write!(f, "Schema::Bool"),
            Schema::Number => write!(f, "Schema::Number"),
            Schema::String => write!(f, "Schema::String"),
            Schema::Vec2 => write!(f, "Schema::Vec2"),
            Schema::Array { item, min, max } => f
                .debug_struct("Schema::Array")
                .field("item", item)
                .field("min", min)
                .field("max", max)
                .finish(),
            Schema::Object { fields } => {
                // Collect and sort keys for stable debug output
                let mut keys: Vec<&String> = fields.keys().collect();
                keys.sort();
                f.debug_struct("Schema::Object")
                    .field("keys", &keys)
                    .finish()
            }
            Schema::Optional(schema) => f.debug_tuple("Schema::Optional").field(schema).finish(),
            Schema::Default { schema, default } => {
                f.debug_struct("Schema::Default")
                    .field("schema", schema)
                    .field("default", default) // Relies on TypedValue's Debug
                    .finish()
            }
        }
    }
}

// Pretty-printed Display implementation for Schema
impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Initial call to the recursive helper
        self.fmt_recursive(f, 0, "")
    }
}

impl Schema {
    // Recursive helper for pretty-printing Schema
    // This method is part of the Schema impl block
    fn fmt_recursive(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent_level: usize,
        suffix: &str,
    ) -> fmt::Result {
        let leading_spaces = "  ".repeat(indent_level);
        match self {
            Schema::Null => write!(f, "{}Null{}", leading_spaces, suffix),
            Schema::Bool => write!(f, "{}Bool{}", leading_spaces, suffix),
            Schema::Number => write!(f, "{}Number{}", leading_spaces, suffix),
            Schema::String => write!(f, "{}String{}", leading_spaces, suffix),
            Schema::Vec2 => write!(f, "{}Vec2{}", leading_spaces, suffix),
            Schema::Array { item, min, max } => {
                let mut constraints = Vec::new();
                if let Some(m) = min {
                    constraints.push(format!("min items: {}", m));
                }
                if let Some(m) = max {
                    constraints.push(format!("max items: {}", m));
                }
                let constraints_str = if !constraints.is_empty() {
                    format!(" ({})", constraints.join(", "))
                } else {
                    String::new()
                };

                // For Array, "Array of:" is on one line, then the item schema is indented on the next.
                writeln!(
                    f,
                    "{}Array of:{}{}",
                    leading_spaces, constraints_str, suffix
                )?;
                item.fmt_recursive(f, indent_level + 1, "") // Item schema starts on a new indented line
            }
            Schema::Object { fields } => {
                if fields.is_empty() {
                    writeln!(f, "{}Object (empty){}", leading_spaces, suffix)?;
                } else {
                    writeln!(f, "{}Object with fields:{}", leading_spaces, suffix)?;
                    let mut sorted_fields: Vec<_> = fields.iter().collect();
                    sorted_fields.sort_by_key(|(k, _)| *k); // Sort for stable output

                    for (i, (key, field_schema)) in sorted_fields.iter().enumerate() {
                        let field_indent_level = indent_level + 1;
                        let field_leading_spaces = "  ".repeat(field_indent_level);

                        let mut field_attributes = Vec::new();
                        if field_schema.optional {
                            field_attributes.push("Optional".into());
                        }
                        if let Some(def_val) = &field_schema.default {
                            field_attributes.push(format!(
                                "Default: {}",
                                format_typed_value_for_display(def_val)
                            ));
                        }
                        let attributes_suffix = if !field_attributes.is_empty() {
                            format!(" ({})", field_attributes.join(", "))
                        } else {
                            String::new()
                        };

                        // Field name part a
                        write!(f, "{}\"{}\": ", field_leading_spaces, key)?;
                        // The schema for the field, potentially with attributes suffix.
                        // It will decide if it needs a newline (e.g. if it's an Array or Object).
                        field_schema
                            .schema
                            .fmt_recursive(f, 0, &attributes_suffix)?; // Start with 0 relative indent for the type itself

                        // Add a newline after each field, unless it's the last one and the parent won't add one.
                        // The typical structure is that writeln! is used by containers, so simple types don't need to add their own.
                        // The loop for object fields should ensure each field ends on its own line.
                        if i < sorted_fields.len() - 1
                            || matches!(
                                field_schema.schema,
                                Schema::Object { .. }
                                    | Schema::Array { .. }
                                    | Schema::Optional(_)
                                    | Schema::Default { .. }
                            )
                        {
                            // If the field schema itself was a container, it already added a newline.
                            // If it was a simple type, we add the newline here.
                            if !matches!(
                                field_schema.schema,
                                Schema::Object { .. }
                                    | Schema::Array { .. }
                                    | Schema::Optional(_)
                                    | Schema::Default { .. }
                            ) {
                                writeln!(f)?;
                            }
                        } else {
                            // Last simple item, no newline, parent handles it. Or it was a container that handled its own.
                            // Actually, for consistency, always writeln after a field's schema has been printed.
                            writeln!(f)?;
                        }
                    }
                }
                Ok(())
            }
            Schema::Optional(sub_schema) => {
                writeln!(f, "{}Optional:{}", leading_spaces, suffix)?;
                sub_schema.fmt_recursive(f, indent_level + 1, "")
            }
            Schema::Default {
                schema: sch,
                default,
            } => {
                writeln!(
                    f,
                    "{}Default (value: {}):{}",
                    leading_spaces,
                    format_typed_value_for_display(default),
                    suffix
                )?;
                sch.fmt_recursive(f, indent_level + 1, "")
            }
        }
    }
}

impl Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaError::TypeMismatch { expected, found } => {
                write!(
                    f,
                    "Type mismatch: expected '{}', found '{}'",
                    expected, found
                )
            }
            SchemaError::MissingField(field) => write!(f, "Missing field: '{}'", field),
            SchemaError::ArrayMinViolation { min, found } => {
                write!(
                    f,
                    "Array length violation: minimum is {}, found {}",
                    min, found
                )
            }
            SchemaError::ArrayMaxViolation { max, found } => {
                write!(
                    f,
                    "Array length violation: maximum is {}, found {}",
                    max, found
                )
            }
            SchemaError::Validation(msg) => write!(f, "Validation error: {}", msg),
        }
    }
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
