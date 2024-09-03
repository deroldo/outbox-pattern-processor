use crate::app_state::AppState;
use crate::error::OutboxPatternProcessorError;
use crate::outbox::Outbox;
use crate::outbox_repository::OutboxRepository;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

impl OutboxRepository {
    pub async fn list(app_state: &AppState) -> Result<Vec<Outbox>, OutboxPatternProcessorError> {
        let processing_until_incremente_interval = format!("{} seconds", app_state.max_in_flight_interval_in_seconds.unwrap_or(30));

        let sql = r#"
        with locked as (
            update outbox o1
            set processing_until = now() + ($2)::interval
            from (
                select partition_key
                from outbox o3
                where o3.processed_at is null and o3.processing_until < now() and attempts < $3
                group by o3.partition_key
                order by min(o3.created_at)
                limit $1
            ) as o2
            where o1.partition_key = o2.partition_key and o1.processed_at is null and o1.processing_until < now() and attempts < $3
            returning o1.idempotent_key
        ),
        to_process as (
            select
                outbox.idempotent_key,
                row_number() over (partition by outbox.partition_key order by outbox.created_at asc) as rnk
            from outbox
            inner join locked on locked.idempotent_key = outbox.idempotent_key
            where outbox.processed_at is null
        )
        select outbox.*
        from outbox
        inner join to_process on to_process.idempotent_key = outbox.idempotent_key and to_process.rnk = 1
        "#;

        sqlx::query_as(sql)
            .bind(app_state.outbox_query_limit.unwrap_or(50) as i32)
            .bind(processing_until_incremente_interval)
            .bind(app_state.outbox_failure_limit.unwrap_or(10) as i32)
            .fetch_all(&app_state.postgres_pool)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to list outboxes"))
    }

    pub async fn mark_as_processed(
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        let sql = r#"
        update outbox
        set processed_at = now(),
            attempts = attempts + 1
        where idempotent_key = ANY($1)
        "#;

        sqlx::query(sql)
            .bind(outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>())
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to mark outboxes as processed"))?;

        Self::unlock_partition_key(transaction, outboxes).await?;

        Ok(())
    }

    pub async fn delete_processed(
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        let sql = r#"
        delete from outbox
        where idempotent_key = ANY($1)
        "#;

        sqlx::query(sql)
            .bind(outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>())
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to delete processed outboxes"))?;

        Self::unlock_partition_key(transaction, outboxes).await?;

        Ok(())
    }

    async fn unlock_partition_key(
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        let sql = r#"
        update outbox
        set processing_until = now()
        where partition_key = ANY($1) and processed_at is null
        "#;

        sqlx::query(sql)
            .bind(outboxes.iter().map(|it| it.partition_key).collect::<Vec<Uuid>>())
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to unlock partition keys"))?;

        Ok(())
    }

    pub async fn increase_attempts(
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        let sql = r#"
        update outbox
        set attempts = attempts + 1
        where idempotent_key = ANY($1)
        "#;

        sqlx::query(sql)
            .bind(outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>())
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to increase attempts"))?;

        Ok(())
    }
}
