use std::collections::BTreeMap;

use crate::TetronError;
use rune::{FromValue, ToValue, TypeHash, Value, alloc::String as RuneString, runtime::Object};
use stupid_simple_kv::{KvKey, KvValue};

pub fn rune_vec_to_kv_key(value: Vec<Value>) -> Result<KvKey, TetronError> {
    let mut key = KvKey::new();
    for item in value {
        match item.type_hash() {
            bool::HASH => {
                key.push(&item.as_bool()?);
            }
            i64::HASH => {
                key.push(&item.as_integer::<i64>()?);
            }
            RuneString::HASH => key.push(&String::from_value(item)?),
            _ => {
                return Err(TetronError::Conversion(format!(
                    "Invalid type for key part {item:#?}"
                )));
            }
        }
    }

    Ok(key)
}

pub fn rune_value_to_kv(value: Value) -> Result<KvValue, TetronError> {
    match value.type_hash() {
        <()>::HASH => Ok(KvValue::Null),
        bool::HASH => Ok(KvValue::Bool(value.as_bool()?)),
        i64::HASH | u64::HASH => Ok(KvValue::I64(value.as_integer::<i64>()?)),
        f64::HASH => {
            let f = f64::from_value(value)?;
            Ok(KvValue::F64(f))
        }
        RuneString::HASH => Ok(KvValue::String(String::from_value(value)?)),
        Vec::<Value>::HASH => {
            let mut values: Vec<KvValue> = Vec::new();
            for val in Vec::<Value>::from_value(value)? {
                values.push(rune_value_to_kv(val)?)
            }
            Ok(KvValue::Array(values))
        }
        Object::HASH => {
            let obj = Object::from_value(value)?;
            let mut map = BTreeMap::new();
            for (key, val) in obj {
                map.insert(key.into_std(), rune_value_to_kv(val)?);
            }
            Ok(KvValue::Object(map))
        }
        _ => Err(TetronError::Conversion(format!(
            "Unsupported value for kv operation: {value:#?}"
        ))),
    }
}

pub fn kv_value_to_rune(value: &KvValue) -> Result<Value, TetronError> {
    match value {
        KvValue::Null => Ok(Value::empty()),
        KvValue::Bool(b) => Ok(b.to_value()?),
        KvValue::I64(i) => Ok(i.to_value()?),
        KvValue::F64(f) => Ok(f.to_value()?),
        KvValue::String(s) => Ok(s.to_owned().to_value()?),
        KvValue::Array(arr) => {
            // recursively convert, then to rune::runtime::Vec, then to Value
            let mut rune_vec = Vec::with_capacity(arr.len());
            for elem in arr {
                rune_vec.push(kv_value_to_rune(elem)?);
            }
            Ok(rune_vec.to_value()?)
        }
        KvValue::Object(map) => {
            let mut obj = Object::new();
            for (k, v) in map {
                obj.insert_value(RuneString::try_from(k.to_owned())?, kv_value_to_rune(v)?)
                    .into_result()
                    .map_err(|e| TetronError::Runtime(e.to_string()))?;
            }
            Ok(obj.to_value()?)
        }

        KvValue::Binary(_) => Err(TetronError::Conversion(
            "Binary objects are not supported".into(),
        )),
    }
}
