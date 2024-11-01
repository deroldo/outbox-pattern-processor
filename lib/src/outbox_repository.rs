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
        let outboxes = Self::insert_all(transaction, vec![outbox]).await?;
        Ok(outboxes[0].clone())
    }

    pub async fn insert_all(
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: Vec<Outbox>,
    ) -> Result<Vec<Outbox>, OutboxPatternProcessorError> {
        if outboxes.is_empty() {
            return Ok(vec![]);
        }

        let sql = r#"
            INSERT INTO outbox
                (idempotent_key, partition_key, destinations, headers, payload, created_at, process_after)
        "#;

        let mut query_builder = sqlx::QueryBuilder::new(sql);

        query_builder.push_values(outboxes.clone(), |mut b, outbox: Outbox| {
            b.push_bind(outbox.idempotent_key)
                .push_bind(outbox.partition_key)
                .push_bind(outbox.destinations)
                .push_bind(outbox.headers)
                .push_bind(outbox.payload)
                .push_bind(outbox.created_at)
                .push_bind(outbox.process_after.unwrap_or(outbox.created_at));
        });

        query_builder.build().execute(&mut **transaction).await.map_err(|error| {
            OutboxPatternProcessorError::new(
                &error.to_string(),
                &format!(
                    "Failed to insert outboxes to partition_keys={}",
                    outboxes.iter().map(|it| it.partition_key.to_string()).collect::<Vec<_>>().join(", ")
                ),
            )
        })?;

        Ok(outboxes)
    }
}
