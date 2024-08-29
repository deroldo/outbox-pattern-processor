use axum::extract::rejection::{JsonDataError, JsonRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::fmt;
use tracing::{error, info};

#[derive(Debug)]
pub struct AppError {
    pub status_code: StatusCode,
    pub cause: String,
    pub message: Option<String>,
}

impl AppError {
    pub fn new(
        cause: &str,
        message: &str,
    ) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            cause: cause.to_string(),
            message: Some(message.to_string()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "cause": self.cause,
            "message": self.message,
        }));

        if self.status_code.is_server_error() {
            error!(self.cause);
        } else if self.status_code.is_client_error() {
            info!(self.cause);
        }

        let response = (self.status_code, body).into_response();

        let (parts, body) = response.into_parts();

        Response::from_parts(parts, body)
    }
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl From<JsonRejection> for AppError {
    fn from(inner: JsonRejection) -> Self {
        Self {
            status_code: inner.status(),
            cause: inner.to_string(),
            message: Some(inner.body_text()),
        }
    }
}

impl From<JsonDataError> for AppError {
    fn from(inner: JsonDataError) -> Self {
        Self {
            status_code: inner.status(),
            cause: inner.to_string(),
            message: Some(inner.body_text()),
        }
    }
}
