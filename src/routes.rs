use crate::controller::health::routes::HealthRoutes;
use crate::state::AppState;
use axum::Router;

pub struct Routes;

impl Routes {
    pub async fn routes(_app_state: &AppState) -> Router {
        Router::new().nest("/health", HealthRoutes::routes())
    }
}
