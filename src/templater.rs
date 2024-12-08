use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};

use once_cell::sync::Lazy;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use yaml_merge_keys::{merge_keys_serde, serde_yaml};

use crate::{error::Error, state::get_state, trigger_draw};

static TEMPLATE: Lazy<Mutex<Arc<String>>> = Lazy::new(|| Mutex::new(Arc::new("".to_string())));

pub fn route(router: Router) -> Router {
    router
        .route("/template", get(get_template))
        .route("/template", post(post_template))
        .route("/rendered", get(get_rendered))
}

struct JsonToYaml {
    json: Value,
}

impl From<JsonToYaml> for gtmpl::Value {
    fn from(json: JsonToYaml) -> Self {
        match json.json {
            Value::Object(map) => {
                let mut outmap: HashMap<String, gtmpl::Value> = HashMap::new();
                for (key, value) in map {
                    let value = gtmpl::Value::from(JsonToYaml { json: value });
                    outmap.insert(key.clone(), value);
                }
                gtmpl::Value::Map(outmap)
            }
            Value::Array(vec) => {
                let mut outvec = Vec::new();
                for value in vec {
                    outvec.push(gtmpl::Value::from(JsonToYaml { json: value }));
                }
                gtmpl::Value::Array(outvec)
            }
            Value::Number(n) => {
                let value: f64;
                if n.is_f64() {
                    value = n.as_f64().unwrap();
                } else if n.is_i64() {
                    value = n.as_i64().unwrap() as f64;
                } else if n.is_u64() {
                    value = n.as_u64().unwrap() as f64;
                } else {
                    value = 0.0;
                }

                gtmpl::Value::Number(gtmpl_value::Number::from(value))
            }
            Value::String(s) => gtmpl::Value::String(s),
            Value::Bool(b) => gtmpl::Value::Bool(b),
            Value::Null => gtmpl::Value::String("null".to_string()),
        }
    }
}

pub async fn get_template() -> Result<String, Error> {
    let template = TEMPLATE.lock().unwrap().clone();
    Ok((*template).clone())
}

pub async fn post_template(payload: String) -> Result<(), (StatusCode, String)> {
    let mut template = TEMPLATE.lock().unwrap();

    *template = Arc::new(payload);

    trigger_draw();
    Ok(())
}

pub async fn get_rendered() -> Result<String, Error> {
    let state = get_state();
    // let content: Vec<UIWrapper> = serde_yaml::from_value(merged_keys).unwrap();

    // println!("{:?}", content);
    let yaml =
        render(state).and_then(|yaml| serde_yaml::to_string(&yaml).map_err(Error::SerdeYaml))?;

    Ok(yaml)
}

pub fn render(state: Arc<Value>) -> Result<serde_yaml::Value, Error> {
    let template = TEMPLATE.lock().unwrap().clone();

    let yaml = gtmpl::template(
        &template,
        JsonToYaml {
            json: (*state).clone(),
        },
    )
    .map_err(Error::TemplateError)?;
    println!("render: yaml = {}", yaml);

    let raw_yaml = serde_yaml::from_str(&yaml).map_err(Error::SerdeYaml)?;
    println!("render: raw_yaml = {:?}", raw_yaml);
    let merged_keys = merge_keys_serde(raw_yaml).map_err(Error::MergeKeyError)?;
    println!("render: merged_yaml = {:?}", merged_keys);

    Ok(merged_keys)
}
