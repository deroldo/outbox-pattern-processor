use crate::controller::health::routes::HealthRoutes;
use axum::Router;

pub struct Routes;

impl Routes {
    pub async fn routes() -> Router {
        Router::new().nest("/health", HealthRoutes::routes())
    }
}
