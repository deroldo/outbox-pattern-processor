use aws_config::BehaviorVersion;
use outbox_pattern_processor::infra::aws::{SnsClient, SqsClient};
use outbox_pattern_processor::infra::database::Database;
use outbox_pattern_processor::infra::environment::Environment;
use outbox_pattern_processor::infra::error::AppError;
use outbox_pattern_processor::infra::http_gateway::HttpGateway;
use sqlx::{Pool, Postgres, Transaction};

#[derive(Clone)]
pub struct AppState {
    pub postgres_pool: Pool<Postgres>,
    pub sqs_client: SqsClient,
    pub sns_client: SnsClient,
    pub http_gateway: HttpGateway,
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
        let http_gateway = HttpGateway::new(Environment::u64("HTTP_TIMEOUT_IN_MILLIS", 3000))?;

        Ok(Self {
            postgres_pool,
            sqs_client,
            sns_client,
            http_gateway,
        })
    }

    pub async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>, AppError> {
        self.postgres_pool
            .begin()
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to created database transaction"))
    }

    pub async fn commit_transaction(
        &self,
        transaction: Transaction<'_, Postgres>,
    ) -> Result<(), AppError> {
        transaction
            .commit()
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to commit database transaction"))?;
        Ok(())
    }
}
