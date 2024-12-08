use crate::error::Error;
use crate::renderer::PrimitiveWrapper;
use crate::{renderer, state, templater};
use axum::{routing::get, Json, Router};
use serde::Serialize;
use yaml_merge_keys::serde_yaml;

#[derive(Debug, Serialize)]
struct DebugResult {
    state: serde_json::Value,
    yaml: Option<serde_yaml::Value>,
    yaml_error: Option<String>,
    primitives: Option<Vec<PrimitiveWrapper>>,
    primitives_error: Option<String>,
}

pub fn route(router: Router) -> Router {
    router.route("/debug", get(get_debug))
}

async fn get_debug() -> Result<Json<DebugResult>, Error> {
    let state = state::get_state();
    let mut result = DebugResult {
        state: (*state).clone(),
        yaml: None,
        yaml_error: None,
        primitives: None,
        primitives_error: None,
    };
    println!("State: {:?}", state);
    let yaml = templater::render(state);
    if yaml.is_err() {
        result.yaml_error = Some(format!("{:?}", yaml.err()));
    } else {
        let yaml = yaml.unwrap();
        result.yaml = Some(yaml.clone());

        let primitives = renderer::parse(yaml);
        if primitives.is_err() {
            result.primitives_error = Some(format!("{:?}", primitives.err()));
        } else {
            result.primitives = Some(
                primitives
                    .unwrap()
                    .into_iter()
                    .map(|p| PrimitiveWrapper(p))
                    .collect(),
            );
        }
    }
    // serde_json::to_value(primitives).map_err(|_| Error::InvalidJsonValue)?

    return Ok(Json(result));
}
