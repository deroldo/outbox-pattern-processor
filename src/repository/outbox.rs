use crate::domain::outbox::Outbox;
use crate::infra::error::AppError;
use crate::state::AppState;
use uuid::Uuid;

pub struct OutboxRepository;

impl OutboxRepository {
    pub async fn list(
        app_state: &AppState,
        limit: i32,
    ) -> Result<Vec<Outbox>, AppError> {
        let sql = r#"
        with ranked as (
            select
                row_number() over (partition by partition_key order by created_at asc) as rnk,
                *
            from outbox
            where processed_at is null and processing_until < now()
            order by created_at
            limit $1
        ),
        processing as (
            update outbox o
            set processing_until = now() + ('1 minutes')::interval
            from ranked r
            where r.rnk = 1 and r.idempotent_key = o.idempotent_key and o.processing_until < now()
            returning o.*
        ),
        locked as (
            update outbox o
            set processing_until = now() + ('1 minutes')::interval
            from processing p
            where p.partition_key = o.partition_key and o.processed_at is null and o.processing_until < now()
            returning o.idempotent_key
        )
        select * from processing order by created_at;
        "#;

        sqlx::query_as(sql)
            .bind(limit)
            .fetch_all(&app_state.postgres_pool)
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to list outboxes"))
    }

    pub async fn mask_as_processed(
        app_state: &AppState,
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
            .execute(&app_state.postgres_pool)
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to mark outboxes as processed"))?;

        Ok(())
    }

    pub async fn delete_processed(
        app_state: &AppState,
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
            .execute(&app_state.postgres_pool)
            .await
            .map_err(|error| AppError::new(&error.to_string(), "Failed to delete processed outboxes"))?;

        Ok(())
    }
}
