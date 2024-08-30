use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use std::collections::HashMap;
use tracing::info;

pub struct HealthRoutes;

impl HealthRoutes {
    pub fn routes() -> Router {
        Router::new().route("/", get(health_handler))
    }
}

async fn health_handler() -> Result<Json<HashMap<&'static str, &'static str>>, (StatusCode, String)> {
    info!("GET /health");
    Ok(Json(HashMap::from([("status", "up")])))
}
