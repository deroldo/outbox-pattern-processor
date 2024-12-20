use crate::app_state::AppState;
use crate::error::OutboxPatternProcessorError;
use crate::outbox::Outbox;
use crate::outbox_cleaner_schedule::OutboxCleanerSchedule;
use crate::outbox_repository::OutboxRepository;
use sqlx::types::chrono::Utc;
use sqlx::{Postgres, Transaction};
use std::time::Duration;
use tracing::instrument;
use uuid::Uuid;

impl OutboxRepository {
    #[instrument(skip_all, name = "lock_and_get_outboxes")]
    pub async fn list(app_state: &AppState) -> Result<Vec<Outbox>, OutboxPatternProcessorError> {
        let lock_id = Uuid::now_v7();
        let processing_until_incremente_interval = format!("{} seconds", app_state.max_in_flight_interval_in_seconds.unwrap_or(30));

        Self::lock_partition_key(&app_state, lock_id, processing_until_incremente_interval).await?;

        Self::get_outboxes_ranked_from_locked_partition_key(&app_state, lock_id).await
    }

    #[instrument(skip_all)]
    async fn get_outboxes_ranked_from_locked_partition_key(
        app_state: &&AppState,
        lock_id: Uuid,
    ) -> Result<Vec<Outbox>, OutboxPatternProcessorError> {
        let sql_list = r#"with locked as (
    select
        o.idempotent_key,
        row_number() over (partition by o.partition_key order by o.process_after asc) as rnk
    from outbox o
    inner join outbox_lock ol on o.partition_key = ol.partition_key
    where o.process_after < now()
        and o.processed_at is null
        and o.attempts < $2
        and ol.lock_id = $1
)
select o.*
from outbox o
inner join locked l on o.idempotent_key = l.idempotent_key
where l.rnk = 1"#;

        sqlx::query_as(sql_list)
            .bind(lock_id)
            .bind(app_state.outbox_failure_limit.unwrap_or(10) as i32)
            .fetch_all(&app_state.postgres_pool)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to list outboxes"))
    }

    #[instrument(skip_all)]
    async fn lock_partition_key(
        app_state: &&AppState,
        lock_id: Uuid,
        processing_until_incremente_interval: String,
    ) -> Result<(), OutboxPatternProcessorError> {
        let sql_lock = r#"insert into outbox_lock (partition_key, lock_id, processing_until)
(
    select o.partition_key, $1 as lock_id, now() + ($3)::interval as processing_until
    from outbox o
    left join outbox_lock ol on o.partition_key = ol.partition_key and ol.processed_at is null
    where ol.partition_key is null
        and o.processed_at is null
        and o.process_after < now()
        and o.attempts < $4
    group by o.partition_key
    order by min(o.process_after)
    limit $2
)
ON CONFLICT DO NOTHING"#;

        sqlx::query(sql_lock)
            .bind(lock_id)
            .bind(app_state.outbox_query_limit.unwrap_or(50) as i32)
            .bind(processing_until_incremente_interval)
            .bind(app_state.outbox_failure_limit.unwrap_or(10) as i32)
            .execute(&app_state.postgres_pool)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to lock outboxes"))?;
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn mark_as_processed(
        app_state: &AppState,
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        let sql = "update outbox set processed_at = now(), attempts = attempts + 1 where idempotent_key = ANY($1)";

        let mut ids_to_update = outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>();
        ids_to_update.push(Uuid::now_v7());

        sqlx::query(sql)
            .bind(ids_to_update)
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to mark outboxes as processed"))?;

        Self::unlock(app_state, transaction, outboxes).await?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn delete_processed(
        app_state: &AppState,
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        let sql = "delete from outbox where idempotent_key = ANY($1)";

        let mut ids_to_delete = outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>();
        ids_to_delete.push(Uuid::now_v7());

        sqlx::query(sql)
            .bind(ids_to_delete)
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to delete processed outboxes"))?;

        Self::unlock(app_state, transaction, outboxes).await?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn increase_attempts(
        app_state: &AppState,
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        if let Some(delay) = app_state.delay_for_failure_attempt_in_seconds {
            if delay > 0 {
                let sql = "update outbox set process_after = $2 where partition_key = ANY($1) and processed_at is null";

                sqlx::query(sql)
                    .bind(outboxes.iter().map(|it| it.partition_key).collect::<Vec<Uuid>>())
                    .bind(Utc::now() + Duration::from_secs(delay))
                    .execute(&mut **transaction)
                    .await
                    .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to increase attempts"))?;
            }
        }

        let sql = "update outbox set attempts = attempts + 1 where idempotent_key = ANY($1)";

        sqlx::query(sql)
            .bind(outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>())
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to increase attempts"))?;

        Self::unlock(app_state, transaction, outboxes).await?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn unlock(
        app_state: &AppState,
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        Self::unlock_by_partition_key(app_state, transaction, outboxes).await?;
        Self::unlock_expired_processes(app_state, transaction, outboxes).await?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn unlock_expired_processes(
        app_state: &AppState,
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        let sql_unlock_by_processing_until = if app_state.scheduled_clear_locked_partition.unwrap_or(false) {
            "update outbox_lock set processed_at = now() where processing_until < now()"
        } else {
            "delete from outbox_lock where processing_until < now()"
        };

        let mut ids_to_delete = outboxes.iter().map(|it| it.partition_key).collect::<Vec<Uuid>>();
        ids_to_delete.push(Uuid::now_v7());

        sqlx::query(sql_unlock_by_processing_until)
            .bind(ids_to_delete)
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to unlock partition keys"))?;
        Ok(())
    }

    #[instrument(skip_all)]
    async fn unlock_by_partition_key(
        app_state: &AppState,
        transaction: &mut Transaction<'_, Postgres>,
        outboxes: &[Outbox],
    ) -> Result<(), OutboxPatternProcessorError> {
        let sql_unlock_by_partition_key = if app_state.scheduled_clear_locked_partition.unwrap_or(false) {
            "update outbox_lock set processed_at = now() where partition_key = ANY($1) and processed_at is null"
        } else {
            "delete from outbox_lock where partition_key = ANY($1) and processed_at is null"
        };

        let mut ids_to_delete = outboxes.iter().map(|it| it.partition_key).collect::<Vec<Uuid>>();
        ids_to_delete.push(Uuid::now_v7());

        sqlx::query(sql_unlock_by_partition_key)
            .bind(ids_to_delete)
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to unlock partition keys"))?;
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn clear_processed_locked_partition_key(transaction: &mut Transaction<'_, Postgres>) -> Result<(), OutboxPatternProcessorError> {
        let sql = "delete from outbox_lock where processed_at is not null and processed_at < now()";

        sqlx::query(sql)
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to clear processed locks"))?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn find_cleaner_schedule(transaction: &mut Transaction<'_, Postgres>) -> Result<Option<OutboxCleanerSchedule>, OutboxPatternProcessorError> {
        let sql = "select * from outbox_cleaner_schedule limit 1 for update skip locked";

        sqlx::query_as(sql)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to update last cleaner execution"))
    }

    #[instrument(skip_all)]
    pub async fn update_last_cleaner_execution(transaction: &mut Transaction<'_, Postgres>) -> Result<(), OutboxPatternProcessorError> {
        let sql = "update outbox_cleaner_schedule set last_execution = now()";

        sqlx::query(sql)
            .execute(&mut **transaction)
            .await
            .map_err(|error| OutboxPatternProcessorError::new(&error.to_string(), "Failed to update last cleaner execution"))?;

        Ok(())
    }
}
