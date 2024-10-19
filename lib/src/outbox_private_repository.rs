use crate::app_state::AppState;
use crate::error::OutboxPatternProcessorError;
use crate::outbox::Outbox;
use crate::outbox_repository::OutboxRepository;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

impl OutboxRepository {
    pub async fn list(app_state: &AppState) -> Result<Vec<Outbox>, OutboxPatternProcessorError> {
        let sql_list = r#"with locked_partition_key as (
    insert into outbox_lock
    (
        select o.partition_key, $1 as lock_id, now() + ($3)::interval as processing_until
        from outbox o
             left join outbox_lock ol on o.partition_key = ol.partition_key
        where ol.partition_key is null and o.processed_at is null and o.process_after < now() and o.attempts < $4
        group by o.partition_key
        order by min(o.process_after)
        limit $2
    )
    ON CONFLICT DO NOTHING
    RETURNING *
),
locked as (
    select
        o.idempotent_key,
        row_number() over (partition by o.partition_key order by o.process_after asc) as rnk
    from outbox o
         inner join locked_partition_key lpk on o.partition_key = lpk.partition_key
    where o.process_after < now()
        and o.processed_at is null
        and o.attempts < $4
)
select o.*
from outbox o
     inner join locked l on o.idempotent_key = l.idempotent_key and l.rnk = 1;"#;

        let lock_id = Uuid::now_v7();
        let processing_until_incremente_interval = format!("{} seconds", app_state.max_in_flight_interval_in_seconds.unwrap_or(30));

        sqlx::query_as(sql_list)
            .bind(lock_id)
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

        let mut ids_to_update = outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>();
        ids_to_update.push(Uuid::now_v7());

        sqlx::query(sql)
            .bind(ids_to_update)
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

        let mut ids_to_delete = outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>();
        ids_to_delete.push(Uuid::now_v7());

        sqlx::query(sql)
            .bind(ids_to_delete)
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to delete processed outboxes"))?;

        Self::unlock_partition_key(transaction, outboxes).await?;

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

        Self::unlock_partition_key(transaction, outboxes).await?;

        Ok(())
    }

    async fn unlock_partition_key(
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        let sql = r#"
        delete from outbox_lock
        where partition_key = ANY($1) or processing_until < now()
        "#;

        let mut ids_to_delete = outboxes.iter().map(|it| it.partition_key).collect::<Vec<Uuid>>();
        ids_to_delete.push(Uuid::now_v7());

        sqlx::query(sql)
            .bind(ids_to_delete)
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to unlock partition keys"))?;

        Ok(())
    }
}
