use crate::infra::database::Database;
use crate::infra::error::AppError;
use aws_config::BehaviorVersion;
use outbox_pattern_processor::aws::{SnsClient, SqsClient};
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub postgres_pool: Pool<Postgres>,
    pub sqs_client: SqsClient,
    pub sns_client: SnsClient,
}

impl AppState {
    pub async fn new() -> Result<Self, AppError> {
        let db_config = Database::from_env();
        let postgres_pool = db_config
            .create_db_pool()
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to created database pool"))?;

        let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let sqs_client = SqsClient::new(&aws_config).await;
        let sns_client = SnsClient::new(&aws_config).await;

        Ok(Self {
            postgres_pool,
            sqs_client,
            sns_client,
        })
    }
}
