use crate::error::OutboxPatternProcessorError;
use crate::outbox::Outbox;
use sqlx::{Postgres, Transaction};
use tracing::instrument;

pub struct OutboxRepository;

impl OutboxRepository {
    #[instrument(skip_all)]
    pub async fn insert(
        transaction: &mut Transaction<'_, Postgres>,
        outbox: Outbox,
    ) -> Result<Outbox, OutboxPatternProcessorError> {
        if let Some(process_after) = outbox.process_after {
            let sql = r#"
            insert into outbox
                (idempotent_key, partition_key, destinations, headers, payload, process_after)
            values
                ($1, $2, $3, $4, $5, $6)
            returning *
            "#;

            sqlx::query_as(sql)
                .bind(outbox.idempotent_key)
                .bind(outbox.partition_key)
                .bind(outbox.destinations)
                .bind(outbox.headers)
                .bind(outbox.payload)
                .bind(process_after)
                .fetch_one(&mut **transaction)
                .await
                .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), &format!("Failed to insert over partition_key={}", outbox.partition_key)))
        } else {
            let sql = r#"
            insert into outbox
                (idempotent_key, partition_key, destinations, headers, payload)
            values
                ($1, $2, $3, $4, $5)
            returning *
            "#;

            sqlx::query_as(sql)
                .bind(outbox.idempotent_key)
                .bind(outbox.partition_key)
                .bind(outbox.destinations)
                .bind(outbox.headers)
                .bind(outbox.payload)
                .fetch_one(&mut **transaction)
                .await
                .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), &format!("Failed to insert over partition_key={}", outbox.partition_key)))
        }
    }
}
