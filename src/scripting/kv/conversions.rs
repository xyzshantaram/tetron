use rhai::{Dynamic, EvalAltResult};
use stupid_simple_kv::{KvKey, KvValue};

use crate::TetronError;

pub fn from_kv_value(value: &KvValue) -> Dynamic {
    use rhai::{Array, Dynamic, Map};

    match value {
        KvValue::Null => Dynamic::UNIT,
        KvValue::Bool(b) => Dynamic::from_bool(*b),
        KvValue::I64(i) => Dynamic::from_int(*i),
        KvValue::F64(f) => Dynamic::from_float(*f),
        KvValue::String(s) => Dynamic::from(s.clone()),
        KvValue::Array(arr) => {
            let vec: Array = arr.iter().map(from_kv_value).collect();
            Dynamic::from_array(vec)
        }
        KvValue::Object(obj) => {
            let mut map = Map::new();
            for (k, v) in obj {
                map.insert(k.into(), from_kv_value(v));
            }
            Dynamic::from_map(map)
        }
        KvValue::Binary(bytes) => Dynamic::from_blob(bytes.clone()),
    }
}

pub fn to_kv_value(value: &Dynamic) -> Result<KvValue, String> {
    use rhai::{Array, Map};
    use std::collections::BTreeMap;

    Ok(match value {
        v if v.is_unit() => KvValue::Null,
        v if v.is_bool() => KvValue::Bool(v.as_bool().unwrap()),
        v if v.is::<i64>() => KvValue::I64(v.as_int().unwrap()),
        v if v.is::<f64>() => KvValue::F64(v.as_float().unwrap()),
        v if v.is_char() => {
            let c = v.clone_cast::<char>();
            KvValue::String(c.to_string())
        }
        v if v.is_string() => KvValue::String(v.to_string()),
        v if v.is_array() => {
            let arr = v.clone_cast::<Array>();
            let mut res = Vec::with_capacity(arr.len());
            for elem in arr {
                res.push(to_kv_value(&elem)?);
            }
            KvValue::Array(res)
        }
        v if v.is_map() => {
            let map = v.clone_cast::<Map>();
            let mut out = BTreeMap::new();
            for (k, v) in map {
                out.insert(k.to_string(), to_kv_value(&v)?);
            }
            KvValue::Object(out)
        }
        v if v.is_blob() => {
            let blob = v.as_blob_ref()?;
            KvValue::Binary(blob.clone())
        }
        _ => {
            return Err("Unsupported Rhai type for KvValue".to_string());
        }
    })
}

pub fn rhai_dyn_to_kvkey(value: Dynamic) -> Result<KvKey, Box<EvalAltResult>> {
    let arr = value.as_array_ref().map_err(|e| {
        TetronError::RhaiRuntime(format!("set_flag: Expected array, got: {e}").into(), None)
    })?;

    if arr.len() > 16 {
        return Err(TetronError::RhaiRuntime(
            "set_flag: Keys must be a maximum of 16 parts long".into(),
            None,
        )
        .into());
    }

    if arr
        .iter()
        .any(|e| !e.is_bool() && !e.is_string() && !e.is_char() && !e.is_int())
    {
        return Err(TetronError::RhaiRuntime(
            "set_flag: all key members must be string/bool/int".into(),
            None,
        )
        .into());
    }

    let mut key = KvKey::new();
    for i in arr.iter() {
        if i.is_bool() {
            let v = i.as_bool()?;
            key.push(&v);
        } else if i.is_int() {
            let v = i.as_int()?;
            key.push(&v);
        } else if i.is_string() {
            let v: String = i.clone_cast::<String>();
            key.push(&v);
        } else {
            unreachable!();
        }
    }

    Ok(key)
}
