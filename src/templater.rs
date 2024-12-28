use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};

use gtmpl::{Context, FuncError, Template};
use gtmpl_value::Number;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
    time::SystemTime,
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
    let yaml = render(state, SystemTime::now())
        .and_then(|(yaml, _)| serde_yaml::to_string(&yaml).map_err(Error::SerdeYaml))?;

    Ok(yaml)
}

struct RenderHiddenContext {
    now: SystemTime,
    next: Option<SystemTime>,
}

thread_local! {
    pub static HIDDEN_CONTEXT: RefCell<RenderHiddenContext> = RefCell::new(RenderHiddenContext{
        now: SystemTime::UNIX_EPOCH,
        next: None,
    });
}

pub fn render(
    state: Arc<Value>,
    now: SystemTime,
) -> Result<(serde_yaml::Value, Option<SystemTime>), Error> {
    let template = TEMPLATE.lock().unwrap().clone();
    render_template(template, state, now)
}

// Return a f64 from a gtmpl value
fn gtmpl_number(n: &Number) -> Result<f64, FuncError> {
    let v = n.as_f64();
    if v.is_some() {
        return Ok(v.unwrap());
    }

    let v = n.as_i64().map(|e| e as f64);
    if v.is_some() {
        return Ok(v.unwrap());
    }

    let v = n.as_i64().map(|e| e as f64);
    if v.is_some() {
        return Ok(v.unwrap());
    }
    Err(FuncError::UnableToConvertFromValue)
}

/// Function to return the current time, rounded to the nearest divisor
/// The default divisor is 60 seconds
fn func_time(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    HIDDEN_CONTEXT.with(|h| {
        let divisor = if args.len() > 0 {
            match args[0] {
                gtmpl::Value::Number(ref n) => gtmpl_number(n)?,
                _ => {
                    return Err(FuncError::UnableToConvertFromValue);
                }
            }
        } else {
            60.0
        };
        let mut ctx = h.borrow_mut();

        // Duration since epoch
        let mut now_num = ctx
            .now
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|e| e.as_secs_f64())
            .map_err(|_| FuncError::UnableToConvertFromValue)?;

        now_num = (now_num / divisor).floor() * divisor;

        ctx.next =
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs_f64(now_num + divisor));

        Ok(gtmpl::Value::from(now_num))
    })
}

fn render_template(
    template: Arc<String>,
    state: Arc<Value>,
    now: SystemTime,
) -> Result<(serde_yaml::Value, Option<SystemTime>), Error> {
    let context = JsonToYaml {
        json: (*state).clone(),
    };

    HIDDEN_CONTEXT.with(|h| {
        let mut ctx = h.borrow_mut();
        ctx.now = now;
        ctx.next = None;
    });

    let mut tmpl = Template::default();
    tmpl.add_func("time", func_time);
    tmpl.parse((*template).clone())
        .map_err(Into::into)
        .map_err(Error::TemplateError)?;

    let yaml: String = tmpl
        .render(&Context::from(context))
        .map_err(Into::into)
        .map_err(Error::TemplateError)?;

    // println!("render: yaml = {}", yaml);

    let raw_yaml = serde_yaml::from_str(&yaml).map_err(Error::SerdeYaml)?;
    // println!("render: raw_yaml = {:?}", raw_yaml);
    let merged_keys = merge_keys_serde(raw_yaml).map_err(Error::MergeKeyError)?;
    // println!("render: merged_yaml = {:?}", merged_keys);

    let next = HIDDEN_CONTEXT.with(|h| {
        let ctx = h.borrow();
        ctx.next
    });

    Ok((merged_keys, next))
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    pub use super::*;
    use serde_json::json;

    // Test a simple rendering
    #[test]
    fn test_render() {
        let state = Arc::new(json!({
            "name": "world",
        }));

        let template = Arc::new("Hello: '{{ .name }}!'".to_string());

        let now = SystemTime::now();

        let (yaml, next) = render_template(template, state, now).unwrap();

        let mut expected_map = serde_yaml::Mapping::new();
        expected_map.insert(
            serde_yaml::Value::String("Hello".to_string()),
            serde_yaml::Value::String("world!".to_string()),
        );

        assert_eq!(yaml, serde_yaml::Value::Mapping(expected_map));
        assert_eq!(next, None);
    }

    #[test]
    fn test_time() {
        let state = Arc::new(json!({
            "name": "world",
        }));

        let template = Arc::new("Hello: {{ time 60 }}".to_string());

        let now_sec = 3600 * 55 * 365 * 24;
        let now_next_min = now_sec + 60;

        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(now_sec);

        let (yaml, next) = render_template(template, state, now).unwrap();

        let mut expected_map = serde_yaml::Mapping::new();
        expected_map.insert(
            serde_yaml::Value::String("Hello".to_string()),
            serde_yaml::Value::Number(now_sec.into()),
        );

        assert_eq!(yaml, serde_yaml::Value::Mapping(expected_map));
        assert_eq!(
            next,
            Some(SystemTime::UNIX_EPOCH + Duration::from_secs(now_next_min))
        );
    }
}
