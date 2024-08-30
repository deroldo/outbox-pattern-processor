use crate::domain::outbox::Outbox;
use crate::infra::error::AppError;
use crate::outbox_processor::OutboxProcessorResources;
use uuid::Uuid;

pub struct OutboxRepository;

impl OutboxRepository {
    pub async fn list(
        resources: &OutboxProcessorResources,
        limit: i32,
    ) -> Result<Vec<Outbox>, AppError> {
        let sql = r#"
        with locked as (
            update outbox o1
            set processing_until = now() + ('1 minutes')::interval
            from (
                select partition_key
                from outbox o3
                where o3.processed_at is null and o3.processing_until < now()
                group by o3.partition_key
                order by min(o3.created_at)
                limit $1
            ) as o2
            where o1.partition_key = o2.partition_key
                and o1.processing_until < now()
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
        inner join to_process on to_process.idempotent_key = outbox.idempotent_key
        where to_process.rnk = 1;
        "#;

        sqlx::query_as(sql)
            .bind(limit)
            .fetch_all(&resources.postgres_pool)
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to list outboxes"))
    }

    pub async fn mask_as_processed(
        resources: &OutboxProcessorResources,
        outboxes: &[Outbox],
    ) -> Result<(), AppError> {
        let sql = r#"
        WITH unlocked AS (
            update outbox
            set processing_until = now()
            where partition_key in (
                select partition_key
                from outbox o2
                where o2.idempotent_key = ANY($1)
            )
        )
        update outbox
        set processed_at = now()
        where idempotent_key = ANY($1);
        "#;

        sqlx::query(sql)
            .bind(outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>())
            .execute(&resources.postgres_pool)
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to mark outboxes as processed"))?;

        Ok(())
    }

    pub async fn delete_processed(
        resources: &OutboxProcessorResources,
        outboxes: &[Outbox],
    ) -> Result<(), AppError> {
        let sql = r#"
        WITH unlocked AS (
            update outbox
            set processing_until = now()
            where partition_key in (
                select partition_key
                from outbox o2
                where o2.idempotent_key = ANY($1)
            )
        )
        delete from outbox
        where idempotent_key = ANY($1);
        "#;

        sqlx::query(sql)
            .bind(outboxes.iter().map(|it| it.idempotent_key).collect::<Vec<Uuid>>())
            .execute(&resources.postgres_pool)
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to delete processed outboxes"))?;

        Ok(())
    }
}
