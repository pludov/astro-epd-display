use std::convert::Infallible;

use crate::error::DrawingError;

pub trait IntoDrawingError {
    fn into_drawing_error(self) -> DrawingError;
}

impl<U: IntoDrawingError> From<U> for DrawingError {
    fn from(e: U) -> DrawingError {
        e.into_drawing_error()
    }
}

impl IntoDrawingError for Infallible {
    fn into_drawing_error(self) -> DrawingError {
        panic!("Infallible failed")
    }
}
