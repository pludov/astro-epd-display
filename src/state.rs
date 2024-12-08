use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use once_cell::sync::Lazy;
use serde_json::{Map, Number, Value};
use std::sync::{Arc, Mutex};

use crate::trigger_draw;

#[derive(Debug)]
struct State {
    root: Arc<Value>,
}

static STATE: Lazy<Mutex<State>> = Lazy::new(|| {
    Mutex::new(State {
        root: Arc::new(Value::Null),
    })
});

#[derive(Debug)]
enum Error {
    InvalidJsonValue,
}

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
            let mut outvec = vec.clone();
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
    let mut state = STATE.lock().unwrap();
    state.root = Arc::new(cleanup(payload.0).or_else(handle_error)?);

    trigger_draw();
    Ok(())
}

pub fn get_state() -> Arc<Value> {
    STATE.lock().unwrap().root.clone()
}
