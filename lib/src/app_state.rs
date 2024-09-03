use crate::aws::{SnsClient, SqsClient};
use crate::http_gateway::HttpGateway;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub postgres_pool: Pool<Postgres>,
    pub sqs_client: SqsClient,
    pub sns_client: SnsClient,
    pub http_gateway: HttpGateway,
    pub outbox_query_limit: Option<u32>,
    pub delete_after_process_successfully: Option<bool>,
    pub max_in_flight_interval_in_seconds: Option<u64>,
}
