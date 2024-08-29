use crate::infra::error::AppError;
use axum::extract::FromRequest;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}
