use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use once_cell::sync::Lazy;
use serde_json::{json, Map, Number, Value};
use std::{
    borrow::Borrow,
    sync::{Arc, Mutex},
};

use crate::{device_driver::RefreshSignal, trigger_draw};

#[derive(Debug)]
struct State {
    root: Arc<Value>,
}

static STATE: Lazy<Mutex<State>> = Lazy::new(|| {
    Mutex::new(State {
        root: Arc::new(json!({})),
    })
});

#[derive(Debug)]
enum Error {
    InvalidJsonValue,
}

// Value2 replaces value1 in case of conflict
fn deep_merge(value1: &Value, value2: &Value) -> Value {
    match value2 {
        Value::Null => value2.clone(),
        Value::Bool(_) => value2.clone(),
        Value::Number(_) => value2.clone(),
        Value::String(_) => value2.clone(),
        Value::Array(arr2) => match value1 {
            Value::Array(arr1) => {
                let mut outvec = Vec::new();
                for i in 0..arr2.len() {
                    if i < arr1.len() {
                        outvec.push(deep_merge(&arr1[i], &arr2[i]));
                    } else {
                        outvec.push(arr2[i].clone());
                    }
                }
                Value::Array(outvec)
            }
            _ => Value::Array(arr2.clone()),
        },
        Value::Object(map2) => {
            match value1 {
                Value::Object(map1) => {
                    let mut outmap = Map::new();
                    // Create a set from map2 keys
                    let map2keys: Vec<String> = map2.keys().cloned().collect();
                    let map2keys: std::collections::HashSet<String> =
                        map2keys.into_iter().collect();

                    for (key, value) in map2 {
                        if map1.contains_key(key) {
                            outmap.insert(key.clone(), deep_merge(&map1[key], value));
                        } else {
                            outmap.insert(key.clone(), value.clone());
                        }
                    }
                    // Also keep keys that are in map1 but not in map2
                    for (key, value) in map1 {
                        if !map2keys.contains(key) {
                            outmap.insert(key.clone(), value.clone());
                        }
                    }
                    Value::Object(outmap)
                }
                _ => Value::Object(map2.clone()),
            }
        }
    }
}

// This ensures that all numbers are f64, as implied by JSON RFC 7159
fn cleanup(value: Value) -> Result<Value, Error> {
    match value {
        Value::Object(map) => {
            let mut outmap = Map::new();
            for (key, value) in map {
                outmap.insert(key.clone(), cleanup(value.clone())?);
            }
            Ok(Value::Object(outmap))
        }
        Value::Array(vec) => {
            let mut outvec = Vec::new();
            for value in vec {
                outvec.push(cleanup(value)?);
            }
            Ok(Value::Array(outvec))
        }
        Value::Number(n) => {
            let n = n.as_f64();
            if n.is_none() {
                return Err(Error::InvalidJsonValue);
            }
            let n = Number::from_f64(n.unwrap());
            if n.is_none() {
                return Err(Error::InvalidJsonValue);
            }
            Ok(Value::Number(n.unwrap()))
        }
        v => Ok(v.clone()),
    }
}

// handle errors by converting them into something that implements
// `IntoResponse`
fn handle_error<A>(err: Error) -> Result<A, (StatusCode, String)> {
    Err((
        StatusCode::BAD_REQUEST,
        format!("Something went wrong: {err:?}"),
    ))
}

pub fn route(router: Router) -> Router {
    router
        .route("/state", get(get_root))
        .route("/state", post(post_root))
}

async fn get_root() -> Json<Value> {
    Json((*get_state()).clone())
}

async fn post_root(payload: Json<Value>) -> Result<(), (StatusCode, String)> {
    merge_state(payload.0, RefreshSignal::Normal)
}

pub fn merge_state(payload: Value, signal: RefreshSignal) -> Result<(), (StatusCode, String)> {
    let mut state = STATE.lock().unwrap();

    let payload = cleanup(payload).or_else(handle_error)?;

    // Do a deep merge of the state and the payload.
    // Ignore the no-change case if the signal is RefreshSignal::Normal
    let current_value = state.root.borrow();
    let new_value = deep_merge(current_value, &payload);
    if matches!(signal, RefreshSignal::Normal) && (new_value == *current_value) {
        return Ok(());
    }

    state.root = Arc::new(new_value);

    trigger_draw(signal);
    Ok(())
}

pub fn get_state() -> Arc<Value> {
    STATE.lock().unwrap().root.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge() {
        let value1 = json!({
            "a": 1,
            "b": 2,
            "c": {
                "d": 3,
                "e": 4,
            },
            "f": [1, 2, 3],
        });

        let value2 = json!({
            "b": 3,
            "c": {
                "e": 5,
                "f": 6,
            },
            "f": [4, 5, 6],
        });

        let expected = json!({
            "a": 1,
            "b": 3,
            "c": {
                "d": 3,
                "e": 5,
                "f": 6,
            },
            "f": [4, 5, 6],
        });

        let result = deep_merge(&value1, &value2);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cleanup() {
        let value = json!({
            "a": 1,
            "b": 2,
            "c": {
                "d": 3,
                "e": 4,
            },
            "f": [1, 2, 3],
        });

        let expected = json!({
            "a": 1.0,
            "b": 2.0,
            "c": {
                "d": 3.0,
                "e": 4.0,
            },
            "f": [1.0, 2.0, 3.0],
        });

        let result = cleanup(value).unwrap();
        assert_eq!(result, expected);
    }
}
