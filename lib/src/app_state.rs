use crate::aws::{SnsClient, SqsClient};
use crate::error::OutboxPatternProcessorError;
use crate::http_gateway::HttpGateway;
use sqlx::{Pool, Postgres, Transaction};

#[derive(Clone)]
pub struct AppState {
    pub postgres_pool: Pool<Postgres>,
    pub sqs_client: Option<SqsClient>,
    pub sns_client: Option<SnsClient>,
    pub http_gateway: HttpGateway,
    pub outbox_query_limit: Option<u32>,
    pub delete_after_process_successfully: Option<bool>,
    pub max_in_flight_interval_in_seconds: Option<u64>,
    pub outbox_failure_limit: Option<u32>,
}

impl AppState {
    pub async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>, OutboxPatternProcessorError> {
        self.postgres_pool
            .begin()
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to create transaction"))
    }

    pub async fn commit_transaction(
        &self,
        transaction: Transaction<'_, Postgres>,
    ) -> Result<(), OutboxPatternProcessorError> {
        transaction
            .commit()
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to commit transaction"))?;

        Ok(())
    }
}
