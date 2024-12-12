use axum::response::{IntoResponse, Response};
use gtmpl::TemplateError;
use yaml_merge_keys::{serde_yaml, MergeKeyError};

#[derive(Debug)]
pub enum Error {
    SerdeYaml(serde_yaml::Error),
    TemplateError(TemplateError),
    MergeKeyError(MergeKeyError),
    InvalidPrimitive(i32, serde_yaml::Error),
    DrawingError(),
    HWError(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let r = match self {
            Error::SerdeYaml(e) => format!("Serde YAML error: {e}"),
            Error::TemplateError(e) => format!("Template error: {e}"),
            Error::MergeKeyError(e) => format!("Merge key error: {e}"),
            Error::InvalidPrimitive(i, e) => format!("Invalid primitive at index {}: {}", i, e),
            Error::HWError(r) => format!("Hardware error: {r}"),
            Error::DrawingError() => "Drawing error".to_string(),
        };

        println!("Error: {r}");
        (axum::http::StatusCode::BAD_REQUEST, r).into_response()
    }
}
