use std::{collections::HashMap, convert::Infallible};

use crate::{
    engine::physics::vec2::Vec2,
    error::TetronError,
    utils::{RuneString, RuneVec},
};
use rune::{FromValue, ToValue, TypeHash, Value, alloc::clone::TryClone, runtime::Object};

#[derive(Debug, Clone)]
pub enum TypedValue {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<TypedValue>),
    Object(HashMap<String, TypedValue>),
    Vector(Vec2),
}

impl TryFrom<&Value> for TypedValue {
    type Error = TetronError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.type_hash() {
            bool::HASH => Ok(Self::Bool(value.as_bool()?)),
            u64::HASH | i64::HASH | f64::HASH => Ok(Self::Number(value.as_float()?)),
            String::HASH => Ok(Self::String(value.try_clone()?.into_string()?.into_std())),
            Vec2::HASH => Ok(Self::Vector(Vec2::from_value(value.try_clone()?)?)),
            Object::HASH => Ok(TypedValue::Object({
                let mut map = HashMap::<String, TypedValue>::new();
                for (key, value) in Object::from_value(value.try_clone()?)? {
                    map.insert(key.into_std(), TryInto::try_into(&value)?);
                }
                map
            })),
            RuneVec::HASH => Ok(TypedValue::Array({
                let mut vec: Vec<TypedValue> = Vec::new();
                for value in value.borrow_ref::<RuneVec>()?.iter() {
                    vec.push(value.try_into()?)
                }
                vec
            })),
            _ => Err(TetronError::Runtime(format!(
                "Could not convert value {value:?} into BehaviourValue"
            ))),
        }
    }
}

impl TryFrom<&TypedValue> for Value {
    type Error = TetronError;

    fn try_from(value: &TypedValue) -> Result<Self, Self::Error> {
        match value {
            TypedValue::String(s) => Ok(s.to_owned().to_value()?),
            TypedValue::Number(f) => Ok(f.to_value()?),
            TypedValue::Bool(b) => Ok(b.to_value()?),
            TypedValue::Array(values) => {
                let mut vec = RuneVec::new();
                for value in values {
                    vec.push(value.try_into()?)?;
                }

                Ok(vec.to_value()?)
            }
            TypedValue::Object(map) => {
                let mut obj = Object::new();
                for (key, value) in map {
                    obj.insert(RuneString::try_from(key.as_str())?, value.try_into()?);
                }

                Ok(obj.to_value()?)
            }
            TypedValue::Vector(v) => Ok(v.to_value()?),
        }
    }
}

impl TryFrom<Value> for TypedValue {
    type Error = TetronError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<TypedValue> for Value {
    type Error = TetronError;

    fn try_from(value: TypedValue) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

// TryFrom implementations for basic types to TypedValue
impl TryFrom<Vec<TypedValue>> for TypedValue {
    type Error = Infallible;

    fn try_from(value: Vec<TypedValue>) -> Result<Self, Self::Error> {
        Ok(TypedValue::Array(value))
    }
}

impl TryFrom<String> for TypedValue {
    type Error = Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(TypedValue::String(value))
    }
}

impl TryFrom<f64> for TypedValue {
    type Error = Infallible;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Ok(TypedValue::Number(value))
    }
}

impl TryFrom<bool> for TypedValue {
    type Error = Infallible;

    fn try_from(value: bool) -> Result<Self, Self::Error> {
        Ok(TypedValue::Bool(value))
    }
}

impl TryFrom<i64> for TypedValue {
    type Error = Infallible;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(TypedValue::Number(value as f64))
    }
}

impl TryFrom<u64> for TypedValue {
    type Error = Infallible;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(TypedValue::Number(value as f64))
    }
}

impl TryFrom<HashMap<String, TypedValue>> for TypedValue {
    type Error = Infallible;

    fn try_from(value: HashMap<String, TypedValue>) -> Result<Self, Self::Error> {
        Ok(TypedValue::Object(value))
    }
}

impl TryFrom<Vec2> for TypedValue {
    type Error = Infallible;

    fn try_from(value: Vec2) -> Result<Self, Self::Error> {
        Ok(TypedValue::Vector(value))
    }
}

impl TryFrom<TypedValue> for Vec<TypedValue> {
    type Error = TetronError;

    fn try_from(value: TypedValue) -> Result<Self, Self::Error> {
        match value {
            TypedValue::Array(vec) => Ok(vec),
            _ => Err(TetronError::Runtime(
                "Cannot convert non-array TypedValue to Vec<TypedValue>".to_string(),
            )),
        }
    }
}

impl TryFrom<TypedValue> for String {
    type Error = TetronError;

    fn try_from(value: TypedValue) -> Result<Self, Self::Error> {
        match value {
            TypedValue::String(s) => Ok(s),
            _ => Err(TetronError::Runtime(
                "Cannot convert non-string TypedValue to String".to_string(),
            )),
        }
    }
}

impl TryFrom<TypedValue> for f64 {
    type Error = TetronError;

    fn try_from(value: TypedValue) -> Result<Self, Self::Error> {
        match value {
            TypedValue::Number(n) => Ok(n),
            _ => Err(TetronError::Runtime(
                "Cannot convert non-number TypedValue to f64".to_string(),
            )),
        }
    }
}

impl TryFrom<TypedValue> for bool {
    type Error = TetronError;

    fn try_from(value: TypedValue) -> Result<Self, Self::Error> {
        match value {
            TypedValue::Bool(b) => Ok(b),
            _ => Err(TetronError::Runtime(
                "Cannot convert non-bool TypedValue to bool".to_string(),
            )),
        }
    }
}

impl TryFrom<TypedValue> for i64 {
    type Error = TetronError;

    fn try_from(value: TypedValue) -> Result<Self, Self::Error> {
        match value {
            TypedValue::Number(n) => Ok(n as i64),
            _ => Err(TetronError::Runtime(
                "Cannot convert non-number TypedValue to i64".to_string(),
            )),
        }
    }
}

impl TryFrom<TypedValue> for u64 {
    type Error = TetronError;

    fn try_from(value: TypedValue) -> Result<Self, Self::Error> {
        match value {
            TypedValue::Number(n) => {
                if n >= 0.0 {
                    Ok(n as u64)
                } else {
                    Err(TetronError::Runtime(
                        "Cannot convert negative number to u64".to_string(),
                    ))
                }
            }
            _ => Err(TetronError::Runtime(
                "Cannot convert non-number TypedValue to u64".to_string(),
            )),
        }
    }
}

impl TryFrom<TypedValue> for HashMap<String, TypedValue> {
    type Error = TetronError;

    fn try_from(value: TypedValue) -> Result<Self, Self::Error> {
        match value {
            TypedValue::Object(map) => Ok(map),
            _ => Err(TetronError::Runtime(
                "Cannot convert non-object TypedValue to HashMap<String, TypedValue>".to_string(),
            )),
        }
    }
}

impl TryFrom<TypedValue> for Vec2 {
    type Error = TetronError;

    fn try_from(value: TypedValue) -> Result<Self, Self::Error> {
        match value {
            TypedValue::Vector(vec) => Ok(vec),
            _ => Err(TetronError::Runtime(
                "Cannot convert non-vector TypedValue to Vec2".to_string(),
            )),
        }
    }
}
