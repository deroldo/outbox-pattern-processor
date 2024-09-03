use crate::error::OutboxPatternProcessorError;
use crate::outbox::Outbox;
use sqlx::{Postgres, Transaction};

pub struct OutboxRepository;

impl OutboxRepository {
    pub async fn insert(
        transaction: &mut Transaction<'_, Postgres>,
        outbox: Outbox,
    ) -> Result<Outbox, OutboxPatternProcessorError> {
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