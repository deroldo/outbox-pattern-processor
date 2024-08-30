use crate::domain::outbox::Outbox;
use crate::infra::environment::Environment;
use crate::infra::error::AppError;
use crate::outbox_processor::OutboxProcessorResources;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub struct OutboxRepository;

impl OutboxRepository {
    pub async fn insert(
        transaction: &mut Transaction<'_, Postgres>,
        outbox: Outbox,
    ) -> Result<Outbox, AppError> {
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
            .map_err(|error| AppError::new(&error.to_string(), &format!("Failed to insert over partition_key={}", outbox.partition_key)))
    }

    pub async fn list(
        resources: &OutboxProcessorResources,
        limit: i32,
    ) -> Result<Vec<Outbox>, AppError> {
        let processing_until_incremente_interval = Environment::string("OUTBOX_PROCESSING_MAX_WAIT_IN_SQL_INTERVAL", "30 seconds");

        let sql = r#"
        with locked as (
            update outbox o1
            set processing_until = now() + ($2)::interval
            from (
                select partition_key
                from outbox o3
                where o3.processed_at is null and o3.processing_until < now()
                group by o3.partition_key
                order by min(o3.created_at)
                limit $1
            ) as o2
            where o1.partition_key = o2.partition_key and o1.processed_at is null and o1.processing_until < now()
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
            .bind(limit)
            .bind(processing_until_incremente_interval)
            .fetch_all(&resources.postgres_pool)
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to list outboxes"))
    }

    pub async fn mark_as_processed(
        resources: &OutboxProcessorResources,
        outboxes: &[Outbox],
    ) -> Result<(), AppError> {
        let sql = r#"
        update outbox
        set processed_at = now()
        where idempotent_key = ANY($1)
        "#;

        sqlx::query(sql)
            .bind(outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>())
            .execute(&resources.postgres_pool)
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to mark outboxes as processed"))?;

        Self::unlock_partition_key(resources, outboxes).await?;

        Ok(())
    }

    pub async fn delete_processed(
        resources: &OutboxProcessorResources,
        outboxes: &[Outbox],
    ) -> Result<(), AppError> {
        let sql = r#"
        delete from outbox
        where idempotent_key = ANY($1)
        "#;

        sqlx::query(sql)
            .bind(outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>())
            .execute(&resources.postgres_pool)
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to delete processed outboxes"))?;

        Self::unlock_partition_key(resources, outboxes).await?;

        Ok(())
    }

    async fn unlock_partition_key(
        resources: &OutboxProcessorResources,
        outboxes: &[Outbox],
    ) -> Result<(), AppError> {
        let sql = r#"
        update outbox
        set processing_until = now()
        where partition_key = ANY($1) and processed_at is null
        "#;

        sqlx::query(sql)
            .bind(outboxes.iter().map(|it| it.partition_key).collect::<Vec<Uuid>>())
            .execute(&resources.postgres_pool)
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to delete processed outboxes"))?;

        Ok(())
    }
}
