use crate::outbox_destination::OutboxDestination;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, FromRow, Clone, PartialEq)]
pub struct Outbox {
    pub idempotent_key: Uuid,
    pub partition_key: Uuid,
    pub destinations: Json<Vec<OutboxDestination>>,
    pub headers: Option<Json<HashMap<String, String>>>,
    pub payload: String,
    pub created_at: DateTime<Utc>,
    pub processing_until: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
pub struct GroupedOutboxed {
    pub sqs: HashMap<String, Vec<Outbox>>,
    pub sns: HashMap<String, Vec<Outbox>>,
    pub http: Vec<Outbox>,
}
