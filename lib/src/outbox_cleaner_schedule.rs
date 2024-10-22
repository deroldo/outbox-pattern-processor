use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone, PartialEq)]
pub struct OutboxCleanerSchedule {
    pub cron_expression: String,
    pub last_execution: DateTime<Utc>,
}
